# Raid Defense design

## Game direction

Raid Defense is a deterministic grid-based tower-defense/maul game inspired by Warcraft III custom maps. The gameplay plane is a 2D integer grid, while the presentation is fully 3D. Players shape enemy routes by placing defensive buildings between multiple edge spawns and a central town.

The key gameplay property is intentional path shaping: buildings block cells, raiders repath around them, and a placement is rejected if it would remove every valid route from any active spawn or currently moving raider to the town.

## Authority boundary

`raid-defense-core` is the sole authority for ECS state and game outcomes. It owns entity lifecycle, components, grid occupancy, pathfinding, wave spawning, movement, targeting, damage, health, rewards, command validation, replay, and checksums.

`raid-defense-wasm` is a versioned serialization adapter only. It maps JSON commands/events/snapshots without recomputing rules.

The browser owns input, camera, interpolation/presentation, and 3D rendering. It may request deterministic ticks, but it never decides where a raider can move, whether a tower may be placed, what a tower targets, or how much damage occurs.

## ECS storage

Raid Defense composes typed component stores over `collection-kernels::SparseMap<T>` and uses `SparseSet` for entity liveness. These are reusable low-level collection primitives from `rust-kernels`, not a cross-repository game framework.

Current components:

- `Transform`: fixed-point X/Z position on the gameplay plane;
- `Health`: current and maximum hit points;
- `Attack`: damage, range, and cooldown;
- `Building`: grid occupancy and building kind;
- `Raider`: raider identity/spawn edge;
- `Movement`: current/next cell, sub-cell progress, and speed.

Systems iterate only the components they need. This boundary is intended to scale to large waves and future projectile/effect populations without central entity structs accumulating unrelated fields.

## Determinism

Authoritative positions are integer fixed-point values (`CELL_SCALE = 1000`). Paths are found with deterministic breadth-first search and a fixed neighbor order. Target selection is ordered by distance and entity id. Equal seeds plus equal ordered commands therefore replay to equal component state and equal checksums.

Commands validate fully before mutation. In particular, tower placement verifies bounds, protected cells, occupancy, gold, and reachability from all four gates plus every active raider before spending resources or spawning an entity.

## Current vertical slice

The first maul slice uses a 17×13 grid with a protected 3×3 town and four edge gates. A wave currently contains one raider from each edge. Guard towers block cells, attack nearby raiders, and award gold on kills. Raiders move toward the town and damage it when they arrive.

The models are deliberately primitive geometry. The rendering contract is already 3D, so later asset work can replace meshes without changing the authoritative simulation representation.

## Next systems

The next gameplay depth should stay vertical:

1. tower archetypes and upgrade paths;
2. projectile entities and impact systems for non-instant attacks;
3. larger timed waves, spawn schedules, and distinct raider archetypes;
4. movement traits such as ground/flying and slow/status components;
5. tower sale/rebuild flows and path-preview feedback;
6. spatial-query acceleration for targeting when entity counts justify it;
7. richer 3D assets, animation, effects, and terrain while preserving simulation authority in Rust.
