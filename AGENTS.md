# Repository agent guidance

`raid-defense` is a deterministic frontier-defense strategy game. The repository has one authoritative game implementation: the Rust workspace under `crates/`.

## Authority

- `crates/raid-defense-core` owns game rules, command validation, deterministic simulation, replay, and checksums.
- `crates/raid-defense-wasm` is a translation boundary only. It may serialize and map values, but it must not own game rules.
- `visualization/` owns browser input and presentation only. It must not recompute authoritative game outcomes.
- Presentation code may derive view-only layout and rendering state.

## Determinism

- Every simulation outcome must be a pure consequence of initial seed, authoritative state, and ordered commands.
- Validate a command completely before mutating state. Rejected commands must leave state unchanged.
- Prefer integer or fixed representations for authoritative rules. Do not introduce wall-clock, platform, browser, or unordered-iteration dependence.
- Add replay or checksum coverage whenever a new system can affect authoritative state.

## Validation

- Preserve the hosted layer order: fast -> integration -> workflow -> e2e. Do not skip a failed required layer.
- Keep cheap deterministic core checks ahead of browser or hosted acceptance.
- GitHub Pages is the primary integrated browser acceptance surface after merge.
- Do not treat missing, cancelled, or unavailable evidence as green.

## UI

- Do not create cards, panels, counters, or framed sections that only explain the page and provide no workflow value.
- Keep controls and feedback close to the world state they affect.
- Keep browser state interaction-focused; the Rust core remains the game-state authority.
