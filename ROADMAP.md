# Raid Defense roadmap

The roadmap is vertical: every milestone should leave GitHub Pages more playable while preserving deterministic Rust/ECS authority.

## A. Grid-maul foundation

- [x] Replace the earlier province/turn prototype with a real-time grid tower-defense model.
- [x] Keep gameplay on an integer 2D grid while exposing 3D-ready transforms.
- [x] Put authoritative entity/component state directly in `raid-defense-core`.
- [x] Use reusable `rust-kernels` sparse collection primitives rather than depending on `ecs-lab` as a game framework.
- [x] Add a central 3×3 town and four protected edge spawn gates.
- [x] Add deterministic tower placement, occupancy, gold cost, and transactional rejection.
- [x] Allow maul-style path shaping while rejecting placements that seal a route.
- [x] Add fixed-point raider movement from all four edges toward the town.
- [x] Add tower targeting, damage, health, kills, rewards, and town damage.
- [x] Render the playable grid, town, towers, and raiders in 3D through React Three Fiber.
- [x] Cover the real WASM/3D client with browser acceptance.

## B. Tower-defense combat depth

- [ ] Add tower archetypes with distinct range, fire rate, damage, and cost.
- [ ] Add deterministic tower upgrades and selling.
- [ ] Add projectile entities for attacks that should travel rather than resolve instantly.
- [ ] Add splash, slow, armor, resistances, and status-effect components/systems.
- [ ] Add clear range/path/build previews without moving legality into the browser.

## C. Waves and raiders

- [ ] Add timed spawn schedules rather than one simultaneous raider per edge.
- [ ] Add larger waves and deterministic composition recipes.
- [ ] Add fast, tank, swarm, ranged, boss, and eventually flying raider archetypes.
- [ ] Add explicit wave completion, preparation windows, and escalating rewards.
- [ ] Add raider attack behavior for relevant buildings/objectives without turning pathfinding into browser state.

## D. ECS scale and performance

- [ ] Add projectile/effect pools as entity populations grow.
- [ ] Measure targeting and movement system costs with representative large waves.
- [ ] Introduce spatial-query acceleration only when measurements justify it.
- [ ] Keep deterministic replay/checksum coverage across component lifecycle and system ordering.
- [ ] Use ECS Lab to compare storage/system strategies, then promote only reusable low-level primitives through `rust-kernels`.

## E. 3D presentation

- [ ] Replace primitive meshes with reproducible tower, town, raider, projectile, and terrain assets.
- [ ] Add animation and effects driven from authoritative events/state.
- [ ] Add camera presets and mobile-friendly interaction without coupling camera state to simulation.
- [ ] Add terrain dressing outside the build plane while keeping the gameplay grid legible.
- [ ] Preserve a performant fallback path for browsers/devices that cannot run the richest rendering path.

## F. Game progression

- [ ] Add build menus, unlocks, tower tech trees, and wave goals.
- [ ] Add deterministic scenarios/maps with different grid shapes, gates, and town layouts.
- [ ] Add score/replay persistence only after the combat loop is strong.
- [ ] Add multiplayer/co-op authority boundaries only after deterministic local simulation is mature.

## G. Repository hardening

- [ ] Generate browser contract types from Rust instead of maintaining handwritten DTO types.
- [ ] Keep dependency revisions immutable and update them through validated PRs.
- [ ] Continue fast → integration → workflow → e2e promotion and Pages acceptance.
- [ ] Add convergence checks as the new ECS/game boundaries stabilize.
