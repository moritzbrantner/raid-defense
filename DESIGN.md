# Raid Defense design

## Game direction

Raid Defense is a deterministic grid-based tower-defense/maul game inspired by Warcraft III custom maps. The gameplay plane is a 2D integer grid, while the presentation is fully 3D.

The central tension is **economy versus defense**, but the economy is deliberately physical rather than just a set of counters. The Town Hall stores spendable resources. Sawmills create wood locally. People must travel to the Sawmills, pick that wood up, and carry it back to the Town Hall before the player can spend it. More economic buildings therefore create a logistics problem as well as more potential income.

Buildings also shape the battlefield. Towers, Sawmills, and Houses occupy grid cells, raiders repath around them, and placement is rejected if it would remove required raider or worker routes.

## Rules and configuration architecture

Gameplay tuning is an explicit input to the simulation rather than a collection of hidden globals.

`rules.rs` defines a typed `GameRules` schema. It groups rules by domain:

- economy — starting storage, production cadence, and local buffers;
- population — starting people, capacity, health, movement, and carrying capacity;
- buildings — costs, health, House unlock progression, and population effects;
- towers — build costs, level stats, upgrade costs, health, and level limit;
- raids — wave size, spawn cadence, health/damage scaling, movement, and resource theft;
- cycle — daytime length, automatic raid cadence, and whether raids pause the economy.

`default_rules.rs` contains `STANDARD_RULES`, the single ordinary balance table for the shipped game. Systems do not own copies of these values: each `GameState` stores one immutable `GameRules` value and reads from it whenever a rule affects simulation behavior.

This distinction is deliberate. A designer should be able to change a Sawmill cost, House unlock wave, carrier capacity, tower damage, raid scaling, spawn cadence, or day duration by changing a rules profile rather than rewriting a system. By contrast, grid dimensions, fixed-point representation, deterministic path-neighbor order, and stable tie-breaking remain engine invariants because changing them alters simulation representation/topology rather than ordinary balance.

Rules validate before a configurable game starts. Invalid profiles fail closed instead of being silently normalized. Every complete rule set also has a deterministic fingerprint, which is included in game checksums so two states produced under different rules cannot accidentally claim the same authoritative identity.

`GameState::new(seed)` remains the standard-game convenience constructor. Custom profiles use the validated rules constructor. This gives tests and future game modes a supported extension point without making JSON/TOML parsing or mod loading part of the core simulation prematurely.

## Resource and logistics model

Wood is the first material resource.

- The Town Hall owns the authoritative spendable `ResourceStorage`.
- Sawmills own `ResourceProducer` components plus bounded local `ResourceStorage` buffers.
- Production fills the local Sawmill buffer; it never teleports directly to the Town Hall.
- People are ECS entities with `Person` cargo/task state, `Transform`, `Health`, and `Movement`.
- Idle people at the Town Hall are deterministically assigned to Sawmills with waiting wood.
- A person travels to an accessible Sawmill-adjacent cell, picks up at most their configured cargo capacity, then returns through the authoritative grid and deposits the load at the Town Hall.
- If Sawmills outproduce the available carriers, their local buffers fill and additional theoretical production is lost. Population is therefore real logistics throughput rather than a passive percentage bonus.
- Towers, upgrades, Sawmills, and Houses consume Town Hall wood.
- Raider kills do not mint resources.
- Raiders that reach the Town Hall steal stored wood; if none remains, they damage the Town Hall.

## Population and progression

Under the standard profile, the settlement begins with two people and population capacity for two, supplied by the Town Hall.

Houses are the first progression-gated economic building. The standard profile makes them available only after **10 waves have completed**. `completed_waves` is tracked separately from the current/last-started `wave`, so starting wave 10 cannot unlock the building early.

A standard House:

- costs wood;
- occupies and path-shapes one grid cell;
- adds authoritative population capacity through a `Housing` component;
- introduces additional people in the same successful command transaction.

