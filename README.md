# CoverForge

Deterministic, API-first image compositing for social media.

AI image models are good at key art and bad at exact typography, repeated logos
and deterministic cross-format branding. CoverForge deliberately does **not**
generate the artwork: it turns already-native key art into final branded assets.

## Stack

- Rust 2024 + Axum
- resvg/tiny-skia deterministic rasterization
- Svelte 5 static template editor
- JSON scene graph templates
- Docker Compose deployment

## Current design

AUTOPUBLISHER generates a distinct key art for each target ratio:

- YouTube 16:9
- Square 1:1
- Instagram feed 4:5
- TikTok / Story 9:16
- Acast reuses the square art with a podcast-specific template

CoverForge then applies the corresponding template. There is no
`16:9 -> blurred background -> inset image` conversion.

Production templates are mutable data and are intentionally kept outside the
public repository. The container reads them from `COVERFORGE_TEMPLATE_DIR`.
The web UI can inspect, edit and atomically save them.

## API

- `GET /health`
- `GET /v1/templates`
- `GET /v1/templates/{id}`
- `PUT /v1/templates/{id}`
- `POST /v1/render`
- `POST /v1/render/batch`
- `POST /api/generate` legacy adapter used by AUTOPUBLISHER

### Render

```json
{
  "template": "example-social",
  "variables": {
    "title": "LE STAND-UP EST MORT",
    "eyebrow": "DPAFM #48",
    "background": "/srv/storage/path/keyart.png",
    "logo": "/srv/storage/path/logo.png"
  },
  "variants": []
}
```

### Legacy adapter

```json
{
  "template_name": "DPAFM - YT",
  "output_filename": "dpafm48-youtube.png",
  "bg_image": "/srv/storage/path/native-16x9.png",
  "fields": {
    "title_copy": "LE STAND-UP EST MORT",
    "eyebrow": "DPAFM #48",
    "PodLogo": "/srv/storage/path/DPAFM.png"
  }
}
```

`DPAFM - YT` resolves to the production template `dpafm-yt.json`.

## Template model

Coordinates are normalized from `0.0` to `1.0`, so a template remains easy to
reason about regardless of pixel dimensions. Supported layers: image, rectangle
and text. Fonts are loaded at runtime; font binaries are not stored in this repo.

## Docker

```sh
docker compose up -d --build
curl -fsS http://127.0.0.1:3099/health
```

The provided compose file is tailored for the Cloud9 IA Studio deployment.
Adjust bind mounts for another machine.

## Development

```sh
cargo test --locked
cd web
npm ci
npm run check
npm run build
```

## Privacy

Private assets, production templates, generated media, machine-specific secrets
and fonts stay outside the repository or under ignored `local/` / `private/`.

## License

MIT
