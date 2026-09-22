# Show styles

CoverForge v0.3 uses **one JSON file per show**.

A show file contains brand identity, palette, fonts and every output layout.

Production layout:

```text
Templates/
├── arezki.json
├── cf.json
├── cp.json
├── dpafm.json
├── lcfp.json
└── ur2w.json
```

Each file contains five formats:

- `youtube` — 3840×2160
- `square` — 1200×1200
- `feed` — 1080×1350
- `vertical` — 2160×3840
- `acast` — 3000×3000

## Why one file per show

A podcast identity is a system, not five unrelated files. One show JSON gives one source of truth for palette, typography, notes and all placements. It also makes new-show creation and review much simpler.

## Native key art

CoverForge does not turn one widescreen image into every social format. The preferred pipeline is:

```text
editorial direction
    ↓
native 16:9 key art
native 1:1 key art
native 4:5 key art
native 9:16 key art
    ↓
CoverForge typography / logo / branding
```

Acast normally reuses the native square key art with its own 3000×3000 final layout.

## Current show grammar

### CF — Chougar Free

Derived from recent channel thumbnails such as Rebut de Presse, La Fin de l'Algorithme and episode 70:

- energetic editorial collage;
- expressive faces;
- episode-specific environment;
- large white title with strong black outline;
- slight title rotation;
- yellow/cyan accent;
- show mark secondary to episode art.

### DPAFM — Dernier Podcast Avant la Fin du Monde

Derived from Apps de dating, Le Ski, Le Nouvel An and Noël:

- cinematic real-world scenario first;
- people integrated into the action rather than isolated portraits;
- physical visual joke tied to episode content;
- organic hand-drawn/display title;
- large black outline;
- show logo anchored low-left;
- apocalyptic / absurd treatment when supported by episode content.

### LCFP — Le Crime Farpait

Derived from Le Clown Tueur and P*rno Mortel:

- tabloid/parody true-crime composition;
- very large characters;
- crime-specific environment;
- yellow/red/white typography;
- show logo upper-left;
- brutal short title near lower-left;
- avoid generic evidence-board imagery unless the episode actually calls for it.

### CP — Conspi Passion

- pulp/conspiracy-cinema;
- concrete episode-specific clues;
- acidic contrast;
- condensed typography;
- avoid automatic cork-board/red-string cliché.

### UR2W

- abrasive pop/culture collage;
- dominant portraits;
- pink/cyan accents;
- intentionally less institutional/polished.

### AREZKI

- direct stand-up/editorial identity;
- strong subject-driven visual;
- compressed readable typography;
- no generic influencer/corporate aesthetic.

## Editing

The UI edits the complete show JSON and writes it atomically.

Production path on Cloud9:

```text
/srv/storage/production/active/Assets/CoverForge/Templates/
```

No rebuild is required after a template edit.
