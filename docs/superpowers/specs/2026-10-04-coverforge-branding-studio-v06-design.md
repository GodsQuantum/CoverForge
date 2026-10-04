# CoverForge v0.6 — Branding Automation Studio

Date: 2026-10-04  
Status: design approved in chat; written spec pending user review  
Repository: `GodsQuantum/CoverForge`

## 1. Product intent

CoverForge v0.6 is no longer presented as a podcast/show cover compositor. It becomes a general-purpose, self-hosted, API-first **branding automation studio**.

Primary promise:

> **Une image, tous les formats. / 一张图，多种格式**  
> **Branding automatisé par API / 通过 API 驱动的品牌自动化**

The product must be useful to creators, brands, agencies, e-commerce teams, social-media teams, advertisers, marketing departments, and automation workflows.

The existing podcast/show use case remains supported as one specialization, not as the product identity.

## 2. Non-goals

v0.6 does not add:

- user accounts;
- billing;
- subscription tiers;
- authentication beyond the deployment perimeter;
- hosted SaaS infrastructure;
- mandatory external AI APIs;
- cloud dependency for cropping or rendering.

CoverForge remains self-hosted and automation-first.

## 3. Branding system

### 3.1 Logo

Use the approved CoverForge logo direction:

- stacked image cards inside crop/focus brackets;
- orange-to-deep-orange gradient;
- CoverForge wordmark with dark `Cover` and orange `Forge`;
- clean, modern, creator/brand-tool positioning.

Required assets:

- `brand/coverforge-logo.svg` — full horizontal wordmark;
- `brand/coverforge-mark.svg` — icon only;
- `web/static/favicon.svg`;
- `web/static/coverforge-logo.svg`.

Raster-only generated references may be used as visual guidance, but the shipped repository asset must be a clean vector recreation without embedding model-generated raster pixels.

### 3.2 Color palette

Canonical palette:

- Forge Orange: `#FF7A00`;
- Deep Orange: `#FF4D00`;
- Amber: `#FFC64D`;
- Cream: `#FFF8F1`;
- Charcoal: `#111827`.

Supporting neutrals may be derived from Cream and Charcoal for borders, muted text, panels and hover states.

The editor must move away from the current green/black visual identity and use the orange/cream/charcoal system consistently.

### 3.3 Typography

Default product UI font should be Inter or another libre system-compatible sans with similar metrics.

Do not bundle proprietary fonts.

## 4. Internationalization

The UI and documentation are bilingual:

- French;
- Simplified Chinese.

The visible UI should show both languages for primary navigation and action labels where space permits, for example:

- Projet / 项目
- Créer / 创建
- Modèles / 模板
- Bibliothèque / 素材库
- Kit de marque / 品牌工具包
- Exports / 导出
- Importer / 导入
- Recadrage intelligent / 智能裁切
- Calques / 图层
- Exporter tout / 全部导出

Long-form descriptions may use a locale toggle to avoid excessive visual density.

The README must contain complete French and Simplified Chinese sections. English may remain for code/API identifiers and concise developer-facing notes, but the user-facing README must not depend on English for comprehension.

## 5. Core user workflow

### 5.1 Start from an image

A user can:

1. drag and drop a JPEG/PNG/WebP image;
2. choose a file through the UI;
3. replace the source image later without destroying layout or brand settings.

Uploaded files are stored under the existing writable CoverForge output/storage area in a dedicated `uploads/` subtree. No new Cloud9 mount is introduced.

### 5.2 Add brand identity

A project/template can define:

- primary logo;
- optional secondary logo/mark;
- palette;
- font families and faces;
- title;
- subtitle;
- badge/episode/number/CTA;
- graphic overlays;
- optional decorative shapes.

These remain JSON-addressable and API-driven.

### 5.3 Smart Reframe

CoverForge generates a recommended crop for each target format from the same source image.

v0.6 algorithm requirements:

- deterministic;
- local;
- no required external ML/API;
- preserves high-detail / high-contrast regions;
- prefers compositionally useful regions;
- considers a rule-of-thirds bias rather than blindly centering;
- returns focal point and crop box;
- allows a global focal-point override;
- allows a per-format override;
- can be reset to automatic.

The algorithm must be isolated behind a dedicated function/module so a future optional face/saliency ML backend can be added without changing the project/template API.

### 5.4 Preview all outputs together

The Create view must show multiple output formats simultaneously.

Initial built-in presets:

- `youtube` — 16:9;
- `square` — 1:1;
- `feed` — 4:5;
- `vertical` — 9:16;
- `landscape` — 1.91:1.

Existing custom template formats remain supported.

The user can select which formats are enabled for the current export.

Clicking a preview opens/focuses detailed editing for that format.

### 5.5 Export package

Users can:

- download one output;
- export selected outputs;
- export all outputs as a ZIP.

