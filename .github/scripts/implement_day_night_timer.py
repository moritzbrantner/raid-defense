from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


# Expose authoritative cycle state through the WASM contract.
path = Path("crates/raid-defense-wasm/src/lib.rs")
text = path.read_text()
text = replace_once(text, "const CONTRACT_VERSION: u8 = 5;", "const CONTRACT_VERSION: u8 = 6;", "contract version")
text = replace_once(
    text,
    "    wave: u32,\n    completed_waves: u32,\n    people: u16,\n",
    "    wave: u32,\n    completed_waves: u32,\n    day_ticks_remaining: u16,\n    is_night: bool,\n    people: u16,\n",
    "snapshot cycle fields",
)
text = replace_once(
    text,
    "            wave: snapshot.wave,\n            completed_waves: snapshot.completed_waves,\n            people: snapshot.people,\n",
    "            wave: snapshot.wave,\n            completed_waves: snapshot.completed_waves,\n            day_ticks_remaining: state.day_ticks_remaining(),\n            is_night: state.is_night(),\n            people: snapshot.people,\n",
    "snapshot cycle mapping",
)
text = replace_once(
    text,
    "    #[test]\n    fn sawmill_contract_matches_native_core() {\n",
    "    #[test]\n    fn snapshot_exposes_authoritative_day_night_state() {\n        let mut state = GameState::new(7);\n        let day = SnapshotDto::from(&state);\n        assert_eq!(day.day_ticks_remaining, raid_defense_core::DAY_LENGTH_TICKS);\n        assert!(!day.is_night);\n\n        state.apply(Command::StartWave).expect(\"wave should start\");\n        let night = SnapshotDto::from(&state);\n        assert_eq!(night.day_ticks_remaining, 0);\n        assert!(night.is_night);\n    }\n\n    #[test]\n    fn sawmill_contract_matches_native_core() {\n",
    "snapshot cycle test",
)
path.write_text(text)


# Update the browser contract types.
path = Path("visualization/src/simulationTypes.ts")
text = path.read_text()
if text.count("contract_version: 5;") != 2:
    raise SystemExit("simulation types: expected two v5 contract literals")
text = text.replace("contract_version: 5;", "contract_version: 6;")
text = replace_once(
    text,
    "  wave: number;\n  completed_waves: number;\n  people: number;\n",
    "  wave: number;\n  completed_waves: number;\n  day_ticks_remaining: number;\n  is_night: boolean;\n  people: number;\n",
    "browser snapshot cycle fields",
)
path.write_text(text)


# Validate the new fields at the browser boundary and require contract v6.
path = Path("visualization/src/simulationClient.ts")
text = path.read_text()
if text.count("contract_version !== 5") != 2:
    raise SystemExit("simulation client: expected two v5 checks")
text = text.replace("contract_version !== 5", "contract_version !== 6")
text = replace_once(
    text,
    "  if (typeof value.houses_unlocked !== \"boolean\") {\n    throw new Error(\"snapshot house unlock state must be boolean\");\n  }\n\n  return value as SnapshotView;\n",
    "  if (typeof value.houses_unlocked !== \"boolean\") {\n    throw new Error(\"snapshot house unlock state must be boolean\");\n  }\n  if (typeof value.day_ticks_remaining !== \"number\" || typeof value.is_night !== \"boolean\") {\n    throw new Error(\"snapshot day/night state must be authoritative and typed\");\n  }\n\n  return value as SnapshotView;\n",
    "browser snapshot cycle validation",
)
path.write_text(text)


