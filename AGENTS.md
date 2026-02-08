# Spire Backend

This file provides guidance on how to work with Spire, a high-throughput MMORPG written in Rust.

## Project Structure

- **game-server**: A stateful server that runs the game logic.
- **lobby-server**: A stateless server that manages the authentication.
- **control-server**: (Not implemented yet.)
- **protocol**: Protocol schema definitions and its auto-generation helpers. The schemas in `protocol/inner` are shared with the client.
- **db**: Database (PostgreSQL) schema definitions and helpers. The schemas are defined in `db/shcema`.
- **data**: Static game data definitions and its auto-generation helpers. The schemas in `data/inner` are shared with the client.
- **util**: Utility functions shared along the backend.

## Main flow

- Run `find docs/ -name "*.md" | sort` to see available docs.
- Read docs that may help to solve the current task.
- Read `compose.yaml` and `compose.local.yaml` to understand how the backend is composed.
- Start work on the task.
- Use skills and commands that may help to solve the current task.