ZIP contents:

- rendered images;
- `manifest.json` describing:
  - template/project id;
  - generated timestamp;
  - source image reference;
  - variables;
  - output format ids;
  - dimensions;
  - filenames;
  - CoverForge version.

## 6. Information architecture

The primary product navigation becomes:

1. **Projet / 项目**
   - source image;
   - variables;
   - multi-format preview;
   - export.

2. **Modèles / 模板**
   - reusable JSON templates;
   - format presets;
   - duplicate template;
   - create from starter.

3. **Bibliothèque / 素材库**
   - uploaded source images;
   - logos;
   - reusable graphic assets.

4. **Kit de marque / 品牌工具包**
   - logos;
   - palette;
   - typography;
   - reusable brand metadata.

5. **Exports / 导出**
   - recent generated packages;
   - individual outputs;
   - ZIP packages.

6. **API / API 接入**
   - OpenAPI endpoint;
   - dataset/autofill fields;
   - example payloads;
   - curl snippets.

The current Fonts and JSON tooling stay available inside the relevant sections, not as the product’s main top-level identity.

## 7. Create screen

Desktop layout:

- left: navigation / project identity;
- main-left: source image with crop/focal overlay;
- main-center: multi-format output grid;
- right: selected layer / format properties.

The screen must adapt to browser width and height without nested scrolling where avoidable.

Important behaviors:

- source image dropzone;
- replace image action;
- focal point indicator;
- smart-crop reset;
- multi-format cards;
- selected-format highlighting;
- per-format overflow menu;
- batch select;
- export-selected;
- export-all;
- layer stack;
- range sliders + exact numeric values;
- font family + real face variant;
- auto-fit;
- undo/redo;
- zoom;
- safe areas;
- grid.

## 8. Data model

The existing Template JSON remains the canonical design document.

### 8.1 Naming

The API keeps the wire name `Template` for backward compatibility in v0.6, but UI language must use **Modèle / 模板** and **Projet / 项目**, never “show style” as the generic product concept.

The README and OpenAPI descriptions must remove podcast-specific language except in backward-compatibility examples.

### 8.2 BrandStyle expansion

`BrandStyle` should support:

- `display_name`;
- `visual_summary`;
- `palette`;
- `fonts`;
- `logos`;
- `notes`.

Logo entries are paths/asset references, never embedded binary data.

### 8.3 Format metadata

Each format may optionally include:

- label;
- category;
- platform;
- aspect-ratio metadata;
- export filename suffix.

Existing JSON without this metadata must continue to deserialize.

### 8.4 Project state

v0.6 does not require a database.

Project state may remain client-side plus template JSON, with uploaded assets stored on disk.

No mandatory persistence subsystem is added unless implementation proves it is necessary.

## 9. API

All existing endpoints remain working.

New endpoints:

### 9.1 Upload asset

`POST /v1/assets`

Multipart upload.

Accepted initial types:

- JPEG;
- PNG;
- WebP;
- SVG for logos only.

Response:

- asset id/path;
- MIME type;
- width;
- height;
- URL for preview.

Uploads must be filename-sanitized and confined to the configured writable upload root.

### 9.2 Smart reframe

`POST /v1/reframe`

Input:

- image asset/path;
- one or more requested formats;
- optional focal override.

Output per format:

- normalized crop box;
- focal point;
- confidence/score metadata if useful.

No file is modified by this endpoint.

### 9.3 Render package

`POST /v1/render/package`

Input:

- template id or inline template;
- variables;
- selected formats;
- optional output stem.

Output:

- standard rendered asset metadata;
- ZIP filename/path/url;
- manifest metadata.

### 9.4 Existing API

Must remain compatible:

- `GET /health`;
- `GET /openapi.json`;
- `GET /v1/templates`;
- `GET /v1/templates/{id}`;
- `PUT /v1/templates/{id}`;
- `GET /v1/templates/{id}/dataset`;
- `GET /v1/fonts`;
- `POST /v1/fonts`;
- `DELETE /v1/fonts/{name}`;
- `GET /v1/asset`;
- `POST /v1/render`;
- `POST /v1/render/preview`;
- `POST /v1/render/batch`;
- `POST /api/generate`.

Legacy AutoPublisher calls remain valid.

## 10. Starter template

Ship a public generic template:

`templates/starter-brand.json`

It must contain:

- source/background image;
- logo;
- title;
- subtitle;
- optional badge;
- decorative overlay;
- independent layouts for:
  - youtube;
  - square;
  - feed;
  - vertical;
  - landscape.

Variables:

- `background`;
- `logo`;
- `title`;
- `subtitle`;
- `badge`.

It must demonstrate:

- auto-fit;
- real font face selection;
- brand palette;
- smart crop compatibility;
- multi-format rendering.

