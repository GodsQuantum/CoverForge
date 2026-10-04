# Canva layout audit — 2026-10-04

Source: the four user-provided Canva folders, inspected read-only through Canva's editing transaction API. Canva exposes exact page geometry and editable text/image bounding boxes, but this connector does **not** expose the font family or point-size metadata of the existing text elements. Therefore:

- geometry below is exact and normalized against the Canva page size;
- existing CoverForge show font families are preserved where already curated;
- CoverForge v0.5 uses exact font face variants (weight + italic) and auto-fit to make text size follow the imported Canva boxes.

## Sources

| Show | Canva design | Design ID | Canvas | CoverForge target |
|---|---|---|---|---|
| CF | Chougar Free - Vignette | DAGsGgx7Hdo | 1080×1080 | square |
| CF | CF - Reel Covers | DAGsGhZstfE | 1080×1920 | vertical |
| CF | Chougar Free - YouTube | DAGsGgmB8t0 | 1280×720 | youtube |
| CP | Conspi Passion - Instagram Cover | DAGsGnjDkJw | 1080×1920 | vertical |
| CP | Conspi Passion - YouTube | DAGsGqGuuHI | 1280×720 | youtube |
| DPAFM | DPAFM - Instagram Cover | DAGsGq8QVog | 1080×1920 | vertical |
| DPAFM | DPAFM - YouTube | DAGsGladGKQ | 1280×720 | youtube |
| LCFP | LCFP - YT Thumbnail | DAHHmt8Yrgw | 1280×720 | youtube |

## Normalized text boxes imported

Coordinates are `x, y, width, height` on a 0–1 canvas; negative or >1 values are intentional when Canva bleeds text beyond the canvas.

### CF

- square episode/eyebrow: `0.021365, 0.037047, 0.078635, 0.062953`
- square title: `0.230341, 0.269995, 0.539318, 0.460011`
- vertical title: `0.047313, 0.579878, 0.875657, 0.154749`
- vertical subtitle/eyebrow: `0.039443, 0.715551, 0.921114, 0.038152`
- youtube title: `0.048560, 0.658640, 0.448692, 0.163301`
- youtube episode/eyebrow: `0.460818, 0.822954, 0.209072, 0.105134`

The Canva YouTube page also contains separate section and date text boxes. They are documented but not forced into the production template yet to avoid introducing required variables that current AutoPublisher jobs do not send.

### CP

- vertical title: `0.028116, 0.591092, 0.971884, 0.155122`
- youtube title: `-0.096576, 0.738271, 1.193152, 0.261729`

The current Canva square asset is flattened (no editable text element exposed), so no fake typography geometry is inferred for square.

### DPAFM

- youtube title: `0.270585, 0.713372, 0.711955, 0.261798` — already matched the existing CoverForge production template before this audit.
- vertical adaptive title envelope: based on current Canva text pages, `0.048687, 0.550665, 0.907803, 0.203866`. This envelope covers the one-line and multi-line title variants and is intended for CoverForge auto-fit.
- vertical guest/eyebrow: `0.590640, 0.716453, 0.382826, 0.041356` (right-aligned reference from the current Canva guest layout).

### LCFP

- youtube title: `0.019628, 0.687270, 0.856061, 0.181641`
- youtube subtitle/guest/eyebrow: `0.019628, 0.851750, 0.330477, 0.112485`

The podcast-square Canva source is currently composed from flattened image fills, so its text font geometry cannot be extracted independently.

## Font metadata limitation

The Canva connector exposes text content and exact element bounds but not the original element's font family/weight/point size. CoverForge therefore does not invent font identities. Existing show families remain authoritative:

- CF: League Spartan + Yanone Kaffeesatz
- CP: Nimbus Sans Narrow + League Spartan
- DPAFM: Impact + League Spartan
- LCFP: League Spartan + Nimbus Sans Narrow

v0.5 exposes every available face/variant of those families in the Studio and persists `font_weight` + `font_style` through the JSON API and Rust renderer.
