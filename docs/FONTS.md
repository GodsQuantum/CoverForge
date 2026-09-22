# Fonts

CoverForge has two font sources.

## Bundled fonts

The Docker image includes a broad curated set of redistributable families, including Noto, Roboto, Roboto Slab, Open Sans, Lato, Fira Code, Comfortaa, League Spartan, League Mono, Yanone Kaffeesatz, Cabin, Quicksand, Lobster, Lobster Two, TeX Gyre, Liberation, DejaVu, FreeFont, URW Base35, Linux Libertine, Gentium Plus, Charis, Andika, Cantarell, Carlito and Caladea.

The current catalogue is available from `GET /v1/fonts`.

## Custom fonts

Production custom fonts live outside Git and outside the image:

```text
/srv/storage/production/active/Assets/CoverForge/Fonts/Custom/
```

Supported extensions:

- `.ttf`
- `.otf`
- `.ttc`
- `.otc`

### Method 1 — UI upload

Use **Fonts → Ajouter une police custom**.

The server validates the file with fontconfig, limits uploads to 20 MiB, sanitizes the filename, saves it to the persistent folder and refreshes fontconfig.

### Method 2 — folder drop

Copy a font directly into the Custom folder. The catalogue scans this folder directly, so a dropped font becomes visible without rebuilding the container.

## Licensing

Do not commit private/commercial font binaries to the public repository. If a Canva design uses a licensed font, keep it only in the production Custom directory when your license permits local use, otherwise map it to a redistributable substitute in the show JSON.

## API

```text
GET    /v1/fonts
POST   /v1/fonts
DELETE /v1/fonts/{filename}
```

The POST request is multipart with a field named `font`.