## 11. Rendering and image processing

The production renderer remains Rust + resvg/tiny-skia.

Raster image processing should remain local Rust where practical.

New image processing must:

- preserve EXIF orientation;
- reject unsupported/invalid files;
- enforce a reasonable decoded-pixel limit;
- avoid path traversal;
- avoid unbounded memory use;
- not allow SVG uploads to become arbitrary filesystem/network fetch vectors.

ZIP generation must use deterministic filenames and avoid directory traversal.

## 12. Frontend architecture

The current monolithic `+page.svelte` has grown too large. v0.6 should split responsibilities without a full framework rewrite.

Recommended components/modules:

- `lib/i18n.ts` — FR/ZH strings and locale;
- `lib/api.ts` — typed CoverForge API client;
- `lib/types.ts` — shared frontend interfaces;
- `lib/components/AppShell.svelte`;
- `lib/components/SourceImagePanel.svelte`;
- `lib/components/MultiFormatGrid.svelte`;
- `lib/components/FormatCard.svelte`;
- `lib/components/LayerInspector.svelte`;
- `lib/components/BrandKitPanel.svelte`;
- `lib/components/ExportPanel.svelte`.

Fabric remains an editing surface, not the persistence format.

JSON remains the canonical source of truth.

## 13. README and docs

Rewrite the top-level README around generic branding automation.

Required structure:

1. CoverForge logo / product tagline;
2. complete French product explanation;
3. complete Simplified Chinese product explanation;
4. visual workflow:
   `Image → Smart crop → Brand kit → Dynamic layers → Multi-format → ZIP/API`;
5. screenshots/product mockup;
6. features;
7. quick start;
8. Docker;
9. API examples;
10. template JSON;
11. security model;
12. contributing;
13. MIT license.

Remove generic statements such as:

- “one show = one style file”;
- “podcast thumbnails” as primary identity.

Replace with:

- one brand/project/template can generate many formats;
- podcast workflows are one possible use case.

## 14. Backward compatibility

The following are hard requirements:

- existing production template JSON files load without migration;
- existing template ids remain valid;
- existing `/api/generate` behavior remains valid;
- existing AutoPublisher integration remains valid;
- existing Cloud9 mounts are not changed;
- existing output paths are not changed unless a new package endpoint explicitly writes to its own subdirectory;
- current font catalog behavior remains supported;
- no production template is deleted or renamed.

## 15. Testing strategy

### Rust

Add tests for:

- asset filename sanitization;
- accepted/rejected MIME/extension cases;
- image metadata extraction;
- EXIF orientation handling if supported by selected library;
- decoded-pixel limit;
- smart-crop determinism;
- smart-crop bounds;
- focal override;
- package manifest;
- ZIP contents;
- path traversal rejection;
- backward-compatible template parsing;
- existing legacy adapter.

### Frontend

At minimum:

- Svelte type-check;
- production build;
- unit-testable helpers for:
  - i18n;
  - format selection;
  - API payload generation;
  - crop normalization.

Manual smoke checks:

- drag/drop upload;
- image replace;
- multi-format preview;
- format select/unselect;
- smart crop;
- manual focal override;
- layer editing;
- FR/ZH locale behavior;
- single render;
- ZIP export.

### Production smoke

On Cloud9 CT400:

- health;
- existing templates;
- font catalog;
- legacy endpoint;
- new asset upload;
- reframe;
- starter-brand render;
- package ZIP;
- verify no test artifacts remain.

## 16. Security and operations

Do not alter established Cloud9 mounts without explicit user approval.

Use existing writable storage:

- current CoverForge output tree;
- template directory;
- custom font directory.

If an uploads or packages subdirectory is needed, create it below an already writable CoverForge-controlled path.

No external URL fetch in v0.6 asset upload.

No SVG script execution.

No arbitrary file serving outside configured roots.

Clean temporary files and smoke artifacts after verification.

## 17. Delivery/versioning

Target release: **v0.6.0**

Delivery sequence:

1. implement on an isolated feature branch/worktree;
2. keep existing production templates untouched during code development;
3. validate locally;
4. push feature branch;
5. merge to main only after verification/review;
6. tag `v0.6.0`;
7. deploy to existing CT400 CoverForge stack;
8. preserve Cloud9’s intentional `coverforge:latest` compose override;
9. final live smoke-test;
10. remove all temporary artifacts.

## 18. Success criteria

v0.6 is successful when a new user with no podcast context can:

1. open CoverForge and immediately understand it as a branding automation studio;
2. import one image;
3. add a logo, title and subtitle;
4. see that image branded across multiple common formats;
5. adjust crops and layers;
6. export all selected outputs at once;
7. perform the same workflow through the API;
8. understand the product in French or Simplified Chinese;
9. do all of this without breaking existing CoverForge/AutoPublisher workflows.
