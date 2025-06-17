use std::path::PathBuf;
use clap::{Parser, Subcommand, CommandFactory};
use clap_complete::{generate, shells::{Bash, Fish, Zsh}};

use core::commands;
use utils::app_config::AppConfig;
use utils::error::Result;
use utils::types::LogLevel;


#[derive(Parser, Debug)]
#[command(
    name = "glyphos",
    author,
    about,
    long_about = "GlyphOS",
    version
)]
//TODO: #[clap(setting = AppSettings::SubcommandRequired)]
//TODO: #[clap(global_setting(AppSettings::DeriveDisplayOrder))]
pub struct Cli {
    /// Set a custom config file
    /// TODO: parse(from_os_str)
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Set a custom config file
    #[arg(name="debug", short, long="debug", value_name = "DEBUG")]
    pub debug: Option<bool>,

    /// Set Log Level 
    #[arg(name="log_level", short, long="log-level", value_name = "LOG_LEVEL")]
    pub log_level: Option<LogLevel>,

    /// Subcommands
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[clap(
        name = "hazard",
        about = "Generate a hazardous occurance",
        long_about = None, 
    )]
    Hazard,
    #[clap(
        name = "error",
        about = "Simulate an error",
        long_about = None, 
    )]
    Error,
    #[clap(
        name = "encode",
        about = "Encode JSON5 glyph files to binary .glyph format",
        long_about = "Convert human-readable JSON5 glyph files to compact binary .glyph format with optional signing",
    )]
    Encode {
        /// Input JSON5 file
        #[arg(short, long)]
        input: String,
        
        /// Output .glyph file
        #[arg(short, long)]
        output: Option<String>,
        
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
    },
    #[clap(
        name = "decode",
        about = "Decode binary .glyph files to human-readable formats",
        long_about = "Convert binary .glyph files to JSON, YAML, or human-readable text with optional signature verification",
    )]
    Decode {
        /// Input .glyph file or QR code image
        #[arg(short, long)]
        input: String,
        
        /// Input is a QR code image
        #[arg(long)]
        qr: bool,
        
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
        #[arg(short = 'V', long)]
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
    },
    #[clap(
        name = "completion",
        about = "Generate completion scripts",
        long_about = None,
        )]
        Completion {
            #[clap(subcommand)]
            subcommand: CompletionSubcommand,
        },
    #[clap(
        name = "config",
        about = "Show Configuration",
        long_about = None,
    )]
    Config,
}

#[derive(Subcommand, PartialEq, Debug)]
enum CompletionSubcommand {
    #[clap(about = "generate the autocompletion script for bash")]
    Bash,
    #[clap(about = "generate the autocompletion script for zsh")]
    Zsh,
    #[clap(about = "generate the autocompletion script for fish")]
    Fish,
}

pub fn cli_match() -> Result<()> {
    // Parse the command line arguments
    let cli = Cli::parse();

    // Merge clap config file if the value is set
    AppConfig::merge_config(cli.config.as_deref())?;

    let app = Cli::command();
    let matches = app.get_matches();
    
    AppConfig::merge_args(matches)?;

    // Execute the subcommand
    match &cli.command {
        Commands::Hazard => commands::hazard()?,
        Commands::Error => commands::simulate_error()?,
        Commands::Encode { input, output, sign, private_key, generate_id, validate } => {
            commands::encode(input, output.as_deref(), *sign, private_key.as_deref(), *generate_id, *validate)?
        },
        Commands::Decode { input, qr, format, output, verify, public_key, verbose, validate, extract, header_only, payload_only } => {
            commands::decode(
                input, 
                *qr, 
                format, 
                output.as_deref(), 
                *verify, 
                public_key.as_deref(), 
                *verbose, 
                *validate, 
                extract.as_deref(), 
                *header_only, 
                *payload_only
            )?
        },
        Commands::Completion {subcommand} => {
            let mut app = Cli::command();
            match subcommand {
                CompletionSubcommand::Bash => {
                    generate(Bash, &mut app, "glyphos", &mut std::io::stdout());
                }
                CompletionSubcommand::Zsh => {
                    generate(Zsh, &mut app, "glyphos", &mut std::io::stdout());
                }
                CompletionSubcommand::Fish => {
                    generate(Fish, &mut app, "glyphos", &mut std::io::stdout());
                }
            }
        }
        Commands::Config => commands::config()?,
    }

    Ok(())
}
