// 1. Key Features of the Implementation:
// 2. Complete Data Structures: All fields from the spec are properly typed
// 3. JSON5 Parsing: Uses serde_json5 for flexible JSON parsing
// 4. CBOR Encoding: Binary output using cbor4ii
// 5. Cryptographic Signing: Ed25519 signature support
// 6. Validation: Comprehensive validation of glyph structure
// 7. CLI Interface: User-friendly command-line interface
// 8. Error Handling: Robust error handling throughout
// 9. Testing: Unit tests for core functionality

use serde::{Deserialize, Serialize};
use serde_json5;
use ciborium::{into_writer, from_reader};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use blake3::Hasher;
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use clap::Parser;
use hex;
use qrcode::QrCode;
use image::{Luma, ImageBuffer};
use base64;

// ============================================================================
// DATA STRUCTURES
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
#[command(name = "glyph-encode")]
#[command(about = "Encode JSON5 glyph files to binary .glyph format")]
struct Args {
    /// Input JSON5 file
    #[arg(short, long)]
    input: String,
    
    /// Output .glyph file
    #[arg(short, long)]
    output: Option<String>,
    
    /// Generate QR code from glyph
    #[arg(long)]
    qr: bool,
    
    /// QR code output file
    #[arg(long)]
    qr_output: Option<String>,
    
    /// Sign the glyph with Ed25519
    #[arg(short, long)]
    sign: bool,
    
    /// Private key file for signing
    #[arg(long)]
    private_key: Option<String>,
    
    /// Generate UUID for glyph
    #[arg(long)]
    generate_id: bool,
    
    /// Validate glyph structure
    #[arg(long)]
    validate: bool,
}

// ============================================================================
// MAIN ENCODER LOGIC
// ============================================================================

impl Glyph {
    pub fn to_qr_code(&self, output_path: Option<&str>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Encode glyph to CBOR
        let cbor_data = self.to_cbor()?;
        
        // Convert CBOR to base64 for QR code (more reliable than raw bytes)
        let base64_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &cbor_data);
        
        // Convert to QR code
        let code = QrCode::new(base64_data.as_bytes())?;

        // Use a simpler approach - render to string first, then convert to image
        let qr_string = code.render()
            .light_color(' ')
            .dark_color('#')
            .build();
        
        // Parse the string representation to create an image
        let lines: Vec<&str> = qr_string.lines().collect();
        let height = lines.len() as u32;
        let width = if height > 0 { lines[0].len() as u32 } else { 0 };
        
        // Create image buffer
        let mut image_buffer = ImageBuffer::new(width, height);
        
