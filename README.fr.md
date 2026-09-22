# CoverForge

**L'IA crée le key art. CoverForge le rend publiable.**

Compositeur Rust déterministe + styles éditables pour thumbnails de podcasts et covers sociales.

[English](README.md) · [简体中文](README.zh-CN.md)

## Pourquoi

Les modèles image savent créer une scène, mais sont mauvais pour répéter exactement typo, logos et branding. CoverForge sépare les rôles :

1. génération d'un **key art natif au bon ratio** ;
2. ajout déterministe de la typo, du logo et des couleurs ;
3. toute l'identité d'une émission dans **un seul JSON**.

Pas de faux `16:9 → fond flou → image incrustée`.

## v0.3

- ⚡ Rust + Axum + resvg
- 🧩 1 JSON par émission avec tous les formats
- 🖥️ éditeur Svelte
- 🔤 gros pack de polices libres
- ⬆️ upload de polices custom dans l'UI
- 📁 dépôt direct de polices dans le dossier Custom
- 🐳 Docker Compose
- 🔌 API + compatibilité avec l'automatisation existante
- 💾 sauvegardes atomiques des templates

## Formats

| Clé | Sortie |
|---|---:|
| `youtube` | 3840×2160 |
| `square` | 1200×1200 |
| `feed` | 1080×1350 |
| `vertical` | 2160×3840 |
| `acast` | 3000×3000 |

Le principe est de générer 16:9 / 1:1 / 4:5 / 9:16 nativement, puis de laisser CoverForge appliquer le branding exact.

## Un JSON par émission

Le JSON réunit palette, polices, notes visuelles et layouts de tous les formats. Voir [docs/SHOW_STYLES.md](docs/SHOW_STYLES.md).

## Polices

Un catalogue de polices libres est inclus. Les polices custom peuvent être uploadées dans l'UI ou déposées directement dans le dossier persistant. Les polices commerciales/privées ne sont jamais commitées. Voir [docs/FONTS.md](docs/FONTS.md).

## Règles simples

- key art au ratio final ;
- typo et logos dans CoverForge ;
- une émission = un fichier style ;
- aucun asset privé dans Git ;
- modifier un template ne doit pas imposer un rebuild.

## Docker

```sh
docker compose up -d --build
curl -fsS http://127.0.0.1:3099/health
```

## Licence

MIT
