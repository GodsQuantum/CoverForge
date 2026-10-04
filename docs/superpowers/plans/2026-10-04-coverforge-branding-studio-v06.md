# CoverForge Branding Studio v0.6 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn CoverForge from a podcast/show-oriented compositor into a generic bilingual branding automation studio that can ingest one image, smart-reframe it into multiple formats, apply brand layers, and export individual assets or a ZIP through both UI and API without breaking existing AutoPublisher workflows.

**Architecture:** Keep Rust/resvg as the deterministic production renderer and keep CoverForge JSON as the source of truth. Add isolated Rust modules for asset ingestion, smart reframe, and packaging; split the oversized Svelte page into typed modules/components while keeping Fabric as an editing surface only; derive uploads/packages from the existing output directory so Cloud9 mounts do not change.

**Tech Stack:** Rust 2024, Axum 0.8, image 0.25, kamadak-exif 0.6.1, resvg 0.48, zip 8.4.0, time 0.3.55, Svelte 5, SvelteKit 3, Vite 8, Fabric.js 7.4, TypeScript 6, Vitest 5.0.3.

**Spec:** `docs/superpowers/specs/2026-10-04-coverforge-branding-studio-v06-design.md`

## Global Constraints

- Target release is **v0.6.0**.
- Keep all existing production template ids and JSON files loadable without migration.
- Keep `POST /api/generate` compatible with AutoPublisher.
- Keep all existing `/v1/*` routes compatible; new behavior is additive.
- Do **not** change Cloud9 mounts, established paths, or the existing output root.
- Derive `uploads/` and `packages/` below the current CoverForge output directory.
- JSON remains the canonical persistence format; Fabric objects are never persisted as the canonical document.
- Smart Reframe is local and deterministic; no required external ML/API.
- UI and user-facing docs are bilingual French + Simplified Chinese.
- Product copy must describe generic branding automation; podcast/show wording is allowed only for backward-compatibility examples.
- Use the approved orange/cream/charcoal brand system: `#FF7A00`, `#FF4D00`, `#FFC64D`, `#FFF8F1`, `#111827`.
- Do not bundle proprietary fonts.
- Do not introduce a database in v0.6.
- No external URL fetching in asset upload/reframe.
- No arbitrary file serving outside configured roots.
- SVG uploads are logo-only and must not permit script execution or external resource loading.
- Remove smoke artifacts and temporary files after verification.
- Cloud9 keeps its intentional local `coverforge:latest` compose override after deployment.

## Review Focus

1. **Very large raster uploads:** a 50+ megapixel image must be rejected before full decoded processing can exhaust the 768 MiB container; Task 2 pins this with `rejects_raster_over_pixel_limit`.
2. **Malicious SVG logos:** scripts, event handlers, external URLs and filesystem/network references must be rejected; Task 2 pins this with `rejects_active_or_external_svg`.
3. **Extreme target ratios:** smart reframe for ultra-wide and ultra-tall targets must remain deterministic and inside normalized bounds; Task 3 pins this with `extreme_ratios_stay_bounded_and_deterministic`.
4. **Package filename collisions / traversal:** duplicate or hostile format/output names must not escape the package and must remain unique; Task 5 pins this with `package_names_are_safe_and_unique`.
5. **Existing production compatibility:** old JSON without v0.6 metadata plus legacy AutoPublisher payloads must behave as before; Tasks 4 and 10 pin this with backward-compatibility tests and final live smoke.

---

### Task 1: Frontend foundation, brand assets, i18n and typed API

**Files:**
- Create: `brand/coverforge-logo.svg`
- Create: `brand/coverforge-mark.svg`
- Create: `web/static/coverforge-logo.svg`
- Create: `web/static/favicon.svg`
- Create: `web/src/lib/types.ts`
- Create: `web/src/lib/i18n.ts`
- Create: `web/src/lib/api.ts`
- Create: `web/src/lib/i18n.test.ts`
- Create: `web/src/lib/brand.test.ts`
- Modify: `web/package.json`
- Modify: `web/package-lock.json`
- Modify: `web/src/routes/+page.svelte`

**Interfaces:**
- Consumes: current inline TypeScript types and fetch calls from `+page.svelte`.
- Produces:
  - `export type Locale = 'fr' | 'zh-CN'`
  - `export function tr(locale: Locale, key: TranslationKey): string`
  - `export const translations: Record<Locale, Record<TranslationKey,string>>`
  - typed frontend model interfaces in `lib/types.ts`
  - `export const api` client with methods matching existing CoverForge routes
  - reusable vector logo/mark assets with no embedded raster image.

- [ ] **Step 1: Add Vitest and the failing i18n/brand tests**

In `web/package.json`, add `"test": "vitest run"` and exact dev dependency `"vitest": "5.0.3"`.

Create tests asserting:

```ts
expect(tr('fr','nav.project')).toBe('Projet')
expect(tr('zh-CN','nav.project')).toBe('项目')
expect(tr('fr','action.exportAll')).toBe('Exporter tout')
expect(tr('zh-CN','action.exportAll')).toBe('全部导出')
```

Brand test reads the SVG files and asserts:

```ts
expect(svg).toContain('#FF7A00')
expect(svg).toContain('#FF4D00')
expect(svg).not.toMatch(/<image\b|data:image|https?:\/\//i)
```

Run: `cd web && npm test -- src/lib/i18n.test.ts src/lib/brand.test.ts`  
Expected: FAIL because modules/assets do not exist.

- [ ] **Step 2: Create the vector brand assets**

