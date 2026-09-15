import { useEffect, useState } from "react";
import App from "./App";
import { GameWiki } from "./GameWiki";
import { ScenarioEditor } from "./ScenarioEditor";
import {
  createStandardScenarioOptions,
  type ScenarioOptions,
} from "./scenarioOptions";
import {
  clearSavedGame,
  flushActiveSession,
  loadStandardScenarioWorld,
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

export default function RootApp() {
  const [screen, setScreen] = useState<Screen>(() => readScreen());
  const [settingsSection, setSettingsSection] = useState<SettingsSection>(() => readSettingsSection());
  const [saveSummary, setSaveSummary] = useState<SavedGameSummary | null>(() =>
    readSavedGameSummary(),
  );
  const [settings, setSettings] = useState<PresentationSettings>(() => readSettings());
  const [scenario, setScenario] = useState<ScenarioOptions>(() => createStandardScenarioOptions());
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
    let cancelled = false;
    void loadStandardScenarioWorld()
      .then((world) => {
        if (!cancelled) {
          setScenario((current) => current.world ? current : { ...current, world });
        }
      })
      .catch((error) => console.error("Unable to load authoritative world defaults", error));
    return () => {
      cancelled = true;
    };
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
    if (next !== "settings") {
      url.searchParams.delete("section");
      url.searchParams.delete("scenarioView");
    }
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
    if (section !== "scenario") url.searchParams.delete("scenarioView");
    window.history.pushState({}, "", url);
    setConfirmNew(false);
    setSettingsSection(section);
    setScreen("settings");
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

  function resetScenario() {
    const reset = createStandardScenarioOptions();
    setScenario(reset);
    void loadStandardScenarioWorld()
      .then((world) => {
        setScenario((current) => current === reset ? { ...reset, world } : current);
      })
      .catch((error) => console.error("Unable to reset authoritative world defaults", error));
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
              Scenario editor
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
              <ScenarioEditor scenario={scenario} onChange={setScenario} hasSave={Boolean(saveSummary)} />
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
                  : resetScenario()
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
