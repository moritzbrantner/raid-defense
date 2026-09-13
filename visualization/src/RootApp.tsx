import { useEffect, useState } from "react";
import App from "./App";
import { GameWiki } from "./GameWiki";
import {
  STANDARD_SCENARIO_OPTIONS,
  type ScenarioOptions,
} from "./scenarioOptions";
import {
  clearSavedGame,
  flushActiveSession,
  prepareNewGame,
  prepareResume,
  readSavedGameSummary,
  type SavedGameSummary,
} from "./simulationClient";
import "./StartMenu.css";

type Screen = "menu" | "game" | "settings";
type SettingsSection = "presentation" | "scenario";

type PresentationSettings = {
  showTouchHints: boolean;
  reduceUiMotion: boolean;
  compactStatus: boolean;
};

type ScenarioSelectProps = {
  label: string;
  description: string;
  value: string;
  options: readonly { value: string; label: string }[];
  onChange: (value: string) => void;
  testId: string;
};

const SETTINGS_KEY = "raid-defense.settings.v1";
const DEFAULT_SETTINGS: PresentationSettings = {
  showTouchHints: true,
  reduceUiMotion: false,
  compactStatus: false,
};

function readScreen(): Screen {
  const value = new URLSearchParams(window.location.search).get("screen");
  return value === "game" || value === "settings" ? value : "menu";
}

function readSettingsSection(): SettingsSection {
  return new URLSearchParams(window.location.search).get("section") === "scenario"
    ? "scenario"
    : "presentation";
}

function readSettings(): PresentationSettings {
  try {
    const raw = window.localStorage.getItem(SETTINGS_KEY);
    if (!raw) return DEFAULT_SETTINGS;
    const value = JSON.parse(raw) as Partial<PresentationSettings>;
    return {
      showTouchHints: value.showTouchHints ?? DEFAULT_SETTINGS.showTouchHints,
      reduceUiMotion: value.reduceUiMotion ?? DEFAULT_SETTINGS.reduceUiMotion,
      compactStatus: value.compactStatus ?? DEFAULT_SETTINGS.compactStatus,
    };
  } catch {
    return DEFAULT_SETTINGS;
  }
}

function createSeed() {
  const values = new Uint32Array(1);
  window.crypto.getRandomValues(values);
  return values[0] ?? 0x5eed;
}

function saveDescription(save: SavedGameSummary) {
  const progress = save.completed_waves > 0
    ? `${save.completed_waves} wave${save.completed_waves === 1 ? "" : "s"} cleared`
    : save.wave > 0
      ? `Wave ${save.wave}`
      : "Opening settlement";
  return `${progress} · tick ${save.tick}`;
}

function ScenarioSelect({
  label,
  description,
  value,
  options,
  onChange,
  testId,
}: ScenarioSelectProps) {
  return (
    <label className="setting-row scenario-setting-row">
      <span>
        <strong>{label}</strong>
        <small>{description}</small>
      </span>
      <select value={value} onChange={(event) => onChange(event.target.value)} data-testid={testId}>
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </label>
  );
}

