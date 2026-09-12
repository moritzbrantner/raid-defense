# Raid Defense design

## Goal

Raid Defense is a deterministic strategy game about holding and extending a vulnerable frontier. The core loop is deliberately small: secure territory, build forts, recruit defenders, expand only from stable positions, and survive raids whose timing and strength are reproducible from the game seed and command history.

## Authority boundary

`raid-defense-core` is the sole authority for game state and outcomes. A command either validates completely and produces an event plus a new authoritative state, or it is rejected without changing state. This transactional boundary is required for replay, testing, browser parity, and future persistence/networking.

`raid-defense-wasm` translates JSON commands and snapshots. It exposes stable contract/error/event codes and the core checksum, but it contains no alternate rule implementation.

The browser is a client of that contract. Selection, layout, focus, animation, and rendering are browser concerns; treasury changes, claims, recruitment capacity, raids, damage, control, and other game outcomes are not.

## Deterministic state

Authoritative state currently contains the seed, day, treasury, influence, capital health, province state, active raid, and deterministic event nonce. Checksums cover all authoritative values. Equal seeds plus equal ordered command streams must replay to equal state and equal checksums.

Authoritative algorithms use integer state. Raid target/strength selection uses a deterministic integer mixer rather than wall-clock or browser randomness.

## Current game loop

A campaign starts at the capital. Forts increase defensive capacity and make adjacent expansion possible. Garrisons are bounded by local fort capacity. Claims require a connected controlled fort with stable control. Time advances resources and control, and every third day can schedule a seeded raid with explicit arrival time. Raid resolution is owned entirely by the core.

## Evidence

Changes promote through four hosted layers:

- fast: format, Clippy, core and adapter tests;
- integration: an opening-defense replay trace and transactional rejection checks;
- workflow: Rust-to-WASM-to-strict-TypeScript browser build;
- e2e: Playwright commands against the real WASM game.

A failed or missing layer is not treated as success.

## Next systems

The next depth should come from deterministic logistics rather than UI breadth: supply reach, reinforcement travel, fort capacity and infrastructure, frontier exposure, and distinct raider objectives. Governance systems such as loyalty, prosperity, taxation, and political pressure should follow once the defense/logistics loop is strong enough for those consequences to matter.
