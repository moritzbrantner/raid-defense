# Raid Defense

A deterministic Warcraft III-style maul/tower-defense experiment: build on a 2D gameplay grid while the battlefield is rendered as a 3D world.

## Current MVP

The playable slice is intentionally small but architectural rather than mocked:

- a 17×13 authoritative build grid;
- a protected Town Hall at the center;
- one raider gate on each edge of the map;
- Arrow and Cannon towers with three deterministic upgrade levels;
- maul-style route shaping without allowing complete path sealing;
- Sawmills that create wood into bounded local buffers;
- **real people/carrier ECS entities** that travel from the Town Hall to Sawmills, pick up wood, and carry it back before it becomes spendable;
- population capacity and progression-gated Houses;
- separate tracking of started waves and completed waves so progression cannot unlock early;
- deterministic daytime preparation followed by automatic night raids;
- fixed-point movement for raiders and workers plus deterministic projectile movement;
- raiders that steal stored Town Hall wood before damaging the Town Hall;
- a React Three Fiber client that renders buildings, people, cargo, raiders, and projectiles from the authoritative Rust/WASM snapshot;
- GitHub Pages deployment with browser acceptance against the real WASM game.

## Architecture

```text
rust-kernels / collection-kernels
        SparseSet + SparseMap<T>
                 |
                 v
crates/raid-defense-core
        rules.rs          typed rule schema + validation
        default_rules.rs  standard balance profile
        lib.rs            authoritative ECS + systems
                 |
                 v
crates/raid-defense-wasm
        versioned JSON translation only
                 |
                 v
visualization/
        3D rendering + player input only
```

`raid-defense-core` is the sole game authority. The browser never recomputes production, carrier assignments, cargo transfers, housing unlocks, day/night transitions, path legality, combat, or theft.

### Rules are explicit simulation input

Ordinary gameplay tuning is centralized in `STANDARD_RULES` rather than scattered through systems. The typed `GameRules` profile covers economy, population/logistics, buildings, towers, raids, progression, and day/night cadence.

Every `GameState` owns one immutable rules profile. Systems consume that profile directly, and the active profile is fingerprinted into deterministic checksums. This means costs, health, production rates, carrier capacity, House unlock timing, tower stats, raid scaling, or day length can be adjusted without rewriting the corresponding ECS systems.

Structural simulation contracts remain separate: grid dimensions, fixed-point representation, deterministic path-neighbor order, and stable tie-breaking are engine invariants rather than ordinary balance knobs.

`GameState::new(seed)` creates the standard game. Validated custom rule profiles are available for tests and future game modes. External JSON/TOML rule loading is intentionally not part of the core yet; the typed profile gives one authoritative configuration surface without adding format/versioning complexity before it is needed.

## ECS direction

The current component boundary includes:

- `Transform` — authoritative fixed-point world position;
- `Health` — Town Hall, buildings, people, and raiders;
- `Attack` — instantiated tower/raider combat values;
- `Building` — grid occupancy and building kind;
- `Tower` — tower archetype and level;
- `ResourceStorage` — spendable Town Hall wood and local Sawmill buffers;
- `ResourceProducer` — timed Sawmill output;
- `Housing` — population capacity;
- `Person` — carrier task, target, cargo, and capacity;
- `Raider` — raid/spawn identity;
- `Movement` — deterministic cell-to-cell unit motion;
- `Projectile` — authoritative in-flight attacks.

Future resources, worker roles, effects, units, status effects, movement traits, destructible objects, and additional tower/raider families should be added as components/systems rather than growing entity-specific inheritance trees. Tunable behavior should be added through the rules schema rather than new system-local constants.

## Validation

Hosted validation promotes changes in this order:

1. `fast` — Rust format, Clippy, unit and adapter tests.
2. `integration` — deterministic economy/defense replay traces and transactional rejection checks.
3. `workflow` — frozen browser install, lint, release WASM generation, strict TypeScript, and Vite build.
4. `e2e` — Playwright against the real WASM simulation and 3D client.

GitHub Pages is the integrated playable surface after merge.