Recreate the approved logo reference as clean SVG: crop/focus brackets + stacked image cards + orange/deep-orange gradient; full wordmark uses Charcoal `Cover` + orange `Forge`.

Copy canonical SVGs into `web/static/` and use the mark as `favicon.svg`.

Run the brand test.  
Expected: brand test PASS.

- [ ] **Step 3: Extract frontend types from `+page.svelte`**

Move `View`, `Frame`, `CanvasDef`, `Layer`, `FormatDef`, `TemplateData`, `RenderedAsset`, `RenderResponse`, `FontRecord` into `web/src/lib/types.ts`. Extend the view union for v0.6:

```ts
export type View = 'project' | 'templates' | 'library' | 'brand' | 'exports' | 'api'
```

Do not change runtime behavior yet.

Run: `cd web && npm run check`  
Expected: PASS.

- [ ] **Step 4: Implement bilingual translations**

`i18n.ts` must contain all navigation labels plus core actions for import, replace image, smart crop, reset crop, layers, formats, export selected, export all, brand kit, templates, library, API, loading and error states.

Run: `cd web && npm test -- src/lib/i18n.test.ts`  
Expected: PASS.

- [ ] **Step 5: Extract typed API calls**

Create `web/src/lib/api.ts` with methods for existing routes only in this task:

```ts
listTemplates(): Promise<string[]>
getTemplate(id: string): Promise<TemplateData>
putTemplate(id: string, template: TemplateData): Promise<void>
listFonts(): Promise<FontRecord[]>
render(request: RenderRequest): Promise<RenderResponse>
renderPreview(request: InlineRenderRequest): Promise<RenderResponse>
```

Replace equivalent raw `fetch` calls in `+page.svelte`.

Run: `cd web && npm run check && npm test`  
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add brand web/package.json web/package-lock.json web/static web/src/lib web/src/routes/+page.svelte
git commit -m "feat: add CoverForge brand and bilingual frontend foundation"
```

---

### Task 2: Secure asset upload and metadata

**Files:**
- Create: `src/assets.rs`
- Modify: `src/main.rs`
- Modify: `src/model.rs`
- Modify: `src/render.rs`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Create: `tests/fixtures/orientation-6.jpg`

**Interfaces:**
- Consumes:
  - `Renderer::output_dir() -> &Path`
  - Axum `Multipart`
- Produces:
  - `pub const MAX_UPLOAD_BYTES: usize = 50 * 1024 * 1024`
  - `pub const MAX_DECODED_PIXELS: u64 = 50_000_000`
  - dependency `kamadak-exif = "0.6.1"`
  - `pub struct AssetStore { root: PathBuf }`
  - `AssetStore::new(output_dir: &Path) -> Result<Self>`
  - `AssetStore::save_upload(filename: &str, mime: Option<&str>, bytes: &[u8]) -> Result<UploadedAsset>`
  - `pub fn load_raster_safely(path: &Path) -> Result<DynamicImage>` — dimension guard + EXIF orientation normalization used by upload and Smart Reframe
  - `pub struct UploadedAsset { id, filename, path, url, mime, width, height, kind }`
  - `POST /v1/assets`
  - static preview path `/uploads/{filename}`.

- [ ] **Step 1: Write failing filename and file-type tests**

Tests in `src/assets.rs`:

```rust
assert_eq!(safe_upload_filename("../../brand photo.png")?, "brand-photo.png");
assert!(safe_upload_filename("../..").is_err());
assert!(validate_extension("photo.exe").is_err());
assert!(!safe_upload_filename("été photo.png")?.contains('/'));
```

Run: `cargo test assets::tests::safe_upload_names --locked`  
Expected: FAIL because `assets` module does not exist.

- [ ] **Step 2: Implement filename sanitization and upload root**

`AssetStore::new` creates `<output_dir>/uploads`. Sanitized names must be slug-like ASCII, preserve only the validated extension, and receive a UUID prefix so collisions do not overwrite prior uploads.

Run the filename tests.  
Expected: PASS.

- [ ] **Step 3: Write failing raster pixel-limit and metadata tests**

Generate a small in-memory PNG and assert width/height/MIME are detected. Add a synthetic image/fixture that exceeds `MAX_DECODED_PIXELS` and assert rejection before full processing.

Required test names:

- `reads_raster_metadata`
- `rejects_raster_over_pixel_limit`
- `rejects_unsupported_raster`
- `normalizes_exif_orientation` — fixture Orientation=6 starts as 2×3 and yields normalized 3×2 dimensions.

Run: `cargo test assets::tests --locked`  
Expected: FAIL on missing validation.

- [ ] **Step 4: Implement JPEG/PNG/WebP validation**

Use image format sniffing rather than trusting filename/MIME. Read image dimensions before full decode, then enforce:

- request field bytes <= 50 MiB;
- decoded width × height <= 50,000,000 pixels;
- accepted raster formats exactly JPEG, PNG, WebP.

For JPEG, read EXIF orientation with `kamadak-exif`. Persist the original validated upload bytes, but make `load_raster_safely` apply EXIF orientation at decode time and return normalized pixels/dimensions; PNG/WebP remain unchanged. Update `Renderer::prepared_image` to use `load_raster_safely` so both render and Smart Reframe see the same orientation without re-encoding the uploaded source. Return `UploadedAsset.kind = "image"` and normalized width/height.

Run: `cargo test assets::tests --locked`  
Expected: raster tests PASS.

- [ ] **Step 5: Write failing malicious SVG test**

Fixtures must reject at least:

```xml
<svg onload="alert(1)"></svg>
<svg><script>alert(1)</script></svg>
<svg><image href="https://example.com/a.png"/></svg>
<svg><use href="file:///etc/passwd#x"/></svg>
```

Test name: `rejects_active_or_external_svg`.

Run it.  
Expected: FAIL.

- [ ] **Step 6: Implement SVG logo validation**

SVG upload rules:

- maximum 2 MiB;
- UTF-8 XML only;
- reject `<script`, event-handler attributes beginning with `on`, `javascript:`, `http://`, `https://`, `file://`, and external `href/xlink:href`;
- accept local vector primitives and local fragment references;
- require a positive `viewBox` or explicit positive width/height and return `kind = "logo"` with resolved dimensions.

