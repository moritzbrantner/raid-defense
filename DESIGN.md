# Raid Defense design

## Game direction

Raid Defense is a deterministic grid-based tower-defense/maul game inspired by Warcraft III custom maps. The gameplay plane is a 2D integer grid, while the presentation is fully 3D.

The central tension is **economy versus defense**, but the economy is deliberately physical rather than just a set of counters. The Town Hall stores spendable resources. Sawmills create wood locally. People must travel to the Sawmills, pick that wood up, and carry it back to the Town Hall before the player can spend it. More economic buildings therefore create a logistics problem as well as more potential income.

Buildings also shape the battlefield. Towers, Sawmills, and Houses occupy grid cells, raiders repath around them, and placement is rejected if it would remove required raider or worker routes.

## Resource and logistics model

Wood is the first material resource.

- The Town Hall owns the authoritative spendable `ResourceStorage`.
- Sawmills own `ResourceProducer` components plus bounded local `ResourceStorage` buffers.
- Production fills the local Sawmill buffer; it never teleports directly to the Town Hall.
- People are ECS entities with `Person` cargo/task state, `Transform`, `Health`, and `Movement`.
- Idle people at the Town Hall are deterministically assigned to Sawmills with waiting wood.
- A person travels to an accessible Sawmill-adjacent cell, picks up at most their cargo capacity, then returns through the authoritative grid and deposits the load at the Town Hall.
- If Sawmills outproduce the available carriers, their local buffers fill and additional theoretical production is lost. Population is therefore real logistics throughput rather than a passive percentage bonus.
- Towers, upgrades, Sawmills, and Houses consume Town Hall wood.
- Raider kills do not mint resources.
- Raiders that reach the Town Hall steal stored wood; if none remains, they damage the Town Hall.

## Population and progression

The settlement begins with two people and population capacity for two, supplied by the Town Hall.

Houses are the first progression-gated economic building. They become available only after **10 waves have completed**. `completed_waves` is tracked separately from the current/last-started `wave`, so starting wave 10 cannot unlock the building early.

A House:

- costs wood;
- occupies and path-shapes one grid cell;
- adds two authoritative population capacity through a `Housing` component;
- introduces two additional people in the same successful command transaction.

This keeps population expansion tied to both progression and an immediate economic/defensive opportunity cost.

## Authority boundary

`raid-defense-core` is the sole authority for ECS state and game outcomes. It owns entity lifecycle, population, housing, local and Town Hall storage, production, carrier assignment/pathing/cargo transfer, grid occupancy, wave completion, building unlocks, raider pathfinding, targeting, projectiles, upgrades, theft, damage, health, command validation, replay, and checksums.

`raid-defense-wasm` is a versioned serialization adapter only. It maps commands/events/snapshots without recomputing rules.

The browser owns input, camera, and 3D presentation. It requests deterministic ticks but does not decide production, carrier assignments, cargo transfers, unlocks, movement, targeting, damage, or resource theft.

## ECS storage

Raid Defense composes typed component stores over `collection-kernels::SparseMap<T>` and uses `SparseSet` for entity liveness.

Current components include:

- `Transform` — fixed-point X/Z world position;
- `Health` — current and maximum hit points;
- `Attack` — tower/raider combat values;
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

Equal seeds plus equal ordered commands therefore replay to equal ECS state and equal checksums.

## Current vertical slice

The current playable loop has:

- 17×13 authoritative grid and 3D presentation;
- central Town Hall with stored wood;
- Arrow and Cannon towers with three upgrade levels and real projectile entities;
- Sawmills that create buffered wood;
- two starting carrier people who shuttle between Town Hall and Sawmills;
- raiders from four edges that steal Town Hall wood;
- completed-wave progression;
- Houses unlocked after 10 completed waves to expand population/logistics capacity.

## Next mechanics

The strongest next mechanics remain vertical rather than content-heavy:

1. automatic preparation/raid cadence so economic growth always consumes scarce time;
2. richer worker allocation/priorities once there are multiple resource types;
3. distinct raider archetypes and larger scheduled waves;
4. worker vulnerability/evacuation only if it improves the economy-defense decision rather than adding busywork;
5. movement/status effects and projectile variants;
6. tower sale/rebuild and authoritative/advisory path-preview feedback;
7. spatial-query acceleration once measured entity counts justify it;
8. richer 3D assets, animation, cargo feedback, terrain, and impact effects without moving simulation authority out of Rust.
