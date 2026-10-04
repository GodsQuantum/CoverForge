<p align="center">
  <img src="brand/coverforge-logo.svg" alt="CoverForge" width="520">
</p>

# CoverForge

**One image, every format. API-first branding automation, self-hosted.**

[Français](README.fr.md) · [简体中文](README.zh-CN.md)

CoverForge turns one source image plus a brand identity into consistent assets for multiple platforms. It combines a browser studio, reusable JSON templates, deterministic Rust rendering, Smart Reframe and batch ZIP export.

![CoverForge workflow](docs/assets/coverforge-workflow.svg)

```text
Image → Smart crop → Brand kit → Dynamic layers → Multi-format → ZIP / API
```

## What CoverForge does

- import JPEG, PNG, WebP and safe SVG logos;
- automatically reframe one raster image for multiple aspect ratios;
- apply logos, palette, fonts, titles, subtitles and badges;
- edit each format independently with Fabric.js;
- render the final result with Rust + resvg/tiny-skia;
- export selected outputs individually or as a ZIP with `manifest.json`;
- expose the same workflow through a JSON/OpenAPI API;
- keep existing legacy AutoPublisher calls working.

The generic built-in `starter-brand` template includes:

| Format | Size |
| --- | ---: |
| YouTube / 16:9 | 1920×1080 |
| Square / 1:1 | 1080×1080 |
| Feed / 4:5 | 1080×1350 |
| Vertical / 9:16 | 1080×1920 |
| Landscape / 1.91:1 | 1200×628 |

## Product model

CoverForge keeps three responsibilities separate:

1. **Source asset** — the image you want to publish.
2. **Brand template** — deterministic layout, typography, logos, palette and variables.
3. **Output package** — one or many rendered formats.

The canonical design document is JSON. Fabric.js is only the interactive editing surface; Rust/resvg remains the production renderer.

## Quick start

Build and run locally:

```bash
docker build -t coverforge .
docker run --rm -p 3099:3099 \
  -e COVERFORGE_BIND=0.0.0.0:3099 \
  -e COVERFORGE_DEFAULT_TEMPLATE_DIR=/app/default-templates \
  coverforge
```

Then open `http://localhost:3099`.

The included `compose.yaml` is intentionally tailored to the existing Cloud9 deployment. Adapt its bind mounts before using it on another host.

## API

Core routes:

```text
GET    /health
GET    /openapi.json

GET    /v1/templates
GET    /v1/templates/{id}
PUT    /v1/templates/{id}
GET    /v1/templates/{id}/dataset

GET    /v1/fonts
POST   /v1/fonts
DELETE /v1/fonts/{name}

GET    /v1/asset?path=...
POST   /v1/assets
POST   /v1/reframe

POST   /v1/render
POST   /v1/render/preview
POST   /v1/render/batch
POST   /v1/render/package

POST   /api/generate
```

Full examples: [docs/API.md](docs/API.md).

## Security

CoverForge is designed for self-hosted use.

- uploaded rasters are format-sniffed and dimension-limited;
- SVG logos reject scripts, event handlers and external references;
- file access is restricted to configured safe roots;
- uploaded assets become immediately usable without widening arbitrary filesystem access;
- Smart Reframe is local and deterministic;
- ZIP entry names are sanitized to prevent traversal;
- private fonts and production assets stay outside the public repository.

Before exposing an instance publicly, add authentication/reverse-proxy access controls appropriate to your deployment.

## Compatibility

Existing production templates and the legacy `POST /api/generate` adapter remain supported. Podcast/show workflows are one use case of the generic branding engine, not the product identity.

## Development

```bash
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings

cd web
npm ci
npm run check
npm test
npm run build
```

## Documentation

- [Guide français](README.fr.md)
- [简体中文说明](README.zh-CN.md)
- [Template model](docs/SHOW_STYLES.md)
- [API reference and examples](docs/API.md)
- `GET /openapi.json` on a running instance

## License

MIT
