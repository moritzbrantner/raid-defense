# Raid Defense

A deterministic Warcraft III-style maul/tower-defense experiment: build towers on a 2D gameplay grid while the battlefield is rendered as a 3D world.

## Current MVP

The playable slice is intentionally small but architectural rather than mocked:

- a 17×13 authoritative build grid;
- a protected 3×3 town at the center;
- one raider gate on each edge of the map;
- Arrow and Cannon tower archetypes placed directly on grid cells;
- three deterministic upgrade levels with archetype-specific costs and combat stats;
- deterministic shortest-path routing from every gate to the town;
- maul-style maze building: towers may redirect raiders but cannot seal all routes;
- fixed-point raider and projectile movement;
- ECS-owned transforms, health, attacks, buildings, towers, raiders, movement, and projectiles;
- deterministic tower targeting, projectile impacts, damage, kills, and gold rewards;
- a React Three Fiber client that renders the authoritative Rust/WASM snapshot in 3D, including in-flight projectiles;
- GitHub Pages deployment with browser acceptance against the real WASM game.

## Architecture

```text
rust-kernels / collection-kernels
        SparseSet + SparseMap<T>
                 |
                 v
crates/raid-defense-core
        authoritative ECS + game systems
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
- `Health` — damageable town, towers, raiders, and future units;
- `Attack` — damage, range, cooldown, and projectile launch speed;
- `Building` — town and tower occupancy on the gameplay grid;
- `Tower` — tower archetype and upgrade level;
- `Raider` — wave/spawn identity;
- `Movement` — deterministic cell-to-cell raider motion;
- `Projectile` — target, damage payload, speed, and originating tower archetype.

Future effects, units, status effects, movement traits, destructible objects, and additional tower/raider families should be added as components/systems rather than growing entity-specific inheritance trees.

## Validation

Hosted validation promotes changes in this order:

1. `fast` — Rust format, Clippy, unit and adapter tests.
2. `integration` — deterministic build/wave replay traces and transactional rejection checks.
3. `workflow` — frozen browser install, lint, release WASM generation, strict TypeScript, and Vite build.
4. `e2e` — Playwright against the real WASM simulation and 3D client.

GitHub Pages is the integrated playable surface after merge.