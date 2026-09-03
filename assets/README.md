# Supernova Starter Assets

This folder ships with every Supernova release so you can start building
games immediately. Both the **Engine** and the **Editor** installers include
a full copy of these assets and point to them at runtime.

## What's included

| Folder       | Contents                                                                        |
|--------------|---------------------------------------------------------------------------------|
| `config/`    | Engine, input, audio, and physics defaults (TOML)                               |
| `scenes/`    | Ready-to-open starter scenes: `game_ready.json` and `empty_world.json`          |
| `materials/` | Physically-based materials (default, player, ground, brick, grass) with PBR maps|
| `textures/`  | Real, generated textures: checker, grass, brick, dirt, rock PBR pack, player & coin sprites, sky |
| `shaders/`   | `pbr.wgsl` (PBR lighting), `unlit.wgsl`, `sprite.wgsl` (2D)                     |
| `sprite/`    | 2D sprite metadata (`player_idle`, `coin`)                                      |
| `audio/`     | A pentatonic BGM loop and designed SFX: jump, coin, damage, powerup, click, hover |

## Quick start

1. Install Supernova (Engine or Editor).
2. Point the asset resolver at this directory (default `install_dir/assets`).
3. Open `scenes/game_ready.json` in the Editor, or load
   `assets/scenes/game_ready.json` from the Engine demo.
4. Duplicate any asset and edit it — the `AssetManager` supports hot reload.

## Notes on the starter content

- **Textures** are procedurally generated at 256x256 (except sprites at 64x64)
  and stored as lossless PNGs. The `rock_*` set is a proper albedo/normal/roughness
  PBR pack. Replace them with real authored art when you start production.
- **Audio** is a simple pentatonic BGM loop plus synthesized SFX with real
  envelopes — perfectly usable for prototypes.
- **Scenes** reference the bundled materials, textures, sprites, and audio, so
  `game_ready.json` opens fully assembled.

## Licensing

All assets are original and are released under the same license as the
Supernova project. You may use them in commercial and non-commercial projects.
