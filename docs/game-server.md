# Game Server

The game server is a stateful server that runs the game simulation. It combines the actor model (Actix) for concurrency, Bevy ECS for game logic, and QUIC (Quinn) for client networking.

## Startup

1. Initialize tracing and load TLS certificates.
2. Load configuration from config files and environment variables.
3. Initialize the ID generator with the configured node ID.
4. Connect to PostgreSQL.
5. Load static game data from spreadsheets.
6. Start the actor system: Authenticator, GameListener, Gateway, PartyManager, GuildManager.
7. Create the default zone (Zone 0) and begin the game loop.

## Connection Flow

```
Client connects via QUIC
        |
  GameListener accepts connection
        |
  Authenticator validates JWT token
        |
  Gateway loads player data from DB
        |
  Zone receives PlayerTransfer
        |
  Zone spawns ECS entity with Session + PlayerData
        |
  Zone sends ZoneTransfer protocol to client
```

## Networking

See [game-server/networking.md](game-server/networking.md) for protocol details.

The server uses QUIC (Quinn) with TLS 1.3 for all client communication. Each session has two tokio tasks:

- **Receive task** - Reads from the QUIC stream, decodes protocol headers/bodies, and routes messages. Local protocols are buffered in a crossbeam channel for processing during the zone tick. Global protocols are handled immediately.
- **Send task** - Batches and writes encoded protocols to the QUIC stream.

Unreliable messaging (e.g., movement sync) uses QUIC datagrams. Idle timeout is 30 seconds. Ingress rate limiting is configurable per session (protocols/second and bytes/second).

## Zones

Each zone is an Actix actor with its own Bevy ECS World and Schedule. Zones are the core unit of game simulation, running an independent tick loop (default 50ms interval, configurable).

### Tick Loop

Each tick:

1. **Collect protocols** - Drain ingress protocol channels from all sessions.
2. **Run ECS schedule** - Execute registered systems in order:
   - `movement` - Process movement commands and sync states to clients.
   - `session` - Clean up finished sessions.
   - `task` - Process async task callbacks.
3. **Update time** - Advance the tick counter and record delta time.

### Regions

Regions group multiple zones together. The `RegionGenerator` uses seeded RNG for deterministic world generation:

- Configurable zone count (8-15 per region).
- Zone shapes: 1x1, 2x1, 1x2, L-shaped, T-shaped patterns.
- Flood-fill algorithm from center on a 16x16 grid.

## Player

A player entity is composed of these ECS components:

| Component | Description |
|---|---|
| `Session` | Network session (QUIC connection, protocol channels, stop signal) |
| `Character` | Core identity loaded from DB (id, name, race) |
| `Transform` | Position (Point3) and direction (UnitVector2) |
| `PathTree` | Skill/class tree (nodes, points) |
| `Equipment` | Equipped items |
| `Inventory` | Owned items |
| `Resource` | Health, Mana, Stamina, Shield |
| `Status` | Combat, Movement, Crafting, Growth states |
| `SkillSet` | Active skills |
| `Sense` | Vision and audition perception |

### Movement

1. Client sends `MovementCommand` with timestamp and direction.
2. Handler pushes command to the `MovementCommands` queue.
3. Each tick: `process_commands` drains the queue, interpolates between command timestamps, and updates `Transform`.
4. `sync_movement_states` sends `MovementSync` to all clients via QUIC datagrams.

Movement states: Walking, Running, Rolling, Jumping. Movement is disabled when the player is in a "Bound" state (stuns, roots).

Speed values (walk/run) use `BasedValue<Speed>` for base + modifier tracking.

## Protocol Handlers

Protocols are split into two categories:

- **Local** - Processed during the zone tick with full ECS world access. Examples: MovementCommand, SkillUse, ItemPickup.
- **Global** - Handled immediately outside the tick loop. Examples: Ping/Pong, ZoneTransferReady.

Handler modules:

| Module | Protocols |
|---|---|
| `handler/net` | Ping, Pong, ZoneTransferReady |
| `handler/play` | Movement, skills, item pickup |
| `handler/social` | Party create/invite, guild operations |
| `handler/tool` | Cheat commands (if enabled) |

## Task System

An async task queue for entities that need to perform work outside the tick loop (e.g., database queries):

- **Serial** - Waits for the previous task to complete before starting.
- **Parallel** - Spawns immediately, but callbacks are still synchronized to the tick loop.

Tasks run via tokio. Callbacks (`on_complete`, `on_success`) execute within the ECS world after task completion.

## Actors

| Actor | Responsibility |
|---|---|
| `GameListener` | Accepts incoming QUIC connections |
| `Authenticator` | Validates JWT tokens, extracts account/character IDs |
| `Gateway` | Routes players to zones, loads player data from DB, tracks character-to-zone mappings |
| `Zone` | Runs ECS simulation for a portion of the game world |
| `PartyManager` | Manages party creation and invitations |
| `GuildManager` | Manages guild operations |

## Configuration

Configuration is loaded from config files (TOML) and environment variables (`SPIRE_` prefix):

| Category | Settings |
|---|---|
| `app` | Data directory, cheat mode, zone tick interval |
| `auth` | Login timeout, TLS cert/key paths, JWT token key file |
| `db` | Database connection (host, port, user, password, name) |
| `net` | Node ID, game port, control port, application protocol name, rate limits |