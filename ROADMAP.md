# Raid Defense roadmap

The rebuild is vertical: each milestone leaves behind a deterministic playable slice with hosted evidence rather than a broad scaffold with placeholder behavior.

## A. Foundation and authoritative core

- [x] Replace the farm-game-engine slice with an independent Rust workspace.
- [x] Establish seeded authoritative state, command validation, replay, and checksums.
- [x] Implement the first defense loop: forts, garrisons, frontier claims, raids, and resolution.
- [x] Add repository metadata, pinned Rust/Bun CI, Renovate, Pages, and layered hosted validation.
- [x] Prove the replacement through exact-head fast -> integration -> workflow -> e2e evidence.
- [x] Route the playable runtime through the pinned `ecs-lab` sparse-set entity world without moving Raid Defense-specific rules out of the domain core.

## B. Stable contract and WASM boundary

- [x] Add a thin `raid-defense-wasm` adapter over the authoritative runtime.
- [x] Version command, snapshot, event, and error envelopes.
- [x] Prove a native/WASM checksum parity path and fail-closed malformed-command handling.
- [ ] Generate browser DTO types from the authoritative contract instead of maintaining parallel handwritten view types.
- [ ] Expand parity coverage to representative multi-command traces.

## C. Replacement browser client

- [x] Replace the monolithic legacy application with a small browser shell against the new contract.
- [x] Make map selection, province inspection, fort construction, recruitment, advancing time, claims, and raid feedback playable end to end.
- [x] Keep game state in Rust; React owns interaction state and presentation only.
- [x] Add focused Playwright flows for successful commands, raid appearance, and rejected-command immutability.
- [ ] Promote the ECS-backed MVP through GitHub Pages after merge and verify the hosted WASM flow.

## D. Logistics and defense depth

- [ ] Model supply reach, reinforcement travel, fort capacity, and frontier exposure as deterministic systems.
- [ ] Promote reusable logistics/topology data into shared ECS components where the cross-repository contract is genuinely reusable.
- [ ] Make province topology and infrastructure change where raids can emerge and how quickly they can be answered.
- [ ] Add distinct raider compositions and objectives without moving combat resolution into the browser.
- [ ] Preserve bounded, inspectable algorithms and deterministic replay.

## E. Governance after survival

- [ ] Add control, loyalty, prosperity, taxation, and political pressure only after the defense loop is strong.
- [ ] Feed governance consequences back into manpower, supply, intelligence, and raid pressure.
- [ ] Add progression and objectives through core-owned rules rather than browser state.

## F. Repository hardening

- [ ] Generate and commit the canonical Rust lockfile for fully locked hosted Cargo commands.
- [ ] Prune browser dependencies no longer needed by the replacement client and refresh the Bun lock deterministically.
- [ ] Add repository-foundation/convergence checks from coding-tooling as the shared contract stabilizes.
- [ ] Reassess wasm-pack pinning and installation strategy without weakening reproducibility.
