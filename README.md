# Raid Defense

A deterministic frontier-defense strategy game with a Rust-authoritative simulation core, a thin WASM contract boundary, and a browser client that presents rather than reimplements the rules.

## Current playable slice

The rebuilt game currently includes:

- seeded province state across a frontier;
- fort construction and garrison recruitment;
- connected frontier claims;
- deterministic raid scheduling and resolution;
- replay from an ordered command stream;
- an authoritative state checksum;
- fail-closed command validation with no mutation on rejection;
- browser controls for selecting provinces, fortifying, recruiting, claiming territory, advancing time, and responding to raid feedback.

## Architecture

```text
crates/raid-defense-core   authoritative rules and deterministic state
          |
          v
crates/raid-defense-wasm   versioned serialization/browser boundary only
          |
          v
visualization/             input and presentation
```

The browser may derive visual layout, but it does not recompute simulation outcomes that belong to the core.

## Validation

Hosted validation promotes changes in this order:

1. `fast` — Rust formatting, Clippy with warnings denied, and unit/adapter tests.
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
