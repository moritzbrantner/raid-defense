# Repository agent guidance

`raid-defense` is a deterministic grid-based tower-defense/maul game built around an economy-versus-defense tradeoff. The repository has one authoritative game implementation: the Rust workspace under `crates/`.

## Authority

- `crates/raid-defense-core` owns ECS entity/component state, resources, population, housing, production, logistics, spending, theft, game rules, command validation, grid occupancy, pathfinding, deterministic simulation, replay, and checksums.
- `crates/raid-defense-wasm` is a translation boundary only. It may serialize and map values, but it must not own game rules.
- `visualization/` owns browser input, camera, and 3D presentation only. It must not recompute authoritative game outcomes.
- The gameplay plane is a 2D grid. Rendering may be fully 3D, but visual transforms must derive from authoritative Rust positions.

## Economy and logistics invariants

- The Town Hall is the authoritative settlement store. Do not add browser/global shadow resource counters.
- Wood is the first material resource. Towers, tower upgrades, Sawmills, and Houses spend stored Town Hall wood.
- Sawmills produce wood into bounded **local** storage. Production does not directly increase Town Hall wood.
- People are real ECS entities with bounded cargo. They travel from the Town Hall to Sawmills, pick up wood, and physically carry it back before that wood becomes spendable.
- Combat kills do not create wood. Production plus logistics remain the economic authority.
- Raiders that reach the Town Hall steal stored wood before they damage the Town Hall itself.
- Economic buildings participate in grid occupancy and path shaping just like defensive buildings.
- Building placement must preserve both raider access to the Town Hall and worker access to every active Sawmill/task.
- Extend generic storage/production/logistics components when adding resources or producers rather than introducing unrelated special-case counters.

## Population and progression invariants

- The Town Hall provides the initial population capacity and the initial people.
- Population capacity comes from authoritative `Housing` components.
- Houses are locked until `HOUSE_UNLOCK_COMPLETED_WAVES` waves have actually completed; merely starting wave 10 must not unlock them.
- Building a House increases population capacity and introduces the corresponding people in the same authoritative command transaction.
- People/capacity must remain visible in snapshots/checksums because logistics throughput is gameplay state, not presentation.

## ECS and reuse

- Compose typed component stores inside `raid-defense-core` over reusable low-level kernels such as `collection-kernels::SparseMap<T>` and `SparseSet`.
- Do not make `ecs-lab` a runtime dependency; it is an experiment/reference harness. Promote reusable low-level primitives through `rust-kernels` instead.
- Prefer components and systems over entity-specific inheritance or monolithic per-entity structs as buildings, towers, people, raiders, projectiles, effects, resources, and units grow.
- Keep game-specific components and systems in this repository unless a genuinely cross-repository low-level primitive emerges.

## Determinism

- Every simulation outcome must be a pure consequence of initial seed, authoritative state, and ordered commands.
- Validate a command completely before mutating state. Rejected commands must leave state unchanged.
- Prefer integer or fixed-point representations for authoritative movement, logistics, economy, and combat rules.
- System iteration/order must be deterministic when it can affect outcomes; tie-break with stable entity identifiers where needed.
- Add replay or checksum coverage whenever a new component/system can affect authoritative state.

## Tower-defense invariants

- Building placement happens on the authoritative grid.
- Buildings may shape routes but must not remove all valid routes from any active spawn or moving raider to the Town Hall.
- Spawn gates and Town Hall cells are protected from ordinary building placement.
- Browser path previews, range displays, logistics displays, and economy projections are advisory presentation; Rust remains authoritative for legality, production, transfer, spending, theft, movement, and combat.

## Validation

- Preserve the hosted layer order: fast -> integration -> workflow -> e2e. Do not skip a failed required layer.
- Keep cheap deterministic core checks ahead of browser or hosted acceptance.
- GitHub Pages is the primary integrated browser acceptance surface after merge.
- Do not treat missing, cancelled, or unavailable evidence as green.

## UI

- The main battlefield should be the 3D game world, not explanatory dashboard cards.
- Keep economy/build/raid/progression controls and feedback close to the world state they affect.
- Render people, cargo, and buildings from authoritative snapshots rather than maintaining browser-owned substitutes.
- Keep browser state interaction-focused; the Rust core remains the game-state authority.