Add test `safe_svg_logo_renders` proving a sanitized uploaded SVG can be used by an image layer. Update `Renderer::prepared_image` to detect SVG, rerun the same sanitizer on file bytes, parse with the existing resvg/usvg stack, and rasterize it into the requested layer box; Smart Reframe continues to reject SVG.

Run all asset tests.  
Expected: PASS.

- [ ] **Step 7: Add `POST /v1/assets`, renderer SVG support and serving**

Add `asset_store: Arc<AssetStore>` to `AppState`, derive it from existing renderer output dir, add route and `ServeDir` for `/uploads`. Ensure `Renderer::prepared_image` dispatches raster files through `load_raster_safely` and sanitized SVG logos through the existing resvg/usvg pipeline.

Also make the renderer treat its own canonical `output_dir` as an allowed asset root in addition to `COVERFORGE_ASSET_ROOTS`, deduplicated. Add regression test `output_dir_is_safe_asset_root` proving an uploaded file can be used immediately as `{{background}}` while `/etc/passwd` remains rejected.

Raise Axum `DefaultBodyLimit` to **52 MiB** to accommodate 50 MiB multipart payload plus overhead.

Endpoint expects multipart field `asset`.

Run: `cargo test --locked && cargo clippy --locked --all-targets -- -D warnings`  
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml Cargo.lock src/assets.rs src/main.rs src/model.rs src/render.rs tests/fixtures/orientation-6.jpg
git commit -m "feat: add secure asset upload API"
```

---

### Task 3: Deterministic Smart Reframe engine

**Files:**
- Create: `src/reframe.rs`
- Modify: `src/main.rs`
- Modify: `src/model.rs`

**Interfaces:**
- Consumes:
  - uploaded/local raster path resolved through existing safe asset roots plus renderer `output_dir`;
  - Task 2 `load_raster_safely(path: &Path) -> Result<DynamicImage>`;
  - `Frame { x, y, width, height }`.
- Produces:
  - `pub struct Point { x: f32, y: f32 }`
  - `pub struct ReframeTarget { id: String, width: u32, height: u32 }`
  - `pub struct ReframeRequest { asset: String, formats: Vec<ReframeTarget>, focal_override: Option<Point> }`
  - `pub struct ReframeResult { format, crop: Frame, focal: Point, score: f32 }`
  - `pub fn smart_reframe(image: &DynamicImage, targets: &[ReframeTarget], focal_override: Option<Point>) -> Vec<ReframeResult>`
  - `POST /v1/reframe`.

- [ ] **Step 1: Write failing centered-image reframe tests**

Create a synthetic image with the highest contrast/detail in a known region and assert 1:1 and 9:16 crops include that region and remain in `0..=1`.

Run: `cargo test reframe::tests::finds_salient_region --locked`  
Expected: FAIL because module does not exist.

- [ ] **Step 2: Implement normalized crop geometry**

For each target ratio, compute the largest target-aspect crop inside the source. Represent crop as normalized `Frame`. This geometry must never depend on output pixel resolution.

Run geometry tests.  
Expected: PASS for crop dimensions/bounds but saliency test remains failing until scoring exists.

- [ ] **Step 3: Implement deterministic saliency scoring**

Use a downscaled analysis image capped at **256 px on the longest side**. Score cells using:

- local luminance edge/detail;
- color saturation;
- center-of-mass of high scoring cells;
- mild rule-of-thirds attraction.

No randomness.

The final focal point is the weighted centroid clamped to `0..=1`; crop is positioned around it while staying inside source bounds.

Run: `cargo test reframe::tests --locked`  
Expected: saliency tests PASS.

- [ ] **Step 4: Add focal override test and implementation**

Test:

```rust
let focal = Point { x: 0.9, y: 0.1 };
let result = smart_reframe(&image, &targets, Some(focal));
assert_eq!(result[0].focal, focal);
```

Crop must still be clamped inside bounds.

Run focal override test.  
Expected: PASS after implementation.

- [ ] **Step 5: Add Review Focus extreme-ratio test**

Test name: `extreme_ratios_stay_bounded_and_deterministic`.

Targets:

- `10000 × 200`
- `200 × 10000`.

Call Smart Reframe twice; results must be exactly equal and all crop coordinates/sizes valid.

Run: `cargo test reframe::tests::extreme_ratios_stay_bounded_and_deterministic --locked`  
Expected: PASS.

- [ ] **Step 6: Add `POST /v1/reframe`**

Resolve the raster source using the same safe root policy as rendering, then decode through Task 2 `load_raster_safely` so pixel limits and EXIF orientation are identical to upload. Reject SVG and unsupported image types. Return JSON results in request order.

Run: `cargo test --locked && cargo clippy --locked --all-targets -- -D warnings`  
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/reframe.rs src/main.rs src/model.rs
git commit -m "feat: add deterministic smart reframe API"
```

