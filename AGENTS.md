# Repository agent guidance

`raid-defense` is being rebuilt from the ground up. New implementation work belongs under `v2/` until the replacement browser client is ready to take over the root surfaces.

## Authority

- `v2/crates/raid-defense-core` is authoritative for game rules, command validation, deterministic simulation, replay, and checksums.
- The existing root Rust crate and `visualization/` are migration inputs and compatibility surfaces, not architecture to copy into v2.
- Presentation code must not recompute authoritative game outcomes. It may derive view-only layout and rendering state.
- Adapters may translate data but must not own game rules.

## Determinism

- Every simulation outcome must be a pure consequence of initial seed, authoritative state, and ordered commands.
- Validate a command completely before mutating state. Rejected commands must leave state unchanged.
- Prefer integer/fixed representations for authoritative rules. Do not introduce wall-clock, platform, browser, or unordered-iteration dependence.
- Add replay or checksum coverage whenever a new system can affect authoritative state.

## Validation

- Preserve the hosted layer order: fast -> integration -> workflow -> e2e. Do not skip a failed required layer.
- Keep the cheap deterministic core checks ahead of browser or hosted acceptance.
- GitHub Pages is the primary browser acceptance surface once a slice is integrated.
- Do not treat missing, cancelled, or unavailable evidence as green.

## UI

- Do not create cards, panels, counters, or framed sections that only explain the page and provide no workflow value.
- Keep controls and feedback close to the world state they affect.
- The browser client should expose the Rust-owned game rather than duplicating it in TypeScript.
