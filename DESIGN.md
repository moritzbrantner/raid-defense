# Raid Defense design

## Game direction

Raid Defense is a deterministic grid-based tower-defense/maul game inspired by Warcraft III custom maps. The gameplay plane is a 2D integer grid, while the presentation is fully 3D.

The central game tension is **economy versus defense**. The Town Hall stores the settlement's resources. Economic buildings increase future income, while defensive buildings protect that accumulated value from raiders. Spending too much on economy leaves the settlement exposed; spending everything on towers leaves no compounding production base.

Buildings also shape the battlefield. Towers and economic buildings occupy grid cells, raiders repath around them, and a placement is rejected if it would remove every valid route from any active spawn or currently moving raider to the Town Hall.

## Resource model

Wood is the first authoritative resource.

- The Town Hall owns a `ResourceStorage` component and begins with a bounded amount of stored wood.
- Sawmills own `ResourceProducer` components. Every production interval they deposit wood into the Town Hall, bounded by its storage capacity.
- Sawmills cost wood to construct, so investing in production has an immediate defensive opportunity cost.
- Arrow towers, Cannon towers, and tower upgrades consume wood from the Town Hall.
- Raider kills do not mint resources. Wood comes from the economy, not combat rewards.
- Raiders that reach the Town Hall steal stored wood. If there is no wood left to steal, they damage the Town Hall instead.

This makes stored wealth itself part of the risk surface: a stronger economy gives the player more options, but also creates more value for successful raiders to take.

## Authority boundary

`raid-defense-core` is the sole authority for ECS state and game outcomes. It owns entity lifecycle, resource storage/production, building costs, grid occupancy, pathfinding, wave spawning, movement, targeting, projectile lifecycle, upgrades, theft, damage, health, command validation, replay, and checksums.

`raid-defense-wasm` is a versioned serialization adapter only. It maps JSON commands/events/snapshots without recomputing rules.

The browser owns input, camera, interpolation/presentation, and 3D rendering. It may request deterministic ticks, but it never decides production, spending, theft, movement, placement legality, targeting, projectile impacts, or damage.

## ECS storage

Raid Defense composes typed component stores over `collection-kernels::SparseMap<T>` and uses `SparseSet` for entity liveness. These are reusable low-level collection primitives from `rust-kernels`, not a cross-repository game framework.

Current components:

- `Transform`: fixed-point X/Z position on the gameplay plane;
- `Health`: current and maximum hit points;
- `Attack`: damage, range, cooldown, and projectile launch speed;
- `Building`: Town Hall, tower, or Sawmill grid occupancy;
- `ResourceStorage`: stored wood and storage capacity, currently owned by the Town Hall;
- `ResourceProducer`: resource kind, production amount, interval, and progress, currently used by Sawmills;
- `Tower`: tower archetype and upgrade level;
- `Raider`: raider identity/spawn edge;
- `Movement`: current/next cell, sub-cell progress, and speed;
- `Projectile`: target entity, damage payload, speed, and visual/combat archetype.

Systems iterate only the components they need. New resources or economic buildings should extend these generic storage/production boundaries instead of adding unrelated global counters.

## Determinism

Authoritative positions are integer fixed-point values (`CELL_SCALE = 1000`). Paths are found with deterministic breadth-first search and a fixed neighbor order. Target selection is ordered by distance and entity id. Projectiles pursue targets with deterministic integer movement and integer square-root distance normalization. Resource producers are processed in entity-id order. Equal seeds plus equal ordered commands therefore replay to equal component state and equal checksums.

Commands validate fully before mutation. Building placement verifies bounds, protected cells, occupancy, available Town Hall wood, and reachability from all four gates plus every active raider before spending wood or spawning an entity. Tower upgrades validate the selected tower, level cap, and wood price before changing stats or storage.

Entity lifecycle is closed over dependent state: when a raider leaves the world, projectiles targeting that entity are removed at the same authoritative boundary rather than being left as orphaned entities.

## Current vertical slice

The current slice uses a 17×13 grid with a protected 3×3 Town Hall and four edge gates.

The economy currently consists of Sawmills that turn time into Town Hall wood. The player chooses whether to spend the starting wood on Sawmills, Arrow towers, Cannon towers, or tower upgrades. Sawmills and towers both occupy cells and participate in path shaping.

Raids still begin manually in this slice. A raid contains one raider from each edge. Raiders move toward the Town Hall, and successful arrivals steal stored wood before they can damage the Town Hall itself. Towers block grid cells, acquire deterministic targets, create real projectile ECS entities, and only deal damage when those projectiles impact moving raiders.

The models are deliberately primitive geometry. The rendering contract is already 3D, so later asset work can replace meshes without changing the authoritative simulation representation.

## Next systems

The next mechanics should deepen the economy/defense tension before broadening content:

1. automatic raid cadence / preparation timer so waiting for infinite Sawmill income is impossible;
2. returning raiders that visibly carry stolen resources toward an exit, with a rule for whether killing them recovers the loot;
3. larger timed waves, spawn schedules, and distinct raider archetypes;
4. additional resources and economic buildings only when wood/storage/production semantics are stable;
5. movement traits such as ground/flying plus slow/status components;
6. projectile variants such as splash, piercing, or status payloads where the archetype needs them;
7. tower sale/rebuild flows and authoritative/advisory path-preview feedback;
8. spatial-query acceleration for targeting once measured entity counts justify it;
9. richer 3D assets, animation, effects, terrain, and impact feedback while preserving simulation authority in Rust.
