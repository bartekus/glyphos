# GlyphOS

A Glyph-based General Cognition Project
Language of thought (glyphē + logos)

This repository serves as the foundation for developing a symbolic system based on glyphs,
combining cognitive science, epistemology, and computational tools.

## 📦 Glyph Specification

- [spec/glyph-format.md](spec/glyph-format.md) — Detailed .glyph file format specification (JSON + binary)

## 📚 Epistemology

- [docs/epistemology.md](docs/epistemology.md)

## 📖 Glossary

- [docs/glossary.md](docs/glossary.md) — Canonical term definitions for the glyph system

## 🌐 Symbolic Infrastructure

- [docs/infrastructure.md](docs/infrastructure.md)

## 🧪 Core Functionality

- [core/src/glyph_encode.rs](core/src/glyph_encode.rs) — JSON5 → .glyph (CBOR) -> QRCode 
- [core/src/glyph_decode.rs](core/src/glyph_decode.rs) — QRCode -> .glyph (CBOR) → JSON5
- [core/src/glyph_sign.rs](core/src/glyph_sign.rs) — Sign with Ed25519 (WIP)
- [core/src/glyph_validate.rs](core/src/glyph_validate.rs) — Structural & epistemic checks (WIP)

## 🧪 Examples
- [examples/sun.glyph.json](examples/sun.glyph.json)
- [examples/birth.glyph.json](examples/birth.glyph.json)
- [examples/tree.glyph.json](examples/tree.glyph.json)

## 🧠 Whitepaper & Outline

- [whitepaper/glyph-whitepaper.md](whitepaper/glyph-whitepaper.md) — Markdown draft
- [whitepaper/outline.md](whitepaper/outline.md) — Whitepaper outline for iteration

---

## 📍 Project Milestones

- [x] Define .glyph spec
- [x] Create initial glyph examples
- [x] Draft whitepaper outline
- [x] Finalize symbolic infrastructure design
- [ ] Implement full glyph validator and signature tooling
- [ ] Train symbolic model on glyph dataset
