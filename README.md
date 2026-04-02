# Alembic Client

A 2D multiplayer RPG client built with [Bevy](https://bevyengine.org/) in Rust. Connects to an Alembic game server over a custom binary TCP protocol and renders a tile-based world with animated player characters.

## Overview

Alembic is a tile-based online RPG. This repository is the **client** — it handles rendering, input, networking, and asset management. The server (a separate Elixir project) is authoritative over the game world, player positions, and asset manifests.

## Features

- **Custom binary protocol** over TCP — handshake, auth, heartbeat, movement, viewport updates
- **Server-authoritative movement** with client-side prediction and smooth lerp interpolation
- **Tile-based world rendering** using texture atlases with label-based tile lookup
- **Data-driven character animations** — row/frame/flip data comes from the server manifest, no hardcoded mappings
- **Directional walk and idle states** (N/S/E/W) with horizontal sprite flip for west-facing movement
- **Asset caching** — spritesheets and tilesets are downloaded once and cached locally under `~/.cache/alembic/`
- **Pixel-perfect rendering** via nearest-neighbor texture filtering

## Game Flow

```
Main Menu → Server List → Connecting → World Select → Downloading Assets → Character Select / Create → In Game
```

Each screen is a Bevy `GameState` variant with its own setup, update, and cleanup systems.

## Architecture

### Networking (`src/network/`)

- `connection.rs` — TCP connection management, packet framing, send/receive
- `packets.rs` — binary packet parser (server → client) and packet ID constants
- `events.rs` — `NetworkEvent` and `NetworkCommand` Bevy events that decouple the network thread from game systems
- `systems.rs` — Bevy systems that poll the socket, parse packets, and dispatch events

The packet format is:

```
[4 magic][3 version][2 packet_id][4 payload_length][payload...]
```

### Asset Pipeline (`src/asset_cache.rs`, `src/asset_sync.rs`)

On entering a world, the client fetches a manifest from the server:

```
GET /worlds/{world_id}/manifest
```

The manifest describes all tilesets and spritesheets, including per-animation definitions (row, frame count, timing, flip). Assets are downloaded to a per-server, per-world cache directory and validated against their expected image magic bytes before being written to disk.

Cache layout:
```
~/.cache/alembic/servers/{server_addr}/worlds/{world_id}/
    tiles/
    sprites/characters/
```

Downloads run on a background thread and send results back via an `mpsc` channel polled by a Bevy system.

### Animation (`src/game/animation.rs`)

Animations are built at runtime from the server manifest using `load_animations_from_manifest`. Each `AnimationDef` specifies:

| Field | Description |
|---|---|
| `row` | Spritesheet row |
| `frames` | Number of frames |
| `start_col` | Starting column (default 0) |
| `frame_time` | Seconds per frame (default 0.12) |
| `flip_x` | Mirror horizontally (used for west-facing from east row) |
| `looping` | Whether the animation loops (default true) |

Named animation keys (`walk_south`, `idle_east`, `attack_west`, `dying`, etc.) are mapped to `AnimationState` variants. Adding new animations only requires a server-side manifest update.

### Player (`src/game/player.rs`)

- Input is read locally and sent to the server each tick (with a rate-limiting timer)
- The server confirms or corrects the position
- Visual position lerps toward the server-confirmed target for smooth movement
- Directional idle states are set on key release — the sprite holds the last facing direction

## Building

### Prerequisites

- Rust toolchain (edition 2024)
- A running Alembic game server

### Build

```bash
cargo build
```

For a release build:

```bash
cargo build --release
```

The dev profile uses `opt-level = 1` for the crate and `opt-level = 3` for dependencies, which keeps compile times reasonable while keeping Bevy fast.

### Run

```bash
cargo run
```

The client connects to `127.0.0.1:7777` by default. The server address can be selected from the in-game server list UI.

## Dependencies

| Crate | Purpose |
|---|---|
| `bevy` | Game engine — rendering, ECS, input, asset server |
| `ureq` | Synchronous HTTP for manifest and asset downloads |
| `serde` / `serde_json` | Manifest deserialization |
| `tokio` | Async runtime (networking) |
| `hmac` / `sha2` | Auth challenge/response signing |
| `dirs` | Platform-appropriate cache directory |
| `toml` | Server list configuration |
| `chrono` | Timestamps |

## Project Structure

```
src/
  main.rs                  # App setup, state machine, system registration
  asset_cache.rs           # Manifest types, download, validation, local cache
  asset_sync.rs            # Background download thread, Bevy polling systems
  available_sprites.rs     # Loaded sprite handles and animation defs
  tile_textures.rs         # Tile atlas handles and label lookup
  auth.rs                  # Auth token resource
  character.rs             # Character data types
  servers.rs               # Server list persistence
  network/
    connection.rs          # TCP socket, framing, send queue
    packets.rs             # Binary packet parsing
    events.rs              # NetworkEvent / NetworkCommand
    systems.rs             # Bevy network poll systems
  game/
    mod.rs
    world.rs               # Tile rendering, viewport updates, WorldContext
    player.rs              # Player spawn, movement, input, camera follow
    animation.rs           # AnimationState, Animation, animate_sprites system
  ui/
    main_menu.rs
    server_list.rs
    connecting.rs
    world_select.rs
    downloading_assets.rs
    character_select.rs
    create_character.rs
    in_game.rs
```
