#[macro_use]
extern crate log;

pub mod commands;
pub mod error;
pub mod hazard;

pub mod glyph_encode;
pub mod glyph_decode;

use utils::error::Result;

pub fn start() -> Result<()> {
    // does nothing

    Ok(())
}
