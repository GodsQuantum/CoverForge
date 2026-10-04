# Modèles de marque / 品牌模板

CoverForge utilise **un document JSON par modèle de marque ou projet réutilisable**. Le JSON est la source de vérité : Fabric.js sert à l'édition interactive, tandis que Rust/resvg effectue le rendu de production déterministe.

## Structure

Un modèle peut contenir :

- `brand` : nom, description visuelle, palette, polices, logos et notes ;
- `formats` : mises en page indépendantes par ratio ou plateforme ;
- `layers` : image, texte et rectangle ;
- des variables comme `{{background}}`, `{{logo}}`, `{{title}}`, `{{subtitle}}` ou `{{badge}}`.

Exemple générique livré avec l'application :

```text
templates/
└── starter-brand.json
```

`starter-brand` fournit :

- `youtube` — 1920×1080 ;
- `square` — 1080×1080 ;
- `feed` — 1080×1350 ;
- `vertical` — 1080×1920 ;
- `landscape` — 1200×628.

Chaque format possède sa propre pile de calques afin que le cadrage, le logo et la typographie puissent être adaptés au ratio sans modifier les autres sorties.

## Priorité des modèles

CoverForge distingue deux répertoires :

1. `COVERFORGE_TEMPLATE_DIR` — modèles utilisateur persistants et éditables ;
2. `COVERFORGE_DEFAULT_TEMPLATE_DIR` — modèles intégrés en lecture seule dans l'image Docker.

Un modèle utilisateur portant le même identifiant qu'un modèle intégré **prend toujours la priorité**. Une sauvegarde écrit uniquement dans le répertoire utilisateur. Cette règle permet de livrer `starter-brand` sans toucher aux modèles de production existants ni ajouter un mount.

## Calques

Tous les calques partagent :

- `id` — clé stable pour l'API ;
- `name` — libellé humain ;
- `visible` et `locked` ;
- `frame.x/y/width/height` — coordonnées normalisées.

### Image

Une image définit notamment :

- `source` — chemin ou variable ;
- `fit: "cover" | "contain"` ;
- `focal_x`, `focal_y` ;
- `opacity`.

Les uploads CoverForge et les chemins autorisés peuvent être utilisés immédiatement comme source.

### Texte

Un texte peut définir :

- `font_family`, `font_weight`, `font_style` ;
- `font_size`, `min_font_size`, `auto_fit` ;
- `max_lines`, `line_height` ;
- `color`, contour, alignement, rotation et opacité.

Les variables sont découvertes automatiquement depuis les chaînes `{{...}}`.

### Rectangle

Un rectangle définit :

- `fill` ;
- `opacity` ;
- `radius`.

## Métadonnées de format

Les champs suivants sont optionnels et rétrocompatibles :

- `label` ;
- `category` ;
- `platform` ;
- `suffix`.

Les anciens JSON v0.5 sans ces champs restent valides.

## API

Les agents peuvent découvrir les champs d'autofill avec :

```text
GET /v1/templates/{id}/dataset
```

Le preview d'un modèle non sauvegardé utilise exactement le renderer Rust de production :

```text
POST /v1/render/preview
```

Le workflow de branding complet est également exposé par API :

```text
POST /v1/assets
POST /v1/reframe
POST /v1/render/package
```

Le package ZIP contient les sorties sélectionnées et un `manifest.json`. Les uploads sont stockés sous le répertoire de sortie CoverForge, qui reste une racine d'asset sûre sans élargir l'accès au reste du système de fichiers.

## Compatibilité des émissions existantes

Les modèles historiques `cf`, `cp`, `dpafm`, `lcfp`, `arezki` et `ur2w` restent des modèles CoverForge valides. Ils représentent simplement des brand kits spécialisés pour des émissions.

Le chemin Cloud9 actuel reste :

```text
/srv/storage/production/active/Assets/CoverForge/Templates/
```

Aucun renommage, déplacement ou changement de mount n'est requis.

---

# 品牌模板

CoverForge 使用**每个可复用品牌模板/项目一个 JSON 文档**。JSON 是唯一真实来源；Fabric.js 只负责交互编辑，Rust/resvg 负责确定性的生产渲染。

模板可包含：

- `brand`：名称、视觉说明、配色、字体、Logo、备注；
- `formats`：针对不同平台或比例的独立布局；
- `layers`：图片、文字、矩形；
- `{{background}}`、`{{logo}}`、`{{title}}`、`{{subtitle}}`、`{{badge}}` 等变量。

内置的 `starter-brand` 提供 16:9、1:1、4:5、9:16 和 1.91:1 五种常用输出。

## 模板优先级

1. `COVERFORGE_TEMPLATE_DIR`：用户可编辑的持久模板；
2. `COVERFORGE_DEFAULT_TEMPLATE_DIR`：Docker 镜像内置的只读模板。

同名时用户模板优先；保存操作永远只写入用户目录。因此可以升级内置模板而不覆盖生产模板。

旧的节目模板仍完全兼容，只是现在被视为通用品牌模板的一种具体用途。
