# Repository agent guidance

`raid-defense` is a deterministic grid-based tower-defense/maul game. The repository has one authoritative game implementation: the Rust workspace under `crates/`.

## Authority

- `crates/raid-defense-core` owns ECS entity/component state, game rules, command validation, grid occupancy, pathfinding, deterministic simulation, replay, and checksums.
- `crates/raid-defense-wasm` is a translation boundary only. It may serialize and map values, but it must not own game rules.
- `visualization/` owns browser input, camera, and 3D presentation only. It must not recompute authoritative game outcomes.
- The gameplay plane is a 2D grid. Rendering may be fully 3D, but visual transforms must derive from authoritative Rust positions.

## ECS and reuse

- Compose typed component stores inside `raid-defense-core` over reusable low-level kernels such as `collection-kernels::SparseMap<T>` and `SparseSet`.
- Do not make `ecs-lab` a runtime dependency; it is an experiment/reference harness. Promote reusable low-level primitives through `rust-kernels` instead.
- Prefer components and systems over entity-specific inheritance or monolithic per-entity structs as tower, raider, projectile, effect, and unit populations grow.
- Keep game-specific components and systems in this repository unless a genuinely cross-repository low-level primitive emerges.

## Determinism

- Every simulation outcome must be a pure consequence of initial seed, authoritative state, and ordered commands.
- Validate a command completely before mutating state. Rejected commands must leave state unchanged.
- Prefer integer or fixed-point representations for authoritative movement and combat rules.
- System iteration/order must be deterministic when it can affect outcomes; tie-break with stable entity identifiers where needed.
- Add replay or checksum coverage whenever a new component/system can affect authoritative state.

## Tower-defense invariants

- Tower placement happens on the authoritative grid.
- Buildings may shape routes but must not remove all valid routes from any active spawn or moving raider to the town.
- Spawn gates and town cells are protected from ordinary tower placement.
- Browser path previews or range displays are advisory presentation; Rust remains authoritative for legality and combat.

## Validation

- Preserve the hosted layer order: fast -> integration -> workflow -> e2e. Do not skip a failed required layer.
- Keep cheap deterministic core checks ahead of browser or hosted acceptance.
- GitHub Pages is the primary integrated browser acceptance surface after merge.
- Do not treat missing, cancelled, or unavailable evidence as green.

## UI

- The main battlefield should be the 3D game world, not explanatory dashboard cards.
- Keep build/wave controls and feedback close to the world state they affect.
- Keep browser state interaction-focused; the Rust core remains the game-state authority.
