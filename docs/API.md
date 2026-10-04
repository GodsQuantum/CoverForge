# CoverForge API

CoverForge expose la même chaîne de production que l'UI : upload → Smart Reframe → variables → rendu multi-format → package ZIP.

L'instance documente sa version courante via :

```text
GET /openapi.json
```

## Santé

```bash
curl -fsS http://localhost:3099/health
```

Réponse :

```json
{"ok":true,"service":"CoverForge","version":"0.6.0","renderer":"rust/resvg"}
```

## Modèles

Lister, lire ou remplacer atomiquement un modèle utilisateur :

```bash
curl -fsS http://localhost:3099/v1/templates
curl -fsS http://localhost:3099/v1/templates/starter-brand
curl -X PUT http://localhost:3099/v1/templates/my-brand \
  -H 'content-type: application/json' --data-binary @my-brand.json
```

Un modèle utilisateur portant le même id qu'un modèle intégré prend la priorité.

## Dataset / autofill

```bash
curl -fsS http://localhost:3099/v1/templates/starter-brand/dataset
```

La réponse décrit les formats et les variables détectées avec leurs types et usages.

## Upload d'asset

Endpoint : `POST /v1/assets`. Champ multipart requis : `asset`.

```bash
curl -X POST http://localhost:3099/v1/assets -F 'asset=@./photo.jpg'
```

La réponse contient l'id, le chemin, l'URL de preview, le MIME, les dimensions et le type d'asset.

Formats raster : JPEG, PNG, WebP. Les SVG sont acceptés uniquement comme logos sécurisés ; scripts, handlers d'événements et références externes sont refusés.

## Smart Reframe

```bash
curl -X POST http://localhost:3099/v1/reframe \
  -H 'content-type: application/json' \
  -d '{"asset":"/path/image.jpg","formats":[{"id":"youtube","width":1920,"height":1080},{"id":"square","width":1080,"height":1080}]}'
```

La réponse contient pour chaque format un crop normalisé, un point focal et un score. Un `focal_override` optionnel permet d'imposer un point focal global.

## Rendu

Modèle sauvegardé :

```bash
curl -X POST http://localhost:3099/v1/render \
  -H 'content-type: application/json' \
  -d '{"template":"starter-brand","variables":{"background":"/path/photo.jpg","title":"Campaign"},"variants":["youtube","square"]}'
```

Preview d'un JSON non sauvegardé :

```text
POST /v1/render/preview
```

Batch jusqu'à 100 jobs :

```text
POST /v1/render/batch
```

## Package multi-format ZIP

```bash
curl -X POST http://localhost:3099/v1/render/package \
  -H 'content-type: application/json' \
  -d '{"template":"starter-brand","variables":{"background":"/path/photo.jpg","title":"Campaign"},"variants":["youtube","square","feed","vertical","landscape"],"output_stem":"campaign"}'
```

Pour un modèle non sauvegardé, utilise `inline_template` à la place de `template`. Exactement une source de modèle doit être fournie.

La réponse contient les assets rendus, `package_url` et un manifest comprenant version, date, modèle, image source, variables, formats, dimensions et noms de fichiers.

Le ZIP contient les images et `manifest.json`. Les noms d'entrée sont assainis et les collisions sont rendues uniques.

## Polices

```text
GET    /v1/fonts
POST   /v1/fonts
DELETE /v1/fonts/{filename}
```

## Asset preview

```text
GET /v1/asset?path=/allowed/path/image.png
```

Seuls les fichiers sous les racines autorisées ou le répertoire de sortie CoverForge sont servis.

## Compatibilité AutoPublisher

Endpoint historique :

```text
POST /api/generate
```

Exemple de payload :

```json
{"template_name":"DPAFM - YT","output_filename":"episode-48.png","bg_image":"/srv/storage/background.jpg","fields":{"title":"LE STAND-UP EST MORT","PodLogo":"/srv/storage/logo.png"}}
```

La compatibilité legacy est volontairement conservée pendant la transition vers le moteur générique de branding.