---

### Task 4: Generic template metadata and public starter brand

**Files:**
- Modify: `src/model.rs`
- Modify: `src/render.rs`
- Modify: `compose.yaml`
- Create: `templates/starter-brand.json`
- Modify: `docs/SHOW_STYLES.md`
- Test: existing model tests in `src/model.rs`

**Interfaces:**
- Consumes: current `Template`, `BrandStyle`, `FormatTemplate`.
- Produces:
  - `BrandStyle.logos: BTreeMap<String,String>`
  - optional/defaulted `FormatTemplate.label`, `category`, `platform`, `suffix`
  - generic starter template with `youtube`, `square`, `feed`, `vertical`, `landscape`;
  - `Renderer` default-template fallback: user `COVERFORGE_TEMPLATE_DIR` wins, then read-only `COVERFORGE_DEFAULT_TEMPLATE_DIR` if present; saves always target the user template dir.

- [ ] **Step 1: Write failing legacy JSON compatibility test**

Use a v0.5-shaped JSON with no `logos` or format metadata and assert it deserializes with empty/default values.

Test name: `v05_templates_load_without_v06_metadata`.

Run it.  
Expected: FAIL because new fields/types do not exist yet.

- [ ] **Step 2: Add defaulted v0.6 metadata fields**

All new fields must use `#[serde(default)]` so existing production JSON does not need migration.

Run model tests.  
Expected: PASS.

- [ ] **Step 3: Write failing starter-template contract test**

Load `templates/starter-brand.json` and assert:

- id `starter-brand`;
- five required formats with exact starter canvases: `youtube` 1920×1080, `square` 1080×1080, `feed` 1080×1350, `vertical` 1080×1920, `landscape` 1200×628;
- variables `background`, `logo`, `title`, `subtitle`, `badge`;
- every format includes an image layer and title layer;
- title layers use `auto_fit: true`.

Run: `cargo test model::tests::starter_brand_contract --locked`  
Expected: FAIL because file does not exist.

- [ ] **Step 4: Create `starter-brand.json`**

Use only libre defaults and the v0.6 palette. Each format has independent normalized layout tuned to its ratio. Source image uses `{{background}}`; logo uses `{{logo}}`; optional badge uses `{{badge}}`.

Run starter contract test and a render smoke locally with empty optional variables.  
Expected: PASS.

- [ ] **Step 5: Write failing default-template fallback tests**

Create temp user/default template dirs and assert:

- ids from both dirs appear once in `list_templates()`;
- `load_template("starter-brand")` falls back to the default dir when absent from user dir;
- a user template with the same id overrides the default template;
- `save_template` writes only to the user template dir.

Run targeted renderer tests.  
Expected: FAIL before fallback support exists.

- [ ] **Step 6: Implement default-template fallback**

Add optional `default_template_dir` to `Renderer`. `from_env()` reads `COVERFORGE_DEFAULT_TEMPLATE_DIR`; ignore it when missing/nonexistent. In Docker Compose set it to `/app/default-templates`, which is already populated by the Dockerfile. Do not add a mount.

Run renderer/model tests.  
Expected: PASS.

- [ ] **Step 7: Update template documentation**

Rewrite terminology in `docs/SHOW_STYLES.md` from show-specific to generic template/project language while preserving a backward-compatibility section for production shows. Document built-in/default template precedence.

Run: `git diff --check`  
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add src/model.rs src/render.rs compose.yaml templates/starter-brand.json docs/SHOW_STYLES.md
git commit -m "feat: add generic brand template metadata"
```

---

### Task 5: Batch package export and manifest

**Files:**
- Create: `src/package.rs`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `src/main.rs`
- Modify: `src/model.rs`
- Modify: `src/render.rs`

**Interfaces:**
- Consumes:
  - `Renderer::render(&RenderRequest)`
  - `Renderer::render_inline(&InlineRenderRequest)`
  - output directory.
- Produces:
  - dependency `zip = { version = "8.4.0", default-features = false, features = ["deflate"] }`
  - dependency `time = { version = "0.3.55", features = ["formatting"] }`
  - `PackageRenderRequest { template: Option<String>, inline_template: Option<Template>, variables, variants, output_stem }`
  - exactly one of `template` / `inline_template` must be present;
  - `PackageManifest`
  - `PackageRenderResponse`
  - `POST /v1/render/package`
  - packages stored under `<output_dir>/packages` and served through existing `/outputs/packages/...`.

- [ ] **Step 1: Write failing request-validation tests**

Assert requests with both template sources or neither source are rejected. One source is accepted.

Run: `cargo test package::tests::validates_exactly_one_template_source --locked`  
Expected: FAIL.

- [ ] **Step 2: Implement package request validation**

Keep source selection isolated in `package.rs`.

Run validation test.  
Expected: PASS.

- [ ] **Step 3: Write failing manifest test**

Render two variants and assert manifest includes:

- CoverForge version;
- RFC3339 generated timestamp;
- template/project id;
- source image reference derived from `variables.background` when present;
- variables;
- variant ids;
- dimensions;
- filenames.

Run: `cargo test package::tests::manifest_describes_rendered_assets --locked`  
Expected: FAIL.

- [ ] **Step 4: Implement manifest generation**

Use UTC via `time::OffsetDateTime::now_utc()` and RFC3339 formatting.

Run manifest test.  
Expected: PASS.

- [ ] **Step 5: Write failing ZIP safety/collision test**

Test name: `package_names_are_safe_and_unique`.

Use hostile/duplicate requested output stems/format identifiers such as:

- `../../escape`
- `youtube`
- duplicate `youtube`.

Assert ZIP entry names contain no `..`, no absolute path, and are unique.

Run test.  
Expected: FAIL.

- [ ] **Step 6: Implement deterministic ZIP creation**

ZIP root contains images plus `manifest.json`. Normalize the requested variant list to stable first-occurrence order before rendering, then use sanitized basename + deterministic numeric suffix for any remaining filename collisions. No source absolute paths are used as archive entry names.

Run all package tests.  
Expected: PASS.

- [ ] **Step 7: Add package endpoint**

`POST /v1/render/package` renders selected variants first, writes the ZIP under `packages/`, and returns standard asset metadata plus package URL and manifest.

Run: `cargo test --locked && cargo clippy --locked --all-targets -- -D warnings`  
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml Cargo.lock src/package.rs src/main.rs src/model.rs src/render.rs
git commit -m "feat: add multi-format ZIP package export"
```

