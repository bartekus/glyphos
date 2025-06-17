# Decode to JSON
cargo run -- decode --input ./test_glyph.glyph

# Decode to YAML
#cargo run -- decode test_glyph.glyph --format yaml

# Decode to human-readable text
#cargo run -- decode test_glyph.glyph --format text

# Decode with verbose output
#cargo run -- decode test_glyph.glyph --verbose

# Extract specific field
#cargo run -- decode test_glyph.glyph --extract payload.label

# Show only header
#cargo run -- decode test_glyph.glyph --header-only