# Add the cycle timer and ensure simulation time advances even before economy exists.
path = Path("visualization/src/App.tsx")
text = path.read_text()
text = replace_once(
    text,
    "function towerName(archetype: TowerArchetype) {\n  return archetype === \"arrow\" ? \"Arrow tower\" : \"Cannon tower\";\n}\n\n",
    "function towerName(archetype: TowerArchetype) {\n  return archetype === \"arrow\" ? \"Arrow tower\" : \"Cannon tower\";\n}\n\nfunction formatCycleTimer(ticks: number) {\n  const totalSeconds = Math.ceil((ticks * TICK_INTERVAL_MS) / 1000);\n  const minutes = Math.floor(totalSeconds / 60);\n  const seconds = totalSeconds % 60;\n  return `${String(minutes).padStart(2, \"0\")}:${String(seconds).padStart(2, \"0\")}`;\n}\n\n",
    "cycle timer formatter",
)
text = replace_once(
    text,
    "  const raiderCount = snapshot?.entities.filter((entity) => entity.kind === \"raider\").length ?? 0;\n  const sawmillCount = snapshot?.entities.filter((entity) => entity.kind === \"sawmill\").length ?? 0;\n",
    "  const raiderCount = snapshot?.entities.filter((entity) => entity.kind === \"raider\").length ?? 0;\n  const townHealth = snapshot?.town_health;\n",
    "continuous simulation dependencies",
)
text = replace_once(
    text,
    "    if (!client || !snapshot || snapshot.town_health === 0 || (raiderCount === 0 && sawmillCount === 0)) {\n      return;\n    }\n",
    "    if (!client || townHealth === undefined || townHealth === 0) {\n      return;\n    }\n",
    "continuous simulation guard",
)
text = replace_once(
    text,
    "  }, [client, raiderCount, sawmillCount, snapshot?.town_health]);\n",
    "  }, [client, townHealth]);\n",
    "continuous simulation dependencies list",
)
text = replace_once(
    text,
    "  const canStartWave = raiderCount === 0 && snapshot.town_health > 0;\n",
    "  const canStartWave = !snapshot.is_night && snapshot.town_health > 0;\n",
    "authoritative wave start gate",
)
text = replace_once(
    text,
    "          <div>\n            <dt>Wave</dt>\n            <dd data-testid=\"wave-value\">{snapshot.wave}</dd>\n          </div>\n",
    "          <div>\n            <dt>Cycle</dt>\n            <dd data-testid=\"cycle-timer\">\n              {snapshot.is_night\n                ? `Night · Wave ${snapshot.wave}`\n                : `Day · ${formatCycleTimer(snapshot.day_ticks_remaining)}`}\n            </dd>\n          </div>\n          <div>\n            <dt>Wave</dt>\n            <dd data-testid=\"wave-value\">{snapshot.wave}</dd>\n          </div>\n",
    "cycle timer status",
)
path.write_text(text)


# Browser acceptance: the timer must move without requiring a Sawmill and switch to Night on raid start.
path = Path("visualization/e2e/app-screens.spec.ts")
text = path.read_text()
text = replace_once(
    text,
    "  await expect(page.getByTestId(\"completed-waves-value\")).toHaveText(\"0\");\n}\n",
    "  await expect(page.getByTestId(\"completed-waves-value\")).toHaveText(\"0\");\n  await expect(page.getByTestId(\"cycle-timer\")).toContainText(\"Day ·\");\n}\n",
    "initial cycle timer assertion",
)
text = replace_once(
    text,
    "test(\"sawmill wood waits for real people to carry it back to the Town Hall\", async ({ page }) => {\n",
    "test(\"cycle timer counts down through daytime and shows nighttime\", async ({ page }) => {\n  await openGame(page);\n\n  const initialTimer = await page.getByTestId(\"cycle-timer\").textContent();\n  await expect\n    .poll(async () => page.getByTestId(\"cycle-timer\").textContent(), { timeout: 3_000 })\n    .not.toBe(initialTimer);\n\n  await page.getByTestId(\"start-wave\").click();\n  await expect(page.getByTestId(\"cycle-timer\")).toHaveText(\"Night · Wave 1\");\n});\n\ntest(\"sawmill wood waits for real people to carry it back to the Town Hall\", async ({ page }) => {\n",
    "cycle timer browser test",
)
path.write_text(text)