---

### Task 6: Generic bilingual application shell and orange visual system

**Files:**
- Create: `web/src/lib/components/AppShell.svelte`
- Create: `web/src/lib/components/ProjectHeader.svelte`
- Modify: `web/src/routes/+page.svelte`
- Modify: `web/src/app.css`
- Modify: `web/src/lib/i18n.ts`
- Modify: `web/src/lib/i18n.test.ts`

**Interfaces:**
- Consumes:
  - Task 1 `Locale`, `tr`, logo assets, typed `View`.
- Produces:
  - generic navigation views `project/templates/library/brand/exports/api`;
  - locale selector FR / 中文;
  - orange/cream/charcoal CSS tokens;
  - no show/podcast wording in primary UI.

- [ ] **Step 1: Extend failing translation test for full navigation**

Assert all six navigation labels and primary Create actions exist in both locales.

Run: `cd web && npm test -- src/lib/i18n.test.ts`  
Expected: FAIL on missing new keys.

- [ ] **Step 2: Complete translations**

Add all shell/header labels and short descriptions.

Run translation test.  
Expected: PASS.

- [ ] **Step 3: Build `AppShell.svelte` and `ProjectHeader.svelte`**

Shell requirements:

- approved full logo in sidebar/header;
- locale control for long-form copy;
- six generic sections;
- primary navigation/action labels rendered as bilingual FR / 中文 pairs where space permits;
- mobile collapse;
- no “show”, “émission”, “podcast” in generic shell.

Keep old editor internals temporarily mounted under the Project view.

Run: `cd web && npm run check`  
Expected: PASS.

- [ ] **Step 4: Replace green visual tokens**

Set canonical CSS variables:

```css
--forge:#FF7A00;
--deep-orange:#FF4D00;
--amber:#FFC64D;
--cream:#FFF8F1;
--charcoal:#111827;
```

Use cream/light surfaces by default with dark charcoal text and orange primary actions. Preserve contrast for editor canvas areas.

Run: `cd web && npm run build`  
Expected: PASS.

- [ ] **Step 5: Add static-copy regression check**

Run:

```bash
grep -RniE 'podcast|émission|show style' web/src/lib/components web/src/routes/+page.svelte
```

Expected: no generic UI occurrences; compatibility-specific technical labels may remain only where explicitly annotated.

- [ ] **Step 6: Commit**

```bash
git add web/src/lib/components/AppShell.svelte web/src/lib/components/ProjectHeader.svelte web/src/lib/i18n.ts web/src/lib/i18n.test.ts web/src/routes/+page.svelte web/src/app.css
git commit -m "feat: rebrand Studio as bilingual branding automation"
```

---

### Task 7: Source image upload, Smart Reframe controls and multi-format preview

**Files:**
- Create: `web/src/lib/components/SourceImagePanel.svelte`
- Create: `web/src/lib/components/MultiFormatGrid.svelte`
- Create: `web/src/lib/components/FormatCard.svelte`
- Create: `web/src/lib/format-state.ts`
- Create: `web/src/lib/format-state.test.ts`
- Modify: `web/src/lib/api.ts`
- Modify: `web/src/lib/types.ts`
- Modify: `web/src/routes/+page.svelte`
- Modify: `web/src/app.css`

**Interfaces:**
- Consumes:
  - Task 2 `POST /v1/assets`;
  - Task 3 `POST /v1/reframe`;
  - current template formats;
  - current Fabric detailed editor.
- Produces:
  - `api.uploadAsset(file: File): Promise<UploadedAsset>`
  - `api.reframe(request: ReframeRequest): Promise<ReframeResult[]>`
  - `toggleFormatSelection(selected: Set<string>, id: string): Set<string>`
  - `applyReframeToTemplate(template, results): TemplateData`
  - source image drop/replace flow;
  - simultaneous multi-format cards.

- [ ] **Step 1: Write failing format-state tests**

Assert:

- selecting/unselecting preserves stable order;
- unknown formats are ignored;
- applying a crop sets image layer focal values for the corresponding format without mutating other formats;
- an empty selected set is normalized to the active format before export.

Run: `cd web && npm test -- src/lib/format-state.test.ts`  
Expected: FAIL.

- [ ] **Step 2: Implement `format-state.ts`**

Keep state helpers pure and independent of Svelte/Fabric.

Run unit test.  
Expected: PASS.

- [ ] **Step 3: Add typed asset/reframe API client methods**

