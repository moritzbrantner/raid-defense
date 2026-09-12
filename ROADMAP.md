# Raid Defense rebuild roadmap

The rebuild is intentionally vertical: each milestone must leave behind a deterministic, testable slice rather than a broad scaffold with placeholder behavior.

## A. Foundation and authoritative core

- [x] Create an isolated v2 Rust workspace rather than extending the legacy farm-game-engine slice.
- [x] Establish seeded authoritative state, command validation, replay, and checksums.
- [x] Implement the first defense loop: forts, garrisons, frontier claims, raids, and resolution.
- [x] Add repository metadata, pinned Rust/Bun CI, Renovate, Pages, and layered hosted validation.
- [ ] Integrate the slice only after exact-head hosted evidence is green.

## B. Stable contract and WASM boundary

- Add a thin `raid-defense-wasm` adapter over `raid-defense-core`.
- Define versioned command, snapshot, event, and replay DTOs.
- Generate TypeScript declarations from the authoritative contract instead of maintaining parallel handwritten types.
- Prove native/WASM parity with the same command traces and checksums.
- Keep serialization failures fail-closed and separate from game-rule errors.

## C. Replacement browser client

- Build a small browser shell against the v2 contract rather than porting the current 89k-line application component.
- Make map selection, province inspection, fort construction, recruitment, advancing time, and raid feedback playable end to end.
- Keep game state in the Rust core; React owns interaction state and presentation only.
- Prefer world-space feedback over dashboard-style KPI cards.
- Add focused Playwright flows and promote the replacement through Pages.

## D. Logistics and defense depth

- Model supply reach, reinforcement travel, fort capacity, and frontier exposure as deterministic systems.
- Make province topology and infrastructure change where raids can emerge and how quickly they can be answered.
- Add distinct raider compositions and objectives without turning combat resolution into browser-owned logic.
- Preserve bounded, inspectable algorithms and deterministic replay.

## E. Governance after survival

- Reintroduce control, loyalty, prosperity, taxation, and political pressure only after the defense loop is strong.
- Make governance consequences feed back into manpower, supply, intelligence, and raid pressure.
- Reintroduce progression and objectives through v2-owned rules rather than copying legacy state shapes.

## F. Legacy removal

- Switch the browser and Pages surface to v2.
- Remove the old root game crate, generated legacy contracts, and `visualization/` only after equivalent required behavior has migrated.
- Remove compatibility validation at the same time; never keep stale duplicate authorities.
