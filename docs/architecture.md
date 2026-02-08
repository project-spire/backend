# Architecture

Spire is a high-throughput MMORPG backend written in Rust. It is composed of multiple services orchestrated via Docker Compose, communicating through a shared PostgreSQL database and a common protocol layer.

## Services

### Game Server

A stateful server that runs the game simulation. It uses the actor model (Actix) for concurrency, Bevy ECS for game logic, and QUIC (Quinn) for client networking. Each game zone is an independent actor with its own ECS world and tick loop.

See [game-server.md](game-server.md) for details.

### Lobby Server

A stateless gRPC server (tonic) that handles authentication and character management. It exposes three services:

- **DevAuth** (unauthenticated, dev-mode only) - Issues dev accounts and JWT tokens for testing.
- **Characters** (authenticated) - List, create, and delete player characters.
- **SteamAuth** (not implemented) - Planned Steam authentication integration.

All authenticated endpoints use a JWT (HS256) middleware interceptor that validates tokens from the `authentication` gRPC metadata header.

### Control Server

Not implemented yet. Planned for administrative operations.

### Database

PostgreSQL with the following tables:

| Table | Purpose |
|---|---|
| `account` | User accounts (id, created_at) |
| `dev_account` | Dev-mode accounts linked to an account |
| `character` | Player characters with identity, growth, world position, resources, and assets |
| `character_path` | Learned paths/classes per character |
| `character_talent` | Learned talents per character |
| `item` | Inventory items with data reference, count, level, bound flag, and JSON attributes |

Custom PostgreSQL types: `vector3` (x, y, z floats), `location` (floor + id), `race` (enum, auto-generated from data).

The `db` crate uses Diesel with diesel-async and deadpool connection pooling.

## Shared Crates

### Protocol

Protocol Buffers with a custom 4-byte binary header (2 bytes length + 2 bytes protocol ID). Protocols are organized into categories:

| Category | ID Range | Examples |
|---|---|---|
| auth | 1-2 | Login, LoginResult |
| net | 100-104 | Ping, Pong, ZoneTransfer |
| tool | 200-201 | Cheat, CheatResult |
| play | 10000-10010 | MovementCommand, MovementSync, SkillUse, EntitySpawn |
| social | 20000-20003 | PartyCreate, PartyInvite |

Game protocols use the custom binary header for QUIC transport. Lobby protocols use standard gRPC/tonic.

The `protocol/inner/` directory contains schemas shared with the client.

### Data

Static game data defined as JSON schemas sourced from ODS (OpenDocument Spreadsheet) files. Categories:

- **character/** - Race (enum), Path, Talent
- **item/** - Item (abstract), Equipment, Weapon, RandomBox
- **skill/** - Skill (abstract), GenericSkill, ScriptedSkill
- **world/** - BiomeType (enum), Region

The build process auto-generates protobuf messages and SQL types from these schemas. `DataId` and `Link<T>` provide type-safe references between data entries. The `data/inner/` directory contains schemas shared with the client.

### Util

Shared utilities:

- **ID generation** - Snowflake-like 64-bit IDs with two variants: universal (node-aware, 10-bit node + 12-bit sequence) and global (22-bit sequence). Custom epoch: 2025-01-01.
- **Token** - JWT generation and verification (HS256) with Claims (account_id, issue, expire).
- **Grid** - 2D grid container with neighbor iteration (cardinal/diagonal) and bounds checking.
- **Rate limiter** - Token bucket algorithm for ingress rate limiting.
- **Interval counter** - Sliding window for performance monitoring (e.g., FPS).
- **IO** - File reading utilities.

## Deployment

Services are containerized with multi-stage Docker builds using cargo-chef for dependency caching. The compose stack includes:

- `game-server` - Depends on `db`, exposes game port and control port.
- `lobby-server` - Depends on `db`, exposes lobby port.
- `db` - PostgreSQL with health checks and persistent volume.
- `db-admin` (optional, `admin` profile) - pgAdmin for database inspection.

TLS certificates and keys are managed as Docker secrets. Configuration uses environment variables with the `SPIRE_` prefix, with secrets loaded from files.

The local development overlay (`compose.local.yaml`) adds a bridge network and assigns hostnames (e.g., `game.spire.localhost`, `lobby.spire.localhost`).