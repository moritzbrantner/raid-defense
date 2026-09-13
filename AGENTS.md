# Repository agent guidance

`raid-defense` is a deterministic grid-based tower-defense/maul game built around an economy-versus-defense tradeoff. The repository has one authoritative game implementation: the Rust workspace under `crates/`.

## Authority

- `crates/raid-defense-core` owns ECS entity/component state, resources, population, housing, production, logistics, spending, theft, game rules, command validation, grid occupancy, pathfinding, deterministic simulation, replay, and checksums.
- `crates/raid-defense-wasm` is a translation boundary only. It may serialize and map values, but it must not own game rules.
- `visualization/` owns browser input, camera, 3D presentation, and explanatory reference content only. It must not recompute authoritative game outcomes.
- The gameplay plane is a 2D grid. Rendering may be fully 3D, but visual transforms must derive from authoritative Rust positions.

## Game-rules architecture

- `crates/raid-defense-core/src/rules.rs` defines the typed, authoritative `GameRules` schema, validation, rule queries, and deterministic rules fingerprint. It must not become a second simulation implementation.
- `crates/raid-defense-core/src/default_rules.rs` is the single source of ordinary standard-profile balance values: costs, health, production, population/logistics capacity, tower level stats, raid scaling, unlock thresholds, and day/night cadence.
- Every `GameState` owns one immutable `GameRules` value. Simulation systems must read gameplay tuning from `self.rules`; do not introduce new balance literals, duplicated lookup tables, or progression formulas inside systems.
- `GameState::new` uses `STANDARD_RULES`. Use validated custom rules when testing or intentionally creating another balance profile.
- Rules are part of deterministic identity. If a rule can affect authoritative outcomes, it must be represented in `GameRules` and in its fingerprint/checksum binding.
- WASM/browser snapshots expose values from the active state's rules. Do not duplicate default costs or unlock thresholds in adapters or presentation code.
- Keep structural engine invariants separate from balance rules. Grid dimensions, fixed-point cell representation, stable iteration/tie-breaking, pathfinding neighbor order, and serialization authority are deterministic engine contracts rather than ordinary tuning knobs.
- Prefer adding a typed rule field over adding a new global constant when designers may plausibly tune the value later. Add a new structural engine constant only when changing it would alter representation/topology rather than balance.
- Rule profiles must validate fail-closed before a configurable game starts. Do not silently clamp malformed profiles into a different game.

## Economy and logistics invariants

- The Town Hall and Storage Houses are authoritative settlement storage nodes. `GameState::wood()` and capacity represent the aggregate authoritative settlement inventory; do not add browser/global shadow resource counters.
- Settlement spending must operate on authoritative stored wood in a deterministic order. Tower upgrades and ordinary building purchases consume settlement storage through Rust commands.
- Sawmills harvest finite nearby Forest entities into bounded **local** storage. Production does not directly increase spendable settlement wood.
- People are real ECS entities with bounded cargo. They physically carry wood from Sawmills into reachable settlement storage before that wood becomes spendable.
- Tower placement creates an inactive construction site. Workers withdraw material from stocked settlement storage, haul it to the site, and only the Rust core activates the tower after the full build requirement has been delivered.
- Storage selection, worker assignment, pathfinding, pickup, delivery, construction completion, and resource depletion are authoritative Rust/ECS behavior. Presentation may visualize them but must not predict or complete them independently.
- Combat kills do not create wood. Production plus logistics remain the economic authority.
- Raiders target the nearest reachable settlement storage that currently contains wood. They may retarget as storage changes; the Town Hall remains the fallback target, and Town Hall damage occurs only through authoritative raid resolution.
- Economic buildings participate in grid occupancy and path shaping just like defensive buildings.
- Building placement must preserve required raider routes and worker access to active logistics/construction tasks.
- Extend generic storage/production/logistics components when adding resources or producers rather than introducing unrelated special-case counters.

## Population and progression invariants

- The Town Hall provides the initial population capacity and initial people according to the active rules profile.
- Population capacity comes from authoritative `Housing` components.
- House availability is based on **completed** waves through the active rules profile; merely starting a threshold wave must not unlock it early.
- Building a House increases population capacity and introduces its configured people in the same authoritative command transaction.
- People/capacity must remain visible in snapshots/checksums because logistics throughput is gameplay state, not presentation.

## Day/night invariants

- Day/night timing and automatic raid cadence are authoritative Rust rules, not browser timers.
- The standard profile starts a raid after 600 peaceful simulation ticks and resets the day after the active wave completes.
- The standard profile pauses production and carrier logistics while raiders are active. Alternative validated profiles may change that rule without changing system code.
- Presentation may display phase/countdown state but must not decide when a wave starts or whether economic systems advance.

## ECS and reuse

- Compose typed component stores inside `raid-defense-core` over reusable low-level kernels such as `collection-kernels::SparseMap<T>` and `SparseSet`.
- Do not make `ecs-lab` a runtime dependency; it is an experiment/reference harness. Promote reusable low-level primitives through `rust-kernels` instead.
- Prefer components and systems over entity-specific inheritance or monolithic per-entity structs as buildings, towers, people, raiders, projectiles, effects, resources, and units grow.
- Keep game-specific components and systems in this repository unless a genuinely cross-repository low-level primitive emerges.

## Determinism

- Every simulation outcome must be a pure consequence of initial seed, immutable rules, authoritative state, and ordered commands.
- Validate a command completely before mutating state. Rejected commands must leave state unchanged.
- Prefer integer or fixed-point representations for authoritative movement, logistics, economy, and combat rules.
- System iteration/order must be deterministic when it can affect outcomes; tie-break with stable entity identifiers where needed.
- Add replay or checksum coverage whenever a new component/system or rule can affect authoritative state.

## Tower-defense invariants

- Building placement happens on the authoritative grid.
- Buildings may shape routes but must not remove all valid routes from any active spawn or moving raider to the Town Hall.
- Spawn gates and Town Hall cells are protected from ordinary building placement.
- Browser path previews, range displays, logistics displays, economy projections, and help/wiki text are advisory presentation; Rust remains authoritative for legality, production, transfer, spending, theft, movement, construction, and combat.

## Validation

- Preserve the hosted layer order: fast -> integration -> workflow -> e2e. Do not skip a failed required layer.
- Keep cheap deterministic core checks ahead of browser or hosted acceptance.
- GitHub Pages is the primary integrated browser acceptance surface after merge.
- Do not treat missing, cancelled, or unavailable evidence as green.

## UI

- The main battlefield should be the 3D game world, not explanatory dashboard cards.
- Keep economy/build/raid/progression controls and feedback close to the world state they affect.
- Render people, cargo, forests, storage, construction sites, and buildings from authoritative snapshots rather than maintaining browser-owned substitutes.
- Keep browser state interaction-focused; the Rust core remains the game-state authority.
- In-game wiki/help content is explanatory only. Do not encode legality or progression logic there, and do not duplicate tunable numeric rule values in static prose. Read live values from authoritative snapshots when a number is useful, or explain the mechanic without a number.
