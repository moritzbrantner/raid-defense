# Raid Defense design

## Game direction

Raid Defense is a deterministic grid-based tower-defense/maul game inspired by Warcraft III custom maps. The gameplay plane is a 2D integer grid, while the presentation is fully 3D. Players shape enemy routes by placing defensive buildings between multiple edge spawns and a central town.

The key gameplay property is intentional path shaping: buildings block cells, raiders repath around them, and a placement is rejected if it would remove every valid route from any active spawn or currently moving raider to the town.

## Authority boundary

`raid-defense-core` is the sole authority for ECS state and game outcomes. It owns entity lifecycle, components, grid occupancy, pathfinding, wave spawning, movement, targeting, projectile lifecycle, upgrades, damage, health, rewards, command validation, replay, and checksums.

`raid-defense-wasm` is a versioned serialization adapter only. It maps JSON commands/events/snapshots without recomputing rules.

The browser owns input, camera, interpolation/presentation, and 3D rendering. It may request deterministic ticks, but it never decides where a raider or projectile can move, whether a tower may be placed or upgraded, what a tower targets, or how much damage occurs.

## ECS storage

Raid Defense composes typed component stores over `collection-kernels::SparseMap<T>` and uses `SparseSet` for entity liveness. These are reusable low-level collection primitives from `rust-kernels`, not a cross-repository game framework.

Current components:

- `Transform`: fixed-point X/Z position on the gameplay plane;
- `Health`: current and maximum hit points;
- `Attack`: damage, range, cooldown, and projectile launch speed;
- `Building`: grid occupancy and building kind;
- `Tower`: tower archetype and upgrade level;
- `Raider`: raider identity/spawn edge;
- `Movement`: current/next cell, sub-cell progress, and speed;
- `Projectile`: target entity, damage payload, speed, and visual/combat archetype.

Systems iterate only the components they need. This boundary is intended to scale to large waves and future effect/unit populations without central entity structs accumulating unrelated fields.

## Determinism

Authoritative positions are integer fixed-point values (`CELL_SCALE = 1000`). Paths are found with deterministic breadth-first search and a fixed neighbor order. Target selection is ordered by distance and entity id. Projectiles pursue targets with deterministic integer movement and integer square-root distance normalization. Equal seeds plus equal ordered commands therefore replay to equal component state and equal checksums.

Commands validate fully before mutation. Tower placement verifies bounds, protected cells, occupancy, gold, and reachability from all four gates plus every active raider before spending resources or spawning an entity. Tower upgrades validate the selected tower, level cap, and price before changing stats or gold.

Entity lifecycle is closed over dependent state: when a raider leaves the world, projectiles targeting that entity are removed at the same authoritative boundary rather than being left as orphaned entities.

## Current vertical slice

The current maul slice uses a 17×13 grid with a protected 3×3 town and four edge gates. A wave contains one raider from each edge. Players can build:

- Arrow towers: cheaper, faster-firing, shorter-ranged projectiles;
- Cannon towers: more expensive, slower-firing, longer-ranged and harder-hitting projectiles.

Both archetypes have three upgrade levels. Towers block grid cells, acquire deterministic targets, create real projectile ECS entities, and only deal damage when those projectiles impact moving raiders. Kills award gold; raiders that reach the town damage it and are despawned together with any projectiles still targeting them.

The models are deliberately primitive geometry. The rendering contract is already 3D, so later asset work can replace meshes without changing the authoritative simulation representation.

## Next systems

The next gameplay depth should stay vertical:

1. larger timed waves, spawn schedules, and distinct raider archetypes;
2. movement traits such as ground/flying plus slow/status components;
3. projectile variants such as splash, piercing, or status payloads where the archetype needs them;
4. tower sale/rebuild flows and authoritative/advisory path-preview feedback;
5. spatial-query acceleration for targeting once measured entity counts justify it;
6. richer 3D assets, animation, effects, terrain, and impact feedback while preserving simulation authority in Rust.