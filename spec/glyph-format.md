# Glyph Format Specification v0.1

The `.glyph` file format defines a compact, self-describing, multimodal concept unit for symbolic AI cognition.

---

## 🧾 Header

| Field        | Type       | Description                   |
|--------------|------------|-------------------------------|
| `magic`      | `string`   | Literal: `"GLYPH"`            |
| `version`    | `string`   | Format version (e.g. `"0.1"`) |
| `id`         | `uuid`     | Unique glyph identifier       |
| `hash`       | `string`   | Content hash (e.g. blake3)    |
| `signature`  | `string?`  | Optional Ed25519 signature    |
| `created_at` | `datetime` | RFC3339 UTC                   |

---

## 🔤 Core Payload

### Semantic Block
```json5
{
    "label": "sun",
    "aliases": ["star", "solar core"],
    "classes": ["astronomy", "energy_source"],
    "namespace": {                                       // Perspective & Namespacing
        "provision": "glyph:core",
        "contextual_overlays": [
            { "agent": "glyphos:mythology", "label": "Ra", "truth_mode": "mythic" }
        ],
    },
    "truth_mode": {
        "type": "empirical",
        "confidence": 0.98,
        "verified_by": ["agent:NASA", "agent:ESA"]
    },
    "audio": {                                           // Audio Block
        "file": "sun.wav",
        "phonetic": "sʌn"
    },
    "visual": {                                      
        "svg": "<svg>...</svg>",
        "sigil": "base64",                               // stylized QR-like glyph
        "style": {
            "color": "#FFD700",
            "stroke": 1.2
        }
    },
    "relations": [                                       // Relation Block
        { "type": "opposite_of", "target": "glyph:moon" },
        { "type": "powers", "target": "glyph:photosynthesis" }
    ],
    "grounding": {                                       // Grounding Block
        "wikidata": "Q525",
        "sensors": ["lidar:12", "img:3421.jpg"]
    },
}
```

### Glyph 📦 Encoding
	•	Encoded in CBOR by default
	•	Optional *.glyph.json human-readable companion
	•	Signed .glyph.sig companion file supported

### Glyph 🧪 Verification & Validation
	•	Semantic hash used to verify content
	•	Signature tied to agent identity
	•	glyph-validator lints:
	•	structure
	•	truth coherence
	•	logical loops
	•	ungrounded references


