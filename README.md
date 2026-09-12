# Raid Defense

A deterministic Warcraft III-style maul/tower-defense experiment: build an economy and defenses on a 2D gameplay grid while the battlefield is rendered as a 3D world.

## Current MVP

The current playable loop is about balancing economic growth against the value that raiders can steal:

- a 17×13 authoritative build grid;
- a protected 3×3 Town Hall at the center;
- the Town Hall owns bounded wood storage and starts with the settlement's initial wood;
- Sawmills cost wood and periodically deposit new wood into the Town Hall;
- Arrow and Cannon towers, plus tower upgrades, consume Town Hall wood;
- one raider gate on each edge of the map;
- successful raiders steal stored wood when they reach the Town Hall and only damage it when no wood remains;
- deterministic shortest-path routing from every gate to the Town Hall;
- maul-style maze building: Sawmills and towers may redirect raiders but cannot seal all routes;
- three deterministic tower upgrade levels with archetype-specific combat stats;
- fixed-point raider and projectile movement;
- ECS-owned transforms, health, attacks, buildings, resource storage, resource production, towers, raiders, movement, and projectiles;
- deterministic production, spending, targeting, projectile impacts, theft, and damage;
- a React Three Fiber client that renders the authoritative Rust/WASM snapshot in 3D, including Sawmills and in-flight projectiles;
- GitHub Pages deployment with browser acceptance against the real WASM game.

## Architecture

```text
rust-kernels / collection-kernels
        SparseSet + SparseMap<T>
                 |
                 v
crates/raid-defense-core
        authoritative ECS + economy/combat systems
                 |
                 v
crates/raid-defense-wasm
        versioned JSON translation only
                 |
                 v
visualization/
        3D rendering + player input only
```

`raid-defense-core` composes typed sparse component stores over the low-level reusable collection kernels. There is deliberately no second game implementation in JavaScript and no dependency on `ecs-lab` as an application library; `ecs-lab` remains the experiment/reference harness for ECS techniques.

## ECS direction

The current component boundary is designed for the scale of the intended game:

- `Transform` — authoritative 2D world position in fixed-point units, rendered in 3D;
- `Health` — damageable Town Hall, towers, Sawmills, raiders, and future units;
- `Attack` — damage, range, cooldown, and projectile launch speed;
- `Building` — Town Hall, tower, and Sawmill occupancy on the gameplay grid;
- `ResourceStorage` — stored wood and capacity, currently attached to the Town Hall;
- `ResourceProducer` — resource kind, amount, interval, and progress, currently attached to Sawmills;
- `Tower` — tower archetype and upgrade level;
- `Raider` — wave/spawn identity;
- `Movement` — deterministic cell-to-cell raider motion;
- `Projectile` — target, damage payload, speed, and originating tower archetype.

Future resources, producers, effects, units, movement traits, destructible objects, and additional tower/raider families should be added as components/systems rather than growing entity-specific inheritance trees.

## Validation

Hosted validation promotes changes in this order:

1. `fast` — Rust format, Clippy, unit and adapter tests.
2. `integration` — deterministic economy/build/raid replay traces and transactional rejection checks.
3. `workflow` — frozen browser install, lint, release WASM generation, strict TypeScript, and Vite build.
4. `e2e` — Playwright against the real WASM simulation and 3D client.

GitHub Pages is the integrated playable surface after merge.
