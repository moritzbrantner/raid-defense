# Raid Defense

A deterministic frontier-defense strategy game. The project is being rebuilt around a Rust-authoritative simulation core with thin WASM/browser adapters and hosted evidence for every integration layer.

## Rebuild status

The existing root crate and `visualization/` remain available while the replacement is built. New game logic starts in `v2/` and must not depend on the legacy farm-game-engine architecture.

The first v2 slice already owns:

- seeded province state;
- fort construction and garrison recruitment;
- frontier province claims;
- deterministic raid scheduling and resolution;
- replay from an ordered command stream;
- an authoritative state checksum;
- fail-closed command validation with no mutation on rejection.

## Architecture

```text
v2/crates/raid-defense-core   authoritative rules and deterministic state
            |
            v
future WASM/contract adapter  serialization and browser boundary only
            |
            v
future browser client         input, presentation, audio, rendering
```

The browser may derive visual layout, but it must not recompute simulation outcomes that belong to the core.

## Validation

Hosted validation promotes changes in this order:

1. `fast` — format, Clippy, and unit tests for the new core.
2. `integration` — compatibility tests for the existing simulation while migration is active.
3. `workflow` — build the current browser/WASM workflow.
4. `e2e` — Playwright browser acceptance.

GitHub Pages remains the integrated browser surface during the migration.

## Local commands

```bash
cargo fmt --manifest-path v2/Cargo.toml --all -- --check
cargo clippy --manifest-path v2/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path v2/Cargo.toml --all-targets
```

The legacy browser can still be built from `visualization/` until its v2 replacement lands.
