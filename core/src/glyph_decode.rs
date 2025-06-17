use serde::{Deserialize, Serialize};
use serde_json5;
use ciborium::{into_writer, from_reader};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use blake3::Hasher;
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use clap::Parser;
use hex;
use bardecoder;
use image::io::Reader as ImageReader;
use base64;

// ============================================================================
// DATA STRUCTURES (Shared with glyph-encode.rs)
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct GlyphHeader {
    pub magic: String,
    pub version: String,
    pub id: String,
    pub hash: String,
    pub signature: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Origin {
    pub creator: String,
    pub timestamp: DateTime<Utc>,
    pub signature: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Namespace {
    pub provision: String,
    pub contextual_overlays: Vec<ContextualOverlay>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextualOverlay {
    pub agent: String,
    pub label: String,
    pub truth_mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TruthMode {
    pub r#type: String,
    pub confidence: f64,
    pub verified_by: Vec<String>,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Audio {
    pub file: Option<String>,
    pub phonetic: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Visual {
    pub svg: Option<String>,
    pub sigil: Option<String>,
    pub style: Option<VisualStyle>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VisualStyle {
    pub color: Option<String>,
    pub stroke: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Relation {
    pub r#type: String,
    pub target: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Grounding {
    pub wikidata: Option<String>,
    pub sensors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlyphPayload {
    pub label: String,
    pub aliases: Vec<String>,
    pub classes: Vec<String>,
    pub namespace: Namespace,
    pub origin: Option<Origin>,
    pub truth_mode: TruthMode,
    pub audio: Audio,
    pub visual: Visual,
    pub relations: Vec<Relation>,
    pub grounding: Grounding,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Glyph {
    pub header: GlyphHeader,
    pub payload: GlyphPayload,
}

// ============================================================================
// CLI ARGUMENTS
// ============================================================================

#[derive(Parser)]
#[command(name = "glyph-decode")]
#[command(about = "Decode binary .glyph files to human-readable formats")]
struct Args {
    /// Input .glyph file
    #[arg(short, long)]
    input: String,
    
    /// Output format (json, json5, yaml, text)
    #[arg(short, long, default_value = "json")]
    format: String,
    
    /// Output file (defaults to stdout)
    #[arg(short, long)]
    output: Option<String>,
    
    /// Verify signature if present
    #[arg(short, long)]
    verify: bool,
    
    /// Public key file for verification
    #[arg(long)]
    public_key: Option<String>,
    
    /// Show detailed information
    #[arg(short, long)]
    verbose: bool,
    
    /// Validate glyph structure
    #[arg(long)]
    validate: bool,
    
    /// Extract specific field (e.g., "payload.label", "header.hash")
    #[arg(long)]
    extract: Option<String>,
    
    /// Show only the header
    #[arg(long)]
    header_only: bool,
    
    /// Show only the payload
    #[arg(long)]
    payload_only: bool,
}

// ============================================================================
// MAIN DECODER LOGIC
// ============================================================================

impl Glyph {
    pub fn from_qr_code(qr_data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        // Load image from bytes
        let img = ImageReader::new(std::io::Cursor::new(qr_data))
            .with_guessed_format()?
            .decode()?;
        
        // Create QR decoder
        let decoder = bardecoder::default_decoder();
        
        // Decode QR code
        let results = decoder.decode(&img);
        
        // Find the first successful result
        let qr_content = results.into_iter()
            .filter_map(|r| r.ok())
            .next()
            .ok_or("No valid QR code found in image")?;
        
        // Decode base64 content back to CBOR bytes
        let cbor_data = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &qr_content)?;
        
        // Decode CBOR to glyph
        let glyph = Self::from_cbor(&cbor_data)?;
        Ok(glyph)
    }
    
    pub fn from_cbor(cbor_data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let glyph: Glyph = from_reader(cbor_data)?;
        Ok(glyph)
    }
    
    pub fn verify_signature(&self, public_key: &[u8]) -> Result<bool, Box<dyn std::error::Error>> {
        if let Some(signature_str) = &self.header.signature {
            // Parse signature
            let signature_hex = signature_str.strip_prefix("ed25519:")
                .ok_or("Invalid signature format")?;
            let signature_bytes = hex::decode(signature_hex)?;
            
            // Convert signature bytes to the expected format
            if signature_bytes.len() != 64 {
                return Err("Invalid signature length".into());
            }
            let mut sig_array = [0u8; 64];
            sig_array.copy_from_slice(&signature_bytes);
            let signature = Signature::from_bytes(&sig_array);
            
            // Convert public key to the expected format
            if public_key.len() != 32 {
                return Err("Invalid public key length".into());
            }
            let mut key_array = [0u8; 32];
            key_array.copy_from_slice(public_key);
            let verifying_key = VerifyingKey::from_bytes(&key_array)?;
            
            // Verify
            let message = self.header.hash.as_bytes();
            let is_valid = verifying_key.verify(message, &signature).is_ok();
            
            Ok(is_valid)
        } else {
            Ok(false) // No signature to verify
        }
    }
    
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // Validate header
        if self.header.magic != "GLYPH" {
            errors.push("Invalid magic bytes".to_string());
        }
        
        if self.header.version != "0.1" {
            errors.push("Unsupported version".to_string());
        }
        
        // Validate UUID format
        if Uuid::parse_str(&self.header.id).is_err() {
            errors.push("Invalid UUID format".to_string());
        }
        
        // Validate hash format
        if !self.header.hash.starts_with("blake3:") {
            errors.push("Invalid hash format".to_string());
        }
        
        // Validate payload
        if self.payload.label.is_empty() {
            errors.push("Label cannot be empty".to_string());
        }
        
        if self.payload.truth_mode.confidence < 0.0 || self.payload.truth_mode.confidence > 1.0 {
            errors.push("Confidence must be between 0.0 and 1.0".to_string());
        }
        
        // Validate relations
        for relation in &self.payload.relations {
            if relation.target.is_empty() {
                errors.push("Relation target cannot be empty".to_string());
            }
        }
        
        // Validate timestamp
        if self.header.created_at > Utc::now() {
            errors.push("Created timestamp is in the future".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    pub fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        Ok(json)
    }
    
    pub fn to_json5(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Note: serde_json5 doesn't have a to_string function, so we'll use regular JSON
        // and format it as JSON5-compatible
        let json = serde_json::to_string_pretty(self)?;
        Ok(json)
    }
    
    pub fn to_yaml(&self) -> Result<String, Box<dyn std::error::Error>> {
        let yaml = serde_yaml::to_string(self)?;
        Ok(yaml)
    }
    
    pub fn to_text(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut text = String::new();
        
        text.push_str(&format!("GLYPH: {}\n", self.payload.label));
        text.push_str(&format!("ID: {}\n", self.header.id));
        text.push_str(&format!("Version: {}\n", self.header.version));
        text.push_str(&format!("Created: {}\n", self.header.created_at));
        text.push_str(&format!("Hash: {}\n", self.header.hash));
        
        if let Some(signature) = &self.header.signature {
            text.push_str(&format!("Signature: {}\n", signature));
        }
        
        text.push_str(&format!("Classes: {}\n", self.payload.classes.join(", ")));
        text.push_str(&format!("Truth Mode: {} (confidence: {})\n", 
            self.payload.truth_mode.r#type, self.payload.truth_mode.confidence));
        
        if !self.payload.truth_mode.verified_by.is_empty() {
            text.push_str(&format!("Verified by: {}\n", self.payload.truth_mode.verified_by.join(", ")));
        }
        
        if !self.payload.relations.is_empty() {
            text.push_str("Relations:\n");
            for relation in &self.payload.relations {
                text.push_str(&format!("  {} -> {}\n", relation.r#type, relation.target));
            }
        }
        
        if let Some(wikidata) = &self.payload.grounding.wikidata {
            text.push_str(&format!("Wikidata: {}\n", wikidata));
        }
        
        Ok(text)
    }
    
    pub fn extract_field(&self, field_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        match field_path {
            "header.magic" => Ok(self.header.magic.clone()),
            "header.version" => Ok(self.header.version.clone()),
            "header.id" => Ok(self.header.id.clone()),
            "header.hash" => Ok(self.header.hash.clone()),
            "header.signature" => Ok(self.header.signature.clone().unwrap_or_default()),
            "header.created_at" => Ok(self.header.created_at.to_rfc3339()),
            "payload.label" => Ok(self.payload.label.clone()),
            "payload.truth_mode.type" => Ok(self.payload.truth_mode.r#type.clone()),
            "payload.truth_mode.confidence" => Ok(self.payload.truth_mode.confidence.to_string()),
            "payload.grounding.wikidata" => Ok(self.payload.grounding.wikidata.clone().unwrap_or_default()),
            _ => Err(format!("Unknown field: {}", field_path).into()),
        }
    }
    
    pub fn get_info(&self) -> String {
        format!(
            "Glyph: {} (ID: {})\nVersion: {}\nCreated: {}\nHash: {}\nSignature: {}\nClasses: {}\nRelations: {}",
            self.payload.label,
            self.header.id,
            self.header.version,
            self.header.created_at,
            self.header.hash,
            self.header.signature.as_deref().unwrap_or("None"),
            self.payload.classes.join(", "),
            self.payload.relations.len()
        )
    }
}

// ============================================================================
// MAIN FUNCTION
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Read input file
    let cbor_data = fs::read(&args.input)?;
    
    // Decode glyph
    let glyph = Glyph::from_cbor(&cbor_data)?;
    
    // Show info if verbose
    if args.verbose {
        println!("{}", glyph.get_info());
        println!();
    }
    
    // Validate if requested
    if args.validate {
        match glyph.validate() {
            Ok(()) => println!("✓ Glyph validation passed"),
            Err(errors) => {
                eprintln!("✗ Glyph validation failed:");
                for error in errors {
                    eprintln!("  - {}", error);
                }
                if !args.verbose {
                    std::process::exit(1);
                }
            }
        }
    }
    
    // Verify signature if requested
    if args.verify {
        if let Some(key_path) = args.public_key {
            let public_key = fs::read(key_path)?;
            match glyph.verify_signature(&public_key) {
                Ok(true) => println!("✓ Signature verification passed"),
                Ok(false) => {
                    if glyph.header.signature.is_some() {
                        eprintln!("✗ Signature verification failed");
                        if !args.verbose {
                            std::process::exit(1);
                        }
                    } else {
                        println!("ℹ No signature to verify");
                    }
                }
                Err(e) => {
                    eprintln!("✗ Signature verification error: {}", e);
                    if !args.verbose {
                        std::process::exit(1);
                    }
                }
            }
        } else {
            eprintln!("Warning: --verify specified but no --public-key provided");
        }
    }
    
    // Extract specific field if requested
    if let Some(field_path) = args.extract {
        let value = glyph.extract_field(&field_path)?;
        println!("{}", value);
        return Ok(());
    }
    
    // Determine output content
    let output_content = if args.header_only {
        serde_json::to_string_pretty(&glyph.header)?
    } else if args.payload_only {
        serde_json::to_string_pretty(&glyph.payload)?
    } else {
        match args.format.as_str() {
            "json" => glyph.to_json()?,
            "json5" => glyph.to_json5()?,
            "yaml" => glyph.to_yaml()?,
            "text" => glyph.to_text()?,
            _ => {
                eprintln!("Unsupported format: {}. Using JSON.", args.format);
                glyph.to_json()?
            }
        }
    };
    
    // Write output
    if let Some(output_path) = args.output {
        fs::write(output_path, output_content)?;
        println!("✓ Decoded glyph written to output file");
    } else {
        println!("{}", output_content);
    }
    
    Ok(())
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

fn print_usage_examples() {
    println!("Usage examples:");
    println!("  glyph-decode input.glyph                    # Decode to JSON");
    println!("  glyph-decode input.glyph -f yaml            # Decode to YAML");
    println!("  glyph-decode input.glyph -f text            # Decode to human-readable text");
    println!("  glyph-decode input.glyph --extract payload.label  # Extract specific field");
    println!("  glyph-decode input.glyph --header-only      # Show only header");
    println!("  glyph-decode input.glyph --verify --public-key key.pub  # Verify signature");
    println!("  glyph-decode input.glyph --validate         # Validate structure");
    println!("  glyph-decode input.glyph -v                 # Verbose output");
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_glyph_decoding() {
        // Create a test glyph
        let test_glyph = Glyph {
            header: GlyphHeader {
                magic: "GLYPH".to_string(),
                version: "0.1".to_string(),
                id: Uuid::new_v4().to_string(),
                hash: "blake3:test123".to_string(),
                signature: None,
                created_at: Utc::now(),
            },
            payload: GlyphPayload {
                label: "test".to_string(),
                aliases: vec!["test_alias".to_string()],
                classes: vec!["test_class".to_string()],
                namespace: Namespace {
                    provision: "glyph:test".to_string(),
                    contextual_overlays: vec![],
                },
                origin: None,
                truth_mode: TruthMode {
                    r#type: "empirical".to_string(),
                    confidence: 0.95,
                    verified_by: vec!["agent:test".to_string()],
                    conflicts: vec![],
                },
                audio: Audio {
                    file: None,
                    phonetic: None,
                },
                visual: Visual {
                    svg: None,
                    sigil: None,
                    style: None,
                },
                relations: vec![],
                grounding: Grounding {
                    wikidata: None,
                    sensors: vec![],
                },
            },
        };
        
        // Encode to CBOR
        let cbor_data = test_glyph.to_cbor().unwrap();
        
        // Decode from CBOR
        let decoded_glyph = Glyph::from_cbor(&cbor_data).unwrap();
        
        // Verify
        assert_eq!(decoded_glyph.header.magic, "GLYPH");
        assert_eq!(decoded_glyph.payload.label, "test");
    }
    
    #[test]
    fn test_field_extraction() {
        let test_glyph = Glyph {
            header: GlyphHeader {
                magic: "GLYPH".to_string(),
                version: "0.1".to_string(),
                id: "test-id".to_string(),
                hash: "blake3:test".to_string(),
                signature: None,
                created_at: Utc::now(),
            },
            payload: GlyphPayload {
                label: "test".to_string(),
                aliases: vec![],
                classes: vec![],
                namespace: Namespace {
                    provision: "glyph:test".to_string(),
                    contextual_overlays: vec![],
                },
                origin: None,
                truth_mode: TruthMode {
                    r#type: "empirical".to_string(),
                    confidence: 0.95,
                    verified_by: vec![],
                    conflicts: vec![],
                },
                audio: Audio {
                    file: None,
                    phonetic: None,
                },
                visual: Visual {
                    svg: None,
                    sigil: None,
                    style: None,
                },
                relations: vec![],
                grounding: Grounding {
                    wikidata: None,
                    sensors: vec![],
                },
            },
        };
        
        assert_eq!(test_glyph.extract_field("payload.label").unwrap(), "test");
        assert_eq!(test_glyph.extract_field("header.magic").unwrap(), "GLYPH");
        assert!(test_glyph.extract_field("unknown.field").is_err());
    }
}
