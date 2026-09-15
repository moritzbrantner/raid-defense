# Raid Defense design

## Game direction

Raid Defense is a deterministic grid-based tower-defense/maul game inspired by Warcraft III custom maps. The gameplay plane is a 2D integer grid, while the presentation is fully 3D.

The central tension is **economy versus defense**, but the economy is deliberately physical rather than just a set of counters. The Town Hall and Storage Houses store spendable resources. Forests hold renewable wood. People must travel to Forests, gather real stock, carry it to a Sawmill, and then haul the Sawmill stock into settlement storage before the player can spend it. More economic buildings therefore create a logistics problem rather than passive income.

Buildings also shape the battlefield. Towers, Sawmills, Storage Houses, and Houses occupy grid cells, raiders repath around them, and placement is rejected if it would remove required raider or worker routes.

## Rules and configuration architecture

Gameplay tuning is an explicit input to the simulation rather than a collection of hidden globals.

`rules.rs` defines a typed `GameRules` schema. It groups rules by domain:

- economy — starting storage, forest regrowth, forestry work cadence/batch size, and local buffers;
- population — starting people, capacity, health, movement, and carrying capacity;
- buildings — costs, health, House unlock progression, and population effects;
- towers — build costs, level stats, upgrade costs, health, and level limit;
- raids — wave size, spawn cadence, health/damage scaling, movement, and resource theft;
- cycle — daytime length, pre-raid rally duration, automatic raid cadence, and whether active raids pause the economy.

`default_rules.rs` contains `STANDARD_RULES`, the single ordinary balance table for the shipped game. Systems do not own copies of these values: each `GameState` stores one immutable `GameRules` value and reads from it whenever a rule affects simulation behavior.

This distinction is deliberate. A designer should be able to change a Sawmill cost, worker forestry throughput, House unlock wave, carrier capacity, tower damage, raid scaling, rally duration, spawn cadence, or day duration by changing a rules profile rather than rewriting a system. By contrast, grid dimensions, fixed-point representation, deterministic path-neighbor order, and stable tie-breaking remain engine invariants because changing them alters simulation representation/topology rather than ordinary balance.

Rules validate before a configurable game starts. Invalid profiles fail closed instead of being silently normalized. Every complete rule set also has a deterministic fingerprint, which is included in game checksums so two states produced under different rules cannot accidentally claim the same authoritative identity.

`GameState::new(seed)` remains the standard-game convenience constructor. Custom profiles use the validated rules constructor. This gives tests and future game modes a supported extension point without making JSON/TOML parsing or mod loading part of the core simulation prematurely.

## Resource and logistics model

Wood is the first material resource.

- The Town Hall and Storage Houses own authoritative spendable `ResourceStorage`.
- Forests own renewable bounded `ResourceStorage`; depletion leaves the Forest entity in place so it can regrow.
- Sawmills own a forestry work profile plus bounded local `ResourceStorage`, but they do not generate or remove forest wood by themselves.
- People are ECS entities with `Person` cargo/task state, `Transform`, `Health`, and `Movement`.
- Idle people are deterministically assigned to a reachable stocked Forest only when a reachable Sawmill has room for the resulting batch.
- A forestry worker travels to a Forest-adjacent cell, spends the configured Sawmill work interval gathering a bounded batch, removes that amount from the Forest's real stock, then physically carries the batch to a Sawmill.
- Wood deposited at a Sawmill is still not spendable. A person must pick it up from the Sawmill and carry it to reachable Town Hall/Storage House storage.
- Bounded worker cargo and Sawmill buffers make population and route layout real logistics throughput rather than a passive percentage bonus.
- Towers, upgrades, Sawmills, Storage Houses, and Houses consume settlement wood.
- Raider kills do not mint resources.
- Raiders target reachable settlement storage containing wood, steal from it, and fall back toward Town Hall damage when appropriate.

The existing `sawmill_output` and `sawmill_interval_ticks` balance fields intentionally remain useful after removing passive production: they now describe how much a person can gather in one forestry work cycle and how long that work takes. This keeps scenario tuning stable while removing the hidden autonomous producer.

## Population and progression

Under the standard profile, the settlement begins with two people and population capacity for two, supplied by the Town Hall.

Houses are the first progression-gated economic building. The standard profile makes them available only after **10 waves have completed**. `completed_waves` is tracked separately from the current/last-started `wave`, so starting wave 10 cannot unlock the building early.

A standard House:

- costs wood;
- occupies and path-shapes one grid cell;
- adds authoritative population capacity through a `Housing` component;
- introduces additional people in the same successful command transaction.

The exact cost, unlock threshold, capacity, and number of people are rules-profile data, not House-system constants.

## Day/rally/night cycle

Time pressure is authoritative simulation state. The standard profile has a 600-tick peaceful day (60 seconds at the current 100 ms browser tick cadence). When that countdown expires—or when the player manually starts a raid—Rust begins a **50-tick rally window**, equivalent to about 5 seconds at that cadence.

Starting the rally does not spawn a raider. It recalls people toward the Town Hall, stops assigning new economic/construction jobs, and lets people already carrying material finish that delivery before returning. The rally continues to advance authoritative logistics so returning workers actually move through the world rather than being teleported home.

Only when the full rally expires does Rust increment the wave and spawn the first raider. The remaining raiders enter at the configured authoritative spawn interval in a deterministic seed-and-wave-derived edge order.

The active raid phase lasts for the entire spawn schedule plus the lifetime of all spawned raiders. A temporary gap with no live raider does not end the night, restart the day, allow another raid, or resume economic work. Under the standard profile, worker economy/logistics pause for the active raid after rally. Natural forest regrowth remains independent authoritative world progression. Only when no scheduled or live raiders remain is the wave completed and a fresh daytime period started.

