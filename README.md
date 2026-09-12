# Raid Defense

A deterministic frontier-defense strategy game with Rust-authoritative rules, an ECS-backed runtime, a thin WASM contract boundary, and a browser client that presents rather than reimplements outcomes.

## Current playable slice

The rebuilt game currently includes:

- seeded province state across a frontier;
- fort construction and garrison recruitment;
- connected frontier claims;
- deterministic raid scheduling and resolution;
- replay from an ordered command stream;
- an authoritative state checksum;
- fail-closed command validation with no mutation on rejection;
- ECS-backed province entities using the shared `ecs-lab` sparse-set runtime;
- browser controls for selecting provinces, fortifying, recruiting, claiming territory, advancing time, and responding to raid feedback.

## Architecture

```text
crates/raid-defense-core   deterministic domain rules and outcomes
          |
          v
crates/raid-defense-ecs    runtime composition: ecs-lab entities/layout + domain rules
          |
          v
crates/raid-defense-wasm   versioned serialization/browser boundary only
          |
          v
visualization/             input and presentation; deployed with GitHub Pages
```

`raid-defense-ecs` pins the tested `ecs-lab` sparse-set/workload crates. The shared ECS currently owns province entity identity and deterministic layout/topology data; Raid Defense-specific fort, control, economy, and raid semantics remain in `raid-defense-core` instead of being forced into generic position/velocity components.

The browser does not recompute simulation outcomes that belong to Rust. The existing Pages client is the MVP surface and runs the real WASM runtime.

## Validation

Hosted validation promotes changes in this order:

1. `fast` — Rust formatting, Clippy with warnings denied, core/ECS runtime/adapter tests.
2. `integration` — deterministic opening-defense command trace.
3. `workflow` — frozen Bun install, browser lint, release WASM generation, strict TypeScript, and Vite build.
4. `e2e` — Chromium/Playwright acceptance against the real WASM game.

GitHub Pages is the integrated browser surface after merge.

## Local commands

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets

cd visualization
bun install --frozen-lockfile
bun run build
bun run e2e
```
