# Replay contract

Raid Defense replays are authoritative input logs, not histories of materialized game state.

## Authority

A replay contains only the deterministic inputs required to recompute a match:

- the initial seed and scenario/rules configuration;
- `recorded_through_tick`, the deterministic simulation-time cursor for an in-progress save;
- accepted player actions, each identified by simulation tick, contiguous sequence number, player id, and action payload.

Simulation ticks are not player actions and are not persisted as replay entries. Derived events, ECS/entity snapshots, resource totals, wave/progression values, projectiles, pathfinding results, AI decisions, and checksums are not replay authority.

## Reconstruction

Reconstruction starts from the deterministic initial configuration, advances the simulation to each recorded action tick, applies actions in sequence order, then advances to `recorded_through_tick`.

The Rust core exposes replay reconstruction separately from snapshot creation. A snapshot is therefore a function of reconstructed deterministic state rather than a prerequisite for replay.

## Derived caches and integrity

Snapshots and UI progress summaries may be cached to make seeking, reconnect, and menus faster. They are disposable: deleting them must not remove the ability to reconstruct the game from the replay.

A checksum is also derived rather than replay authority, but resume must still fail closed if persisted inputs were corrupted or an incompatible simulation would reconstruct a different state. For this reason persistence stores a separate integrity receipt bound to the replay seed, cursor tick, action count, and reconstructed checksum. The receipt validates the replay; it does not replace any input needed to recompute state.

Legacy save formats are migrated by converting persisted tick runs into action timestamps, reconstructing the same state, and verifying their historical checksum before the v3 replay is accepted.

## Multiplayer seam

The current browser uses player id `0`. The replay envelope already records player identity so a hosted multiplayer runtime can order actions from multiple players without moving game-rule authority into the server. The game remains authoritative for whether an action is accepted and for every resulting state transition.
