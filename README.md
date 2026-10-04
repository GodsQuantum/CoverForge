# CoverForge

**AI makes the key art. CoverForge makes it publishable.**

Deterministic Rust compositor + editable show styles for podcast thumbnails and social covers.

[Français](README.fr.md) · [简体中文](README.zh-CN.md)

## Why

Image models are good at scenes and bad at repeated typography, logos and exact branding. CoverForge keeps those jobs separate:

1. generate **native key art for the target ratio**;
2. apply exact typography / logo / palette with a deterministic renderer;
3. edit the entire visual identity of a show from **one JSON file**.

No fake `16:9 → blurred background → inset image` conversion.

## What ships in v0.4

- ⚡ Rust 2024 + Axum + resvg/tiny-skia production renderer
- 🧩 one JSON show-style containing every independent format layout
- 🎨 Svelte 5 / SvelteKit 3 + Fabric.js 7 visual studio with drag / resize / rotate
- 📐 browser-fit canvas plus percentage sliders with exact numeric X/Y/W/H values
- 🧱 named image / rectangle / text layers with visibility, locking, reorder, duplicate and delete
- 🔤 font-family + real face/variant selection (weight and italic carried to Fabric and Rust/resvg)
- ✍️ text boxes with max size, minimum size and automatic fit-to-box
- 🧪 instant browser canvas plus exact unsaved Rust preview
- ↶ undo / redo, zoom, grid and safe-area overlays
- 🔤 searchable font library + custom font upload/folder-drop
- 🔌 API-first JSON model, batch render and legacy automation adapter
- 🎯 normalized 0–1 coordinates for portable layouts
- 💾 atomic template saves
- 🐳 Docker Compose deployment

## Native social formats

A show style can hold all production outputs in the same file:

| Key | Output |
|---|---:|
| `youtube` | 3840×2160 |
| `square` | 1200×1200 |
| `feed` | 1080×1350 |
| `vertical` | 2160×3840 |
| `acast` | 3000×3000 |

A recommended automation pipeline is:

```text
transcript/editorial direction
        ↓
16:9 key art ──→ YouTube layout
1:1 key art  ──→ Square + Acast layouts
4:5 key art  ──→ Feed layout
9:16 key art ──→ Vertical layout
```

CoverForge only crops when you explicitly ask a layer to use `fit: cover`; it does not invent blurred letterboxing.

## One file per show

Example production tree:

```text
Templates/
├── cf.json
├── dpafm.json
├── lcfp.json
└── my-new-show.json
```

Each show JSON contains:

- `brand.display_name`
- `brand.visual_summary`
- `brand.palette`
- `brand.fonts`
- `brand.notes`
- `formats.youtube`
- `formats.square`
- `formats.feed`
- `formats.vertical`
- `formats.acast`

See [docs/SHOW_STYLES.md](docs/SHOW_STYLES.md).

## Fonts

CoverForge ships with a broad libre font set and exposes a searchable font browser.

Custom fonts can be added either:

- from the UI;
- by dropping `.ttf`, `.otf`, `.ttc` or `.otc` files into the persistent Custom folder.

The public repo does **not** contain private/commercial font binaries.

See [docs/FONTS.md](docs/FONTS.md).

## API

```text
GET    /health
GET    /openapi.json
GET    /v1/templates
GET    /v1/templates/{show}
PUT    /v1/templates/{show}
GET    /v1/templates/{show}/dataset
GET    /v1/fonts
POST   /v1/fonts
DELETE /v1/fonts/{filename}
GET    /v1/asset?path=...
POST   /v1/render
POST   /v1/render/preview
POST   /v1/render/batch
POST   /api/generate
```

### Render one format

```json
{
  "template": "dpafm",
  "variables": {
    "title": "LE STAND-UP EST MORT",
    "eyebrow": "DPAFM #48",
    "background": "/srv/storage/path/native-16x9.png",
    "logo": "/srv/storage/path/DPAFM.png"
  },
  "variants": ["youtube"]
}
```

### Legacy automation adapter

Existing calls such as `DPAFM - YT` still work. They are resolved to show `dpafm`, format `youtube`.

## Docker

```sh
docker compose up -d --build
curl -fsS http://127.0.0.1:3099/health
```

The provided Compose file is intentionally Cloud9-flavoured; change the bind mounts for another machine.

## Development

```sh
cargo test --locked

cd web
npm ci
npm run check
npm run build
```

## No-BS rules

- key art should be generated for the **actual target ratio**;
- typography belongs to the compositor, not the image model;
- one show = one style file;
- private fonts and production assets stay outside Git;
- a template edit must not require an app rebuild.

## License

MIT
