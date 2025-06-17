use super::error;
use super::hazard;
use super::glyph_encode;
use super::glyph_decode;

use utils::app_config::AppConfig;
use utils::error::Result;
use std::fs;

/// Show the configuration file
pub fn hazard() -> Result<()> {
    // Generate, randomly, True or False
    let random_hazard: bool = hazard::generate_hazard()?;

    if random_hazard {
        println!("You got it right!");
    } else {
        println!("You got it wrong!");
    }

    Ok(())
}

/// Show the configuration file
pub fn config() -> Result<()> {
    let config = AppConfig::fetch()?;
    println!("{:#?}", config);

    Ok(())
}

/// Simulate an error
pub fn simulate_error() -> Result<()> {
    // Log this Error simulation
    info!("We are simulating an error");

    // Simulate an error
    error::simulate_error()?;

    // We should never get here...
    Ok(())
}

/// Encode JSON5 glyph files to binary .glyph format
pub fn encode(
    input: &str,
    output: Option<&str>,
    sign: bool,
    private_key: Option<&str>,
    generate_id: bool,
    validate: bool,
) -> Result<()> {
    // Read input file
    let json5_content = fs::read_to_string(input)?;
    
    // Parse and create glyph
    let mut glyph = glyph_encode::Glyph::from_json5(&json5_content)
        .map_err(|e| utils::error::Error::new(&e.to_string()))?;
    
    // Generate ID if requested
    if generate_id {
        glyph.header.id = uuid::Uuid::new_v4().to_string();
    }
    
    // Sign if requested
    if sign {
        if let Some(key_path) = private_key {
            let private_key_bytes = fs::read(key_path)?;
            glyph.sign(&private_key_bytes)
                .map_err(|e| utils::error::Error::new(&e.to_string()))?;
        } else {
            eprintln!("Warning: --sign specified but no --private-key provided");
        }
    }
    
    // Validate if requested
    if validate {
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
    let cbor_data = glyph.to_cbor()
        .map_err(|e| utils::error::Error::new(&e.to_string()))?;
    
    // Determine output path
    let output_path = if let Some(output) = output {
        output.to_string()
    } else {
        let input_path = std::path::Path::new(input);
        let stem = input_path.file_stem().unwrap().to_str().unwrap();
        format!("{}.glyph", stem)
    };
    
    // Write output
    fs::write(&output_path, cbor_data)?;
    println!("✓ Encoded glyph written to: {}", output_path);
    
    // Also write JSON companion
    let json_companion = format!("{}.json", output_path);
    let json_content = serde_json::to_string_pretty(&glyph)?;
    fs::write(json_companion, json_content)?;
    println!("✓ JSON companion written to: {}.json", output_path);
    
    Ok(())
}

/// Decode binary .glyph files to human-readable formats
pub fn decode(
    input: &str,
    qr: bool,
    format: &str,
    output: Option<&str>,
    verify: bool,
    public_key: Option<&str>,
    verbose: bool,
    validate: bool,
    extract: Option<&str>,
    header_only: bool,
    payload_only: bool,
) -> Result<()> {
    // Read input file
    let input_data = fs::read(input)?;
    
    // Decode glyph based on input type
    let glyph = if qr {
        glyph_decode::Glyph::from_qr_code(&input_data)
            .map_err(|e| utils::error::Error::new(&e.to_string()))?
    } else {
        glyph_decode::Glyph::from_cbor(&input_data)
            .map_err(|e| utils::error::Error::new(&e.to_string()))?
    };
    
    // Show info if verbose
    if verbose {
        println!("{}", glyph.get_info());
        println!();
    }
    
    // Validate if requested
    if validate {
        match glyph.validate() {
            Ok(()) => println!("✓ Glyph validation passed"),
            Err(errors) => {
                eprintln!("✗ Glyph validation failed:");
                for error in errors {
                    eprintln!("  - {}", error);
                }
                if !verbose {
                    std::process::exit(1);
                }
            }
        }
    }
    
    // Verify signature if requested
    if verify {
        if let Some(key_path) = public_key {
            let public_key_bytes = fs::read(key_path)?;
            match glyph.verify_signature(&public_key_bytes)
                .map_err(|e| utils::error::Error::new(&e.to_string())) {
                Ok(true) => println!("✓ Signature verification passed"),
                Ok(false) => {
                    if glyph.header.signature.is_some() {
                        eprintln!("✗ Signature verification failed");
                        if !verbose {
                            std::process::exit(1);
                        }
                    } else {
                        println!("ℹ No signature to verify");
                    }
                }
                Err(e) => {
                    eprintln!("✗ Signature verification error: {}", e);
                    if !verbose {
                        std::process::exit(1);
                    }
                }
            }
        } else {
            eprintln!("Warning: --verify specified but no --public-key provided");
        }
    }
    
    // Extract specific field if requested
    if let Some(field_path) = extract {
        let value = glyph.extract_field(field_path)
            .map_err(|e| utils::error::Error::new(&e.to_string()))?;
        println!("{}", value);
        return Ok(());
    }
    
    // Determine output content
    let output_content = if header_only {
        serde_json::to_string_pretty(&glyph.header)?
    } else if payload_only {
        serde_json::to_string_pretty(&glyph.payload)?
    } else {
        match format {
            "json" => glyph.to_json()
                .map_err(|e| utils::error::Error::new(&e.to_string()))?,
            "json5" => glyph.to_json5()
                .map_err(|e| utils::error::Error::new(&e.to_string()))?,
            "yaml" => glyph.to_yaml()
                .map_err(|e| utils::error::Error::new(&e.to_string()))?,
            "text" => glyph.to_text()
                .map_err(|e| utils::error::Error::new(&e.to_string()))?,
            _ => {
                eprintln!("Unsupported format: {}. Using JSON.", format);
                glyph.to_json()
                    .map_err(|e| utils::error::Error::new(&e.to_string()))?
            }
        }
    };
    
    // Write output
    if let Some(output_path) = output {
        fs::write(output_path, output_content)?;
        println!("✓ Decoded glyph written to output file");
    } else {
        println!("{}", output_content);
    }
    
    Ok(())
}