Use `FormData` field name `asset`. Add exact response/request interfaces to `types.ts`. At the same time sync frontend v0.6 model metadata: `BrandStyle.logos` and optional `FormatDef.label/category/platform/suffix`, matching Task 4 defaults.

Run: `cd web && npm run check`  
Expected: PASS.

- [ ] **Step 4: Implement `SourceImagePanel.svelte`**

Behavior:

- drag/drop and file picker;
- accepted UI types JPEG/PNG/WebP;
- upload progress/loading state;
- preview uploaded asset;
- replace image;
- global focal marker;
- “Recadrage intelligent / 智能裁切” action;
- reset to automatic;
- error state from API.

After successful upload set `variables.background` to the server-side asset path while preview uses the returned URL.

Run: `cd web && npm run check`  
Expected: PASS.

- [ ] **Step 5: Implement multi-format grid**

Each card shows:

- format label;
- dimensions/ratio;
- selected checkbox;
- live image preview using returned crop/focal;
- click opens detailed format editing.

Built-in ordering for starter template: YouTube, Square, Feed, Vertical, Landscape. Custom formats append after built-ins.

Run: `cd web && npm run check && npm test`  
Expected: PASS.

- [ ] **Step 6: Integrate with detailed Fabric editor**

Selecting a card updates `activeFormat`, selected layer state, and Fabric canvas. Manual focal edits remain per-format; global reframe must not overwrite a format marked manual until user resets it to automatic.

Represent this UI-only mode as `Record<string,'auto'|'manual'>`; do not add persistence metadata unless required.

Run: `cd web && npm run check && npm run build`  
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add web/src/lib/components/SourceImagePanel.svelte web/src/lib/components/MultiFormatGrid.svelte web/src/lib/components/FormatCard.svelte web/src/lib/format-state.ts web/src/lib/format-state.test.ts web/src/lib/api.ts web/src/lib/types.ts web/src/routes/+page.svelte web/src/app.css
git commit -m "feat: add upload smart reframe and multi-format preview"
```

---

### Task 8: Brand Kit, library, layer inspector and export UX

**Files:**
- Create: `web/src/lib/components/BrandKitPanel.svelte`
- Create: `web/src/lib/components/AssetLibrary.svelte`
- Create: `web/src/lib/components/LayerInspector.svelte`
- Create: `web/src/lib/components/ExportPanel.svelte`
- Create: `web/src/lib/export-state.ts`
- Create: `web/src/lib/export-state.test.ts`
- Modify: `web/src/lib/api.ts`
- Modify: `web/src/lib/types.ts`
- Modify: `web/src/routes/+page.svelte`
- Modify: `web/src/app.css`

**Interfaces:**
- Consumes:
  - current layer editing logic from `+page.svelte`;
  - Task 2 upload;
  - Task 5 package endpoint.
- Produces:
  - extracted reusable `LayerInspector`;
  - Brand Kit editor for logos/palette/fonts;
  - uploaded-session asset library;
  - `api.renderPackage(request): Promise<PackageRenderResponse>`;
  - export-selection helper and ZIP download action.

- [ ] **Step 1: Write failing export-state tests**

Assert:

- export request carries only selected formats;
- request uses saved template id when not dirty;
- request uses inline template when dirty;
- exactly one template source is sent.

Run: `cd web && npm test -- src/lib/export-state.test.ts`  
Expected: FAIL.

- [ ] **Step 2: Implement pure export request builder**

Signature:

```ts
buildPackageRequest(input: {
  templateId: string;
  template: TemplateData;
  dirty: boolean;
  variables: Record<string,string>;
  selectedFormats: string[];
}): PackageRenderRequest
```

Run test.  
Expected: PASS.

- [ ] **Step 3: Extract `LayerInspector.svelte`**

Move current layer controls without losing:

- X/Y/W/H sliders + numeric values;
- family + real font face;
- text auto-fit;
- alignment;
- image focal X/Y;
- opacity;
- layer visibility/lock;
- advanced typography.

No functional regression.

Run: `cd web && npm run check`  
Expected: PASS.

- [ ] **Step 4: Implement Brand Kit panel**

Edit:

- display name;
- primary/secondary logo references;
- palette keys/colors;
- font roles/families.

Uploading a logo uses `POST /v1/assets`; response path becomes the stored brand logo reference.

Run: `cd web && npm run check`  
Expected: PASS.

- [ ] **Step 5: Implement session Asset Library**

Show assets uploaded in the current browser session and brand logos; allow assigning an asset as background or logo without re-upload.

No database or persistent asset-index subsystem in v0.6.

Run: `cd web && npm run check`  
Expected: PASS.

- [ ] **Step 6: Implement Export panel**

Actions:

- render selected;
- render all;
- download individual image;
- create/download ZIP;
- show package manifest summary;
- keep a current-session export history list so the Exports view is useful without adding a database.

Use package endpoint for dirty inline templates so export does not require a save.

Run: `cd web && npm run check && npm test && npm run build`  
Expected: PASS.

- [ ] **Step 7: Remove duplicated editor logic from `+page.svelte`**

After extraction, `+page.svelte` remains orchestration/state only. Target: no individual inspector input markup or raw API fetches in the route file.

Run:

```bash
cd web
npm run check
npm test
npm run build
grep -n "fetch(" src/routes/+page.svelte
```

Expected: checks PASS; grep has no raw API fetch.

- [ ] **Step 8: Commit**

```bash
git add web/src/lib/components web/src/lib/export-state.ts web/src/lib/export-state.test.ts web/src/lib/api.ts web/src/lib/types.ts web/src/routes/+page.svelte web/src/app.css
git commit -m "feat: add brand kit library and export workflow"
```

---

### Task 9: Templates and API utility views

**Files:**
- Create: `web/src/lib/components/TemplatesPanel.svelte`
- Create: `web/src/lib/components/ApiPanel.svelte`
- Create: `web/src/lib/template-state.ts`
- Create: `web/src/lib/template-state.test.ts`
- Modify: `web/src/lib/api.ts`
- Modify: `web/src/lib/types.ts`
- Modify: `web/src/routes/+page.svelte`
- Modify: `web/src/app.css`

**Interfaces:**
- Consumes:
  - Task 1 typed API client;
  - Task 4 `starter-brand` template metadata;
  - Task 6 navigation shell.
- Produces:
  - `cloneTemplateForId(template: TemplateData, id: string): TemplateData`
  - Templates view for list/open/duplicate/create-from-starter;
  - API view exposing live OpenAPI link, autofill dataset and copyable request examples;
  - no new backend endpoint for duplication.

- [ ] **Step 1: Write failing template-state tests**

Assert:

- clone deep-copies the source;
- clone changes only the template id;
- invalid ids containing spaces, slashes or `..` are rejected;
- source object is not mutated.

Run: `cd web && npm test -- src/lib/template-state.test.ts`  
Expected: FAIL.

- [ ] **Step 2: Implement `template-state.ts`**

Use the backend-compatible id policy: ASCII alphanumeric plus `-` and `_`, non-empty.

Run the unit test.  
Expected: PASS.

- [ ] **Step 3: Implement Templates panel**

Show:

- all templates from `GET /v1/templates`;
- current template;
- `starter-brand` highlighted as the generic starting point;
- open/edit;
- duplicate with new id;
- create from starter by GET starter → deep clone → PUT new id.

Do not rename or delete production templates.

Run: `cd web && npm run check`  
Expected: PASS.

- [ ] **Step 4: Implement API panel**

Show:

- `/openapi.json` link;
- selected template dataset from `/v1/templates/{id}/dataset`;
- copyable examples for upload, reframe, render and package export;
- current server version from `/health`.

Add typed client methods `health()` and `getDataset(id)`.

Run: `cd web && npm run check`  
Expected: PASS.

- [ ] **Step 5: Integrate views into AppShell**

`templates` and `api` navigation items must open these components instead of placeholders. Primary titles display bilingual FR / 中文 pairs; long descriptions follow the selected locale.

Run: `cd web && npm run check && npm test && npm run build`  
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add web/src/lib/components/TemplatesPanel.svelte web/src/lib/components/ApiPanel.svelte web/src/lib/template-state.ts web/src/lib/template-state.test.ts web/src/lib/api.ts web/src/lib/types.ts web/src/routes/+page.svelte web/src/app.css
git commit -m "feat: add template and API workspace views"
```