        // Fill the image based on the string representation
        for (y, line) in lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let pixel = if ch == '#' {
                    Luma([0u8]) // Black
                } else {
                    Luma([255u8]) // White
                };
                image_buffer.put_pixel(x as u32, y as u32, pixel);
            }
        }

        // Convert to PNG bytes
        let mut png_data = Vec::new();
        image_buffer.write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png)?;

        // Save to file if path provided
        if let Some(path) = output_path {
            fs::write(path, &png_data)?;
        }

        Ok(png_data)
    }

    pub fn from_json5(json5_content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Parse JSON5 content
        let payload: GlyphPayload = serde_json5::from_str(json5_content)?;
        
        // Generate header
        let header = Self::generate_header(&payload)?;
        
        Ok(Glyph { header, payload })
    }
    
    fn generate_header(payload: &GlyphPayload) -> Result<GlyphHeader, Box<dyn std::error::Error>> {
        // Generate semantic hash
        let semantic_content = Self::extract_semantic_content(payload);
        let hash = Self::compute_hash(&semantic_content);
        
        // Generate UUID if not provided
        let id = Uuid::new_v4().to_string();
        
        Ok(GlyphHeader {
            magic: "GLYPH".to_string(),
            version: "0.1".to_string(),
            id,
            hash,
            signature: None, // Will be set later if signing
            created_at: Utc::now(),
        })
    }
    
    fn extract_semantic_content(payload: &GlyphPayload) -> String {
        // Extract core semantic fields for hashing
        format!(
            "{}|{}|{}|{}|{}",
            payload.label,
            payload.aliases.join(","),
            payload.classes.join(","),
            payload.truth_mode.r#type,
            payload.grounding.wikidata.as_deref().unwrap_or("")
        )
    }
    
    fn compute_hash(content: &str) -> String {
        let mut hasher = Hasher::new();
        hasher.update(content.as_bytes());
        format!("blake3:{}", hasher.finalize().to_hex())
    }
    
    pub fn sign(&mut self, private_key: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        // Convert private key to the expected format
        if private_key.len() != 32 {
            return Err("Private key must be exactly 32 bytes".into());
        }
        
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(private_key);
        
        let signing_key = SigningKey::from_bytes(&key_bytes);
        let message = self.header.hash.as_bytes();
        let signature = signing_key.sign(message);
        
        self.header.signature = Some(format!("ed25519:{}", hex::encode(signature.to_bytes())));
        Ok(())
    }
    
    pub fn to_cbor(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buffer = Vec::new();
        into_writer(self, &mut buffer)?;
        Ok(buffer)
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
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// ============================================================================
// MAIN FUNCTION
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Read input file
    let json5_content = fs::read_to_string(&args.input)?;
    
    // Parse and create glyph
    let mut glyph = Glyph::from_json5(&json5_content)?;
    
    // Generate ID if requested
    if args.generate_id {
        glyph.header.id = Uuid::new_v4().to_string();
    }
    
    // Sign if requested
    if args.sign {
        if let Some(key_path) = args.private_key {
            let private_key = fs::read(key_path)?;
            glyph.sign(&private_key)?;
        } else {
            eprintln!("Warning: --sign specified but no --private-key provided");
        }
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
                std::process::exit(1);
            }
        }
    }
    
    // Encode to CBOR
    let cbor_data = glyph.to_cbor()?;
    
    // Determine output path
    let output_path = if let Some(output) = args.output {
        output
    } else {
        let input_path = std::path::Path::new(&args.input);
        let stem = input_path.file_stem().unwrap().to_str().unwrap();
        format!("{}.glyph", stem)
    };
    
    // Write CBOR output
    fs::write(&output_path, cbor_data)?;
    println!("✓ Encoded glyph written to: {}", output_path);
    
    // Generate QR code if requested
    if args.qr {
        let qr_output_path = args.qr_output.unwrap_or_else(|| {
            let input_path = std::path::Path::new(&args.input);
            let stem = input_path.file_stem().unwrap().to_str().unwrap();
            format!("{}.png", stem)
        });
        
        let _qr_data = glyph.to_qr_code(Some(&qr_output_path))?;
        println!("✓ QR code written to: {}", qr_output_path);
    }
    
    // Also write JSON companion
    let json_companion = format!("{}.json", output_path);
    let json_content = serde_json::to_string_pretty(&glyph)?;
    fs::write(json_companion, json_content)?;
    println!("✓ JSON companion written to: {}.json", output_path);
    
    Ok(())
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_glyph_creation() {
        let json5_content = r#"
        {
            "label": "test",
            "aliases": ["test_alias"],
            "classes": ["test_class"],
            "namespace": {
                "provision": "glyph:test",
                "contextual_overlays": []
            },
            "truth_mode": {
                "type": "empirical",
                "confidence": 0.95,
                "verified_by": ["agent:test"],
                "conflicts": []
            },
            "audio": {
                "file": null,
                "phonetic": null
            },
            "visual": {
                "svg": null,
                "sigil": null,
                "style": null
            },
            "relations": [],
            "grounding": {
                "wikidata": null,
                "sensors": []
            }
        }
        "#;
        
        let glyph = Glyph::from_json5(json5_content).unwrap();
        assert_eq!(glyph.header.magic, "GLYPH");
        assert_eq!(glyph.payload.label, "test");
    }
}
