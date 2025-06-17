# Basic encoding
cargo run -- encode --input ./test_glyph.json5

# Encoding with custom output name
#cargo run -- encode test_glyph.json5 --output my_glyph.glyph

# Encoding with validation
#cargo run -- encode test_glyph.json5 --validate

# Encoding with ID generation
#cargo run -- encode test_glyph.json5 --generate-id

# Encoding with signing (if you have a private key)
#cargo run -- encode test_glyph.json5 --sign --private-key private_key.pem