---

### Task 10: OpenAPI, generic README FR/ZH and compatibility regression

**Files:**
- Modify: `src/main.rs`
- Modify: `README.md`
- Rewrite: `README.fr.md`
- Rewrite: `README.zh-CN.md`
- Modify: `docs/SHOW_STYLES.md`
- Create: `docs/API.md`
- Create: `docs/assets/coverforge-workflow.svg`

**Interfaces:**
- Consumes: all new v0.6 API contracts from Tasks 2, 3 and 5.
- Produces:
  - OpenAPI entries for assets/reframe/package;
  - generic product documentation;
  - complete French and Simplified Chinese user docs;
  - preserved legacy/API compatibility.

- [ ] **Step 1: Write failing OpenAPI contract test**

In `src/main.rs` tests, extract/build the OpenAPI value through a pure helper `openapi_document() -> Value` and assert paths exist:

- `/v1/assets`
- `/v1/reframe`
- `/v1/render/package`.

Assert API description contains “branding” and not “podcast”.

Run targeted test.  
Expected: FAIL.

- [ ] **Step 2: Refactor OpenAPI into pure helper and document new routes**

Keep `async fn openapi() -> Json<Value>` as a thin wrapper around `openapi_document()`.

Run targeted test.  
Expected: PASS.

- [ ] **Step 3: Add backward-compatibility regression tests**

Required assertions:

- deserialize a v0.5 template with no new metadata;
- `legacy_template_parts("DPAFM - YT") == ("dpafm","youtube")`;
- legacy `LegacyGenerateRequest` JSON remains accepted;
- existing render request JSON remains accepted.

Run: `cargo test --locked`  
Expected: PASS.

- [ ] **Step 4: Rewrite README.md positioning**

Top section must include logo and generic workflow:

`Image → Smart crop → Brand kit → Dynamic layers → Multi-format → ZIP/API`.

README.md may be concise and route readers to complete FR/ZH files, but must not market the app primarily as podcast software. Create `docs/assets/coverforge-workflow.svg` as a lightweight product mockup/workflow visual using the canonical logo/palette and the stages Image → Smart crop → Brand kit → Dynamic layers → Multi-format → ZIP/API; embed it in the README without raster/base64 payloads.

- [ ] **Step 5: Rewrite complete French README**

Required sections:

- proposition produit;
- import d’image;
- Smart Reframe;
- Brand Kit;
- calques;
- multi-format;
- export ZIP;
- API/OpenAPI;
- quick start Docker;
- sécurité;
- compatibilité AutoPublisher;
- contribution;
- MIT.