The exact cost, unlock threshold, capacity, and number of people are rules-profile data, not House-system constants.

## Day/night cycle

Time pressure is authoritative simulation state. The standard profile has a 600-tick peaceful day (60 seconds at the current 100 ms browser tick cadence). When that countdown expires, Rust starts the next raid automatically. The first raider enters immediately and the remaining raiders enter at the configured authoritative spawn interval in a deterministic seed-and-wave-derived edge order.

The raid phase lasts for the entire spawn schedule plus the lifetime of all spawned raiders. A temporary gap with no live raider does not end the night, restart the day, allow another raid, or resume economic work. Under the standard profile, Sawmill production and carrier logistics remain paused until there are neither scheduled nor live raiders. Only then is the wave completed and a fresh daytime period started.

Those policy values live in `RaidRules` and `CycleRules`, while the pending schedule itself is checksum-bound `GameState`. The state machine remains deterministic if a future profile changes wave size, spawn cadence, day length, manual-only raids, or whether economic work continues during combat.

## Authority boundary

`raid-defense-core` is the sole authority for ECS state and game outcomes. It owns immutable game rules, entity lifecycle, population, housing, local and Town Hall storage, production, carrier assignment/pathing/cargo transfer, grid occupancy, day/night phase progression, timed raid scheduling, wave completion, building unlocks, raider pathfinding, targeting, projectiles, upgrades, theft, damage, health, command validation, replay, and checksums.

`raid-defense-wasm` is a versioned serialization adapter only. It maps commands/events/snapshots without recomputing rules. Values such as costs and unlock thresholds come from the active `GameState` rules, not duplicate adapter constants.

The browser owns input, camera, and 3D presentation. It requests deterministic ticks but does not decide production, carrier assignments, cargo transfers, phase transitions, spawn timing, unlocks, movement, targeting, damage, or resource theft.

## ECS storage

Raid Defense composes typed component stores over `collection-kernels::SparseMap<T>` and uses `SparseSet` for entity liveness.

Current components include:

- `Transform` — fixed-point X/Z world position;
- `Health` — current and maximum hit points;
- `Attack` — tower/raider combat values instantiated from active rules;
- `Building` — authoritative grid occupancy and building kind;
- `Tower` — tower archetype and upgrade level;
- `ResourceStorage` — Town Hall and Sawmill wood storage;
- `ResourceProducer` — timed Sawmill production;
- `Housing` — authoritative population capacity;
- `Person` — carrier state, target, cargo, and cargo capacity;
- `Raider` — raid identity/spawn edge;
- `Movement` — deterministic cell-to-cell movement used by raiders and people;
- `Projectile` — target, damage payload, speed, and originating tower archetype.

## Determinism and pathing

Authoritative positions are integer fixed-point values (`CELL_SCALE = 1000`). Grid routes use deterministic breadth-first search with fixed neighbor ordering. Carrier assignment is ordered by available Sawmill stock and stable entity id. Towers target by distance and entity id. Projectiles use deterministic integer movement.

Building commands validate completely before mutation. A new blocking building must keep every edge/active raider connected to the Town Hall, every Sawmill reachable by workers, and every currently moving worker able to finish their task.

Equal rules, seeds, and ordered commands replay to equal ECS state and equal checksums. Pending wave-spawn state is part of that deterministic identity, so resuming in a gap between raiders cannot change when or where the next raider appears. Different rules intentionally produce a different checksum identity even if a changed rule has not yet affected an entity.

## Current vertical slice

The current playable loop has:

- 21×21 authoritative grid and 3D presentation;
- central Town Hall with stored wood;
- Arrow and Cannon towers with three upgrade levels and real projectile entities;
- Sawmills that create buffered wood;
- starting carrier people who shuttle between Town Hall and Sawmills;
- raiders from four edges that enter waves over authoritative simulation time and steal settlement wood;
- deterministic daytime preparation followed by automatic night raids;
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
