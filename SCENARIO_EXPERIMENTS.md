# Scenario experiments

`raid-defense` has two complementary scenario surfaces:

- GitHub Pages is for interactively playing one scenario. The start-menu scenario settings are translated into validated `ScenarioOptions`, then into immutable authoritative `GameRules` in Rust.
- `scenario-experiment` is for deterministic comparison across scenarios, seeds, and automated player policies. It uses the same `ScenarioOptions -> GameRules` path as the game and does not reimplement simulation rules.

## Run one custom scenario

```bash
cargo run -p raid-defense-sim --bin scenario-experiment -- \
  --seed 40 \
  --runs 16 \
  --ticks 8000 \
  --policy balanced \
  --starting-supplies lean \
  --forest-density sparse \
  --forest-regrowth slow \
  --raid-size large \
  --raider-strength harsh
```

Every case prints the full scenario option tuple and the deterministic rules fingerprint before the ordinary per-seed simulator report. This makes copied experiment evidence self-describing.

## Sweep one dimension

```bash
cargo run -p raid-defense-sim --bin scenario-experiment -- \
  --seed 40 \
  --runs 32 \
  --ticks 10000 \
  --policy balanced \
  --sweep raid-size
```

This runs `small`, `standard`, and `large` raid sizes against the exact same seed range and all other standard scenario options.

## Cross multiple dimensions

Repeat `--sweep` to create a Cartesian product:

```bash
cargo run -p raid-defense-sim --bin scenario-experiment -- \
  --seed 40 \
  --runs 32 \
  --ticks 10000 \
  --policy balanced \
  --sweep raid-size \
  --sweep raider-strength
```

The example produces nine cases: three raid sizes × three raider strengths. Baseline overrides can be combined with sweeps, so an economy constraint can be held constant while combat dimensions vary.

Supported sweep dimensions are:

- `starting-supplies`
- `forest-density`
- `forest-regrowth`
- `sawmill-throughput`
- `raid-size`
- `raider-strength`
- `day-length`
- `raid-timing`
- `raid-economy`

The harness rejects matrices above 256 scenario cases so accidental combinatorial explosions fail closed.

## Automated player policies

The existing simulator policies remain the experiment driver:

- `passive`: never builds.
- `economy`: favors production and storage.
- `balanced`: grows economy and defense together.
- `defense`: prioritizes towers and upgrades.

For a `manual` raid-timing scenario, the experiment harness deliberately presses `StartWave` after the configured peaceful day length and again after each completed wave. This preserves deterministic day cadence while keeping the authoritative core's manual-raid rule intact. Future experiments can add different manual-wave driver strategies without changing `GameRules` or simulation systems.

## Interpretation

Useful comparisons include survival rate, completed waves, kills, resource production/delivery/theft, peak live raiders, settlement composition, final health, and checksums. Compare the same seed range and policy when evaluating a rule change; change one dimension at a time first, then use cross-sweeps to find interactions.

A scenario experiment is evidence, not balance authority. Ordinary balance values continue to live in `STANDARD_RULES`; scenario options transform that profile through validated Rust rules, and all game outcomes remain authoritative in `raid-defense-core`.