export default function RootApp() {
  const [screen, setScreen] = useState<Screen>(() => readScreen());
  const [settingsSection, setSettingsSection] = useState<SettingsSection>(() => readSettingsSection());
  const [saveSummary, setSaveSummary] = useState<SavedGameSummary | null>(() =>
    readSavedGameSummary(),
  );
  const [settings, setSettings] = useState<PresentationSettings>(() => readSettings());
  const [scenario, setScenario] = useState<ScenarioOptions>(() => ({ ...STANDARD_SCENARIO_OPTIONS }));
  const [confirmNew, setConfirmNew] = useState(false);

  useEffect(() => {
    const url = new URL(window.location.href);
    if (!url.searchParams.has("screen")) {
      url.searchParams.set("screen", "menu");
      window.history.replaceState({}, "", url);
    }

    function onPopState() {
      flushActiveSession();
      setSaveSummary(readSavedGameSummary());
      setConfirmNew(false);
      setScreen(readScreen());
      setSettingsSection(readSettingsSection());
    }

    window.addEventListener("popstate", onPopState);
    return () => window.removeEventListener("popstate", onPopState);
  }, []);

  useEffect(() => {
    window.localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
    const root = document.documentElement;
    root.classList.toggle("hide-touch-hints", !settings.showTouchHints);
    root.classList.toggle("reduce-ui-motion", settings.reduceUiMotion);
    root.classList.toggle("compact-status", settings.compactStatus);
  }, [settings]);

  function navigate(next: Screen) {
    const url = new URL(window.location.href);
    url.searchParams.set("screen", next);
    url.searchParams.delete("new");
    url.searchParams.delete("seed");
    if (next !== "settings") url.searchParams.delete("section");
    window.history.pushState({}, "", url);
    setConfirmNew(false);
    setScreen(next);
  }

  function openSettingsSection(section: SettingsSection) {
    const url = new URL(window.location.href);
    url.searchParams.set("screen", "settings");
    url.searchParams.set("section", section);
    url.searchParams.delete("new");
    url.searchParams.delete("seed");
    window.history.pushState({}, "", url);
    setConfirmNew(false);
    setSettingsSection(section);
    setScreen("settings");
  }

  function updateScenario<K extends keyof ScenarioOptions>(key: K, value: ScenarioOptions[K]) {
    setScenario((current) => ({ ...current, [key]: value }));
  }

  function startNewGame() {
    prepareNewGame(createSeed(), scenario);
    setSaveSummary(null);
    navigate("game");
  }

  function resumeGame() {
    prepareResume();
    navigate("game");
  }

  function returnToMenu() {
    flushActiveSession();
    setSaveSummary(readSavedGameSummary());
    navigate("menu");
  }

  function deleteSave() {
    clearSavedGame();
    setSaveSummary(null);
    setConfirmNew(false);
  }

  if (screen === "game") {
    return (
      <>
        <App />
        <GameWiki />
        <button
          className="game-menu-button"
          type="button"
          onClick={returnToMenu}
          data-testid="return-to-menu"
        >
          Menu
        </button>
      </>
    );
  }

  if (screen === "settings") {
    return (
      <main className="front-door-shell settings-shell" data-testid="settings-screen">
        <section className="settings-panel" aria-labelledby="settings-title">
          <span className="menu-kicker">Raid Defense</span>
          <h1 id="settings-title">Settings</h1>

          <nav className="settings-sections" aria-label="Settings sections">
            <button
              type="button"
              className={settingsSection === "presentation" ? "active" : ""}
              onClick={() => openSettingsSection("presentation")}
              data-testid="settings-presentation-tab"
            >
              Presentation
            </button>
            <button
              type="button"
              className={settingsSection === "scenario" ? "active" : ""}
              onClick={() => openSettingsSection("scenario")}
              data-testid="settings-scenario-tab"
            >
              Scenario options
            </button>
          </nav>

          {settingsSection === "presentation" ? (
            <div data-testid="presentation-settings">
              <p className="menu-copy">
                Presentation preferences stay outside the authoritative simulation and can be changed at any time.
              </p>

              <label className="setting-row">
                <span>
                  <strong>Touch guidance</strong>
                  <small>Show the tap, drag, and pinch hint over the battlefield on phones.</small>
                </span>
                <input
                  type="checkbox"
                  checked={settings.showTouchHints}
                  onChange={(event) =>
                    setSettings((current) => ({ ...current, showTouchHints: event.target.checked }))
                  }
                  data-testid="setting-touch-hints"
                />
              </label>

              <label className="setting-row">
                <span>
                  <strong>Reduce UI motion</strong>
                  <small>Disable decorative UI transitions while keeping simulation timing unchanged.</small>
                </span>
                <input
                  type="checkbox"
                  checked={settings.reduceUiMotion}
                  onChange={(event) =>
                    setSettings((current) => ({ ...current, reduceUiMotion: event.target.checked }))
                  }
                  data-testid="setting-reduce-motion"
                />
              </label>

              <label className="setting-row">
                <span>
                  <strong>Compact status strip</strong>
                  <small>Use tighter spacing for settlement status information.</small>
                </span>
                <input
                  type="checkbox"
                  checked={settings.compactStatus}
                  onChange={(event) =>
                    setSettings((current) => ({ ...current, compactStatus: event.target.checked }))
                  }
                  data-testid="setting-compact-status"
                />
              </label>
            </div>
          ) : (
            <div data-testid="scenario-settings">
              <p className="menu-copy">
                Tune the next new game without changing the simulation authority. The chosen scenario is saved with
                that run, and Resume always reconstructs the same rules.
              </p>

              <ScenarioSelect
                label="Starting supplies"
                description="Begin with lean, standard, or rich Town Hall reserves."
                value={scenario.starting_supplies}
                options={[
                  { value: "lean", label: "Lean" },
                  { value: "standard", label: "Standard" },
                  { value: "rich", label: "Rich" },
                ]}
                onChange={(value) => updateScenario("starting_supplies", value as ScenarioOptions["starting_supplies"])}
                testId="scenario-starting-supplies"
              />

              <ScenarioSelect
                label="Forest coverage"
                description="Change how sparse or dense the seeded renewable forest is."
                value={scenario.forest_density}
                options={[
                  { value: "sparse", label: "Sparse" },
                  { value: "standard", label: "Standard" },
                  { value: "dense", label: "Dense" },
                ]}
                onChange={(value) => updateScenario("forest_density", value as ScenarioOptions["forest_density"])}
                testId="scenario-forest-density"
              />

              <ScenarioSelect
                label="Forest regrowth"
                description="Change how quickly depleted forest stock grows back."
                value={scenario.forest_regrowth}
                options={[
                  { value: "slow", label: "Slow" },
                  { value: "standard", label: "Standard" },
                  { value: "fast", label: "Fast" },
                ]}
                onChange={(value) => updateScenario("forest_regrowth", value as ScenarioOptions["forest_regrowth"])}
                testId="scenario-forest-regrowth"
              />

              <ScenarioSelect
                label="Sawmill throughput"
                description="Change how much wood a production cycle can harvest."
                value={scenario.sawmill_throughput}
                options={[
                  { value: "slow", label: "Low" },
                  { value: "standard", label: "Standard" },
                  { value: "fast", label: "High" },
                ]}
                onChange={(value) => updateScenario("sawmill_throughput", value as ScenarioOptions["sawmill_throughput"])}
                testId="scenario-sawmill-throughput"
              />

              <ScenarioSelect
                label="Raid size"
                description="Change how many raiders enter each wave."
                value={scenario.raid_size}
                options={[
                  { value: "small", label: "Small" },
                  { value: "standard", label: "Standard" },
                  { value: "large", label: "Large" },
                ]}
                onChange={(value) => updateScenario("raid_size", value as ScenarioOptions["raid_size"])}
                testId="scenario-raid-size"
              />

              <ScenarioSelect
                label="Raider strength"
                description="Change raider health, damage, and their wave-to-wave growth."
                value={scenario.raider_strength}
                options={[
                  { value: "gentle", label: "Gentle" },
                  { value: "standard", label: "Standard" },
                  { value: "harsh", label: "Harsh" },
                ]}
                onChange={(value) => updateScenario("raider_strength", value as ScenarioOptions["raider_strength"])}
                testId="scenario-raider-strength"
              />

              <ScenarioSelect
                label="Day length"
                description="Change the peaceful build-up time between automatic raids."
                value={scenario.day_length}
                options={[
                  { value: "short", label: "Short" },
                  { value: "standard", label: "Standard" },
                  { value: "long", label: "Long" },
                ]}
                onChange={(value) => updateScenario("day_length", value as ScenarioOptions["day_length"])}
                testId="scenario-day-length"
              />

              <ScenarioSelect
                label="Raid timing"
                description="Use the standard automatic cadence or start every raid manually."
                value={scenario.raid_timing}
                options={[
                  { value: "standard", label: "Automatic" },
                  { value: "manual", label: "Manual" },
                ]}
                onChange={(value) => updateScenario("raid_timing", value as ScenarioOptions["raid_timing"])}
                testId="scenario-raid-timing"
              />

              <ScenarioSelect
                label="Economy during raids"
                description="Keep the standard pause or let production and logistics continue during combat."
                value={scenario.raid_economy}
                options={[
                  { value: "standard", label: "Pause" },
                  { value: "continuous", label: "Continue" },
                ]}
                onChange={(value) => updateScenario("raid_economy", value as ScenarioOptions["raid_economy"])}
                testId="scenario-raid-economy"
              />

              {saveSummary ? (
                <p className="scenario-note">These choices affect the next new game only. The current save keeps its original scenario.</p>
              ) : null}
            </div>
          )}

          <div className="settings-actions">
            <button type="button" className="menu-button primary" onClick={() => navigate("menu")}>
              Back
            </button>
            <button
              type="button"
              className="menu-button subtle"
              onClick={() =>
                settingsSection === "presentation"
                  ? setSettings(DEFAULT_SETTINGS)
                  : setScenario({ ...STANDARD_SCENARIO_OPTIONS })
              }
            >
              {settingsSection === "presentation" ? "Reset presentation" : "Reset scenario"}
            </button>
            {saveSummary ? (
              <button type="button" className="menu-button danger" onClick={deleteSave}>
                Delete saved game
              </button>
            ) : null}
          </div>
        </section>
      </main>
    );
  }

  return (
    <main className="front-door-shell" data-testid="start-menu">
      <section className="start-menu" aria-labelledby="start-menu-title">
        <span className="menu-kicker">Deterministic settlement defense</span>
        <h1 id="start-menu-title">Raid Defense</h1>
        <p className="menu-copy">
          Build the supply chain before the raiders arrive. Forests, workers, storage, construction,
          and combat all run through the authoritative Rust simulation.
        </p>

        <div className="menu-actions">
          <button
            type="button"
            className="menu-button primary"
            disabled={!saveSummary}
            onClick={resumeGame}
            data-testid="resume-game"
          >
            Resume
          </button>
          {saveSummary ? <p className="save-summary">{saveDescription(saveSummary)}</p> : null}

          {!confirmNew ? (
            <button
              type="button"
              className="menu-button"
              onClick={() => (saveSummary ? setConfirmNew(true) : startNewGame())}
              data-testid="new-game"
            >
              New game
            </button>
          ) : (
            <div className="new-game-confirm" data-testid="new-game-confirmation">
              <p>Starting a new game replaces the current saved run.</p>
              <div>
                <button type="button" className="menu-button danger" onClick={startNewGame}>
                  Replace and start
                </button>
                <button type="button" className="menu-button subtle" onClick={() => setConfirmNew(false)}>
                  Cancel
                </button>
              </div>
            </div>
          )}

          <button
            type="button"
            className="menu-button"
            onClick={() => openSettingsSection("presentation")}
            data-testid="open-settings"
          >
            Settings
          </button>
        </div>
      </section>
    </main>
  );
}