Those policy values live in `RaidRules` and `CycleRules`, while rally/schedule progress itself is checksum-bound `GameState`. The state machine remains deterministic if a future profile changes wave size, rally duration, spawn cadence, day length, manual-only raids, or whether economic work continues during combat.

## Placement interaction

Building placement is presentation-driven input over authoritative Rust validation.

Selecting a Sawmill, Storage House, House, Arrow Tower, or Cannon Tower does **not** issue a build command and does not reuse a previously inspected tile. It arms browser-only placement intent and clears stale tile inspection. While placement is armed, a translucent ghost of the chosen building follows the current battlefield target. Clicking or tapping that tile dispatches the ordinary Rust placement command using exactly that grid coordinate.

The Rust core still decides whether the target is in bounds, occupied, protected, affordable, and route-safe. A rejected placement leaves the simulation unchanged and keeps placement intent active so another tile can be tried.

For touch/accessibility, the UI also preserves a precise coordinate/nudge fallback. It previews and places the same armed ghost; it is optional and does not restore the former tile-first workflow. A fresh game therefore starts with no selected build tile.

## Authority boundary

`raid-defense-core` is the sole authority for ECS state and game outcomes. It owns immutable game rules, entity lifecycle, population, housing, Forest/Sawmill/settlement storage, forestry work, carrier assignment/pathing/cargo transfer, grid occupancy, rally/day/night phase progression, timed raid scheduling, wave completion, building unlocks, raider pathfinding, targeting, projectiles, upgrades, theft, damage, health, command validation, replay, and checksums.

`raid-defense-wasm` is a versioned serialization adapter only. It maps commands/events/snapshots without recomputing rules. Values such as costs, rally duration, and unlock thresholds come from the active `GameState` rules, not duplicate adapter constants.

The browser owns input, camera, ghost placement presentation, and 3D presentation. It requests deterministic ticks but does not decide forestry, carrier assignments, cargo transfers, placement legality, phase transitions, spawn timing, unlocks, movement, targeting, damage, or resource theft.

## ECS storage

Raid Defense composes typed component stores over `collection-kernels::SparseMap<T>` and uses `SparseSet` for entity liveness.

Current components include:

- `Transform` — fixed-point X/Z world position;
- `Health` — current and maximum hit points;
- `Attack` — tower/raider combat values instantiated from active rules;
- `Building` — authoritative grid occupancy and building kind;
- `Tower` — tower archetype and upgrade level;
- `ResourceStorage` — Town Hall, Storage House, Sawmill, Forest, and construction material storage;
- `ResourceProducer` — Sawmill forestry work profile retained as ECS/configuration data, not an autonomous production system;
- `Housing` — authoritative population capacity;
- `Person` — worker state, target, cargo, and cargo capacity;
- `Raider` — raid identity/spawn edge;
- `Movement` — deterministic cell-to-cell movement used by raiders and people;
- `Projectile` — target, damage payload, speed, and originating tower archetype.

`GameState` also owns deterministic per-worker forestry work ticks and raid-rally ticks. Both participate in checksum/replay identity because either can change future authoritative outcomes.

## Determinism and pathing

Authoritative positions are integer fixed-point values (`CELL_SCALE = 1000`). Grid routes use deterministic breadth-first search with fixed neighbor ordering. Worker selection/pathing uses stable ordering and entity identifiers when outcomes could otherwise depend on iteration order. Towers target by distance and entity id. Projectiles use deterministic integer movement.

Building commands validate completely before mutation. A new blocking building must keep every edge/active raider connected to the Town Hall, every Sawmill reachable by workers, and every currently moving worker able to finish their task.

Equal rules, seeds, and ordered commands replay to equal ECS state and equal checksums. Rally progress, worker forestry progress, and pending wave-spawn state are part of that deterministic identity, so resume cannot change when a worker finishes gathering or when the next raider appears. Different rules intentionally produce a different checksum identity even if a changed rule has not yet affected an entity.

## Current vertical slice

The current playable loop has:

- 21×21 authoritative grid and 3D presentation;
- central Town Hall plus distributed Storage Houses for spendable wood;
- renewable seeded Forests;
- people who gather Forest stock, carry it to Sawmills, then haul Sawmill stock into settlement storage;
- cursor/touch ghost placement with authoritative Rust validation;
- Arrow and Cannon towers with three upgrade levels, worker-built construction sites, and real projectile entities;
- raiders from four edges that enter waves over authoritative simulation time and steal settlement wood;
- deterministic daytime preparation, a worker rally window, then active night raids;
- completed-wave progression;
- Houses unlocked by completed-wave progression to expand population/logistics capacity;
- a centralized, validated rules profile controlling ordinary balance and progression values.

## Next mechanics

The strongest next mechanics remain vertical rather than content-heavy:

1. larger deterministic wave composition recipes and distinct raider archetypes;
2. richer worker allocation/priorities once there are multiple resource types;
3. worker vulnerability/evacuation only if it improves the economy-defense decision rather than adding busywork;
4. movement/status effects and projectile variants;
5. tower sale/rebuild and authoritative/advisory path-preview feedback;
6. spatial-query acceleration once measured entity counts justify it;
7. richer 3D assets, animation, cargo feedback, terrain, and impact effects without moving simulation authority out of Rust.

External rule-file loading or mod profiles can be added later if there is a concrete need. The current typed Rust profile already provides one authoritative configuration surface without introducing parsing/versioning complexity prematurely.
