# Raid Defense design

## Goal

Raid Defense is a deterministic strategy game about holding and extending a vulnerable frontier. The core loop is deliberately small: secure territory, build forts, recruit defenders, expand only from stable positions, and survive raids whose timing and strength are reproducible from the game seed and command history.

## Authority boundary

`raid-defense-core` is the sole authority for Raid Defense-specific rules and outcomes. A command either validates completely and produces an event plus a new domain state, or it is rejected without changing that state. This transactional boundary is required for replay, testing, browser parity, and future persistence/networking.

`raid-defense-ecs` is the runtime composition layer. It pins the shared `ecs-lab` sparse-set/workload implementation, creates one ECS entity per province, owns deterministic layout/topology components, validates command addressing against the ECS entity set, and incorporates ECS state into the runtime checksum. It delegates fort, control, economy, claim, raid, and combat semantics to `raid-defense-core` rather than duplicating them.

`raid-defense-wasm` translates JSON commands and snapshots over that ECS runtime. It exposes stable contract/error/event codes and the runtime checksum, but it contains no alternate rule implementation.

The browser is a client of that contract. Selection, focus, animation, and rendering are browser concerns; treasury changes, claims, recruitment capacity, raids, damage, control, and other game outcomes are not.

## ECS boundary

`ecs-lab` is an experiment harness rather than a general-purpose game ECS framework. Raid Defense therefore reuses the parts that are already authoritative and reusable: entity lifecycle, deterministic sparse-set storage, and position/layout components. Game-specific components remain in the domain core until a shared typed-component contract exists that can represent them without abusing generic `Position` or `Velocity` data.

This keeps the migration real but narrow: the playable runtime now goes through the shared ECS technology without moving game rules into the browser or inventing a second ECS implementation locally.

## Deterministic state

Authoritative runtime state contains the seeded domain state plus the ECS province world. Checksums cover the core checksum and canonical ECS entity/component snapshot. Equal seeds plus equal ordered command streams must replay to equal state and equal checksums.

Authoritative algorithms use integer state. Raid target/strength selection uses a deterministic integer mixer rather than wall-clock or browser randomness.

## Current game loop

A campaign starts at the capital. Forts increase defensive capacity and make adjacent expansion possible. Garrisons are bounded by local fort capacity. Claims require a connected controlled fort with stable control. Time advances resources and control, and every third day can schedule a seeded raid with explicit arrival time. Raid resolution is owned entirely by the core.

## Evidence

Changes promote through four hosted layers:

- fast: format, Clippy, core/ECS runtime/adapter tests;
- integration: an opening-defense replay trace and transactional rejection checks;
- workflow: Rust-to-WASM-to-strict-TypeScript browser build;
- e2e: Playwright commands against the real WASM game.

A failed or missing layer is not treated as success.

## Next systems

The next depth should come from deterministic logistics rather than UI breadth: supply reach, reinforcement travel, fort capacity and infrastructure, frontier exposure, and distinct raider objectives. Those systems should become ECS components only when the shared storage contract can model them directly and clearly. Governance systems such as loyalty, prosperity, taxation, and political pressure should follow once the defense/logistics loop is strong enough for those consequences to matter.