- [ ] **Step 6: Rewrite complete Simplified Chinese README**

Mirror all French capabilities and examples, not a shortened translation.

- [ ] **Step 7: Add API documentation**

`docs/API.md` documents request/response examples for upload, reframe, package export, dataset and existing render endpoint.

Run:

```bash
git diff --check
grep -nEi 'podcast thumbnails|one show = one style file|une émission = un fichier' README.md README.fr.md README.zh-CN.md
```

Expected: diff check PASS; deprecated primary-positioning phrases absent.

- [ ] **Step 8: Full local verification**

Run:

```bash
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cd web
npm run check
npm test
npm run build
npm audit --json
```

Expected: all commands exit 0; npm audit reports 0 vulnerabilities.

- [ ] **Step 9: Commit**

```bash
git add src/main.rs README.md README.fr.md README.zh-CN.md docs/SHOW_STYLES.md docs/API.md docs/assets/coverforge-workflow.svg
git commit -m "docs: publish CoverForge branding automation v0.6"
```

---

### Task 11: Release, Cloud9 deployment and live end-to-end smoke

**Files:**
- Modify version only after all prior verification:
  - `Cargo.toml`
  - `Cargo.lock`
  - `web/package.json`
  - `web/package-lock.json`
  - `compose.yaml`
- Do not modify or rename existing production templates. The built-in `starter-brand` is served from the image fallback directory `/app/default-templates`; the persistent Cloud9 template mount remains untouched.

**Interfaces:**
- Consumes: verified feature branch from Tasks 1–10.
- Produces:
  - tagged `v0.6.0`;
  - updated existing CT400 CoverForge container;
  - unchanged Cloud9 mounts and local `coverforge:latest` override.

- [ ] **Step 1: Run pre-release version test RED**

Before bumping, assert:

```bash
test "$(grep '^version = ' Cargo.toml | head -1)" = 'version = "0.6.0"'
```

Expected: FAIL while package is still v0.5.1.

- [ ] **Step 2: Bump version to 0.6.0**

Update Rust, web package files and tracked compose image to `coverforge:0.6.0`. Refresh locks offline/through normal package manager as appropriate.

Run version assertion.  
Expected: PASS.

- [ ] **Step 3: Run complete verification again**

Run exactly:

```bash
cargo fmt --check &&
cargo test --locked &&
cargo clippy --locked --all-targets -- -D warnings &&
cd web &&
npm run check &&
npm test &&
npm run build &&
npm audit --json
```

Expected: all PASS and npm audit 0 vulnerabilities.

- [ ] **Step 4: Commit release version**

```bash
git add Cargo.toml Cargo.lock web/package.json web/package-lock.json compose.yaml
git commit -m "chore: release CoverForge v0.6.0"
```

- [ ] **Step 5: Whole-branch review before merge**

Use `superpowers:requesting-code-review` or the executing-plans self-review fallback. Review from merge-base with `main` through current HEAD, with special focus on Review Focus items and compatibility.

Expected: no unresolved Critical/Important findings before merge.

- [ ] **Step 6: Merge/push/tag**

After review fixes and final suite:

- fast-forward/merge feature branch to `main`;
- push `main`;
- create annotated tag `v0.6.0`;
- push tag.

Expected: GitHub main and tag point at the verified release commit.

- [ ] **Step 7: Diagnose live CT400 state before deployment**

Read-only checks:

```bash
git status --short --branch
git diff -- compose.yaml
docker inspect coverforge
docker ps --filter name=coverforge
```

Expected: only known local compose image override is present; no unexpected live changes.

- [ ] **Step 8: Deploy without changing mounts**

In CT400 repo:

1. temporarily restore tracked compose;
2. fast-forward to `origin/main`;
3. reapply only `image: coverforge:latest`;
4. assert compose differs from tracked file only by that line;
5. verify tracked compose contains `COVERFORGE_DEFAULT_TEMPLATE_DIR=/app/default-templates` and no mount changes;
6. `docker compose build coverforge`;
7. `docker compose up -d --force-recreate coverforge`.

Do not edit existing production template files, volumes, ports or established paths. The new `starter-brand` must appear through the read-only image fallback, not by copying into the persistent production template directory.

- [ ] **Step 9: Live smoke — old and new workflows**

Verify:

1. `GET /health` → version 0.6.0;
2. existing templates include `cf`, `cp`, `dpafm`, `lcfp` and the newly installed `starter-brand`;
3. existing font catalog returns faces;
4. `POST /api/generate` legacy DPAFM call renders;
5. upload a generated local PNG through `POST /v1/assets`;
6. `POST /v1/reframe` returns five bounded crops;
7. `starter-brand` renders all five formats;
8. `POST /v1/render/package` produces ZIP;
9. inspect ZIP names + `manifest.json`;
10. root UI contains CoverForge logo/copy plus FR/ZH labels;
11. no generic podcast/show wording in primary UI.

Expected: all PASS.

- [ ] **Step 10: Clean smoke artifacts**

Delete only:

- uploaded smoke asset;
- generated smoke images;
- smoke ZIP/package;
- temporary deploy compose backups.

Confirm production templates/fonts and non-smoke outputs remain untouched.

- [ ] **Step 11: Final live status**

Run:

```bash
docker inspect -f '{{.State.Health.Status}}' coverforge
git status --short --branch
git diff -- compose.yaml
```

Expected:

- `healthy`;
- branch at v0.6.0/main;
- only intentional `coverforge:latest` compose override;
- no smoke files.
