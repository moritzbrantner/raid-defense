import { useLocale, useTranslation } from "@moritzbrantner/i18n/react";
import { SettingsBooleanField } from "@moritzbrantner/settings-browser/react";
import { useEffect, useRef, useState } from "react";
import App from "./App";
import { GameWiki } from "./GameWiki";
import {
  createPresentationSettingsFoundation,
  DEFAULT_PRESENTATION_SETTINGS,
  persistPresentationSettingsFoundation,
  presentationEntries,
  presentationSettingDefinitions,
  readLegacyPresentationSettings,
  writeLegacyPresentationSettings,
  type PresentationSettings,
  type SettingsFoundationSession,
} from "./presentationSettings";
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
import type { AppLocale } from "./translations";
import "@moritzbrantner/ui/styles.css";
import "@moritzbrantner/ui/component-sources.css";
import "./StartMenu.css";

type Screen = "menu" | "game" | "settings";
type SettingsSection = "presentation" | "scenario";

function readScreen(): Screen {
  const value = new URLSearchParams(window.location.search).get("screen");
  return value === "game" || value === "settings" ? value : "menu";
}

function readSettingsSection(): SettingsSection {
  return new URLSearchParams(window.location.search).get("section") === "scenario"
    ? "scenario"
    : "presentation";
}

function createSeed() {
  const values = new Uint32Array(1);
  window.crypto.getRandomValues(values);
  return values[0] ?? 0x5eed;
}

export default function RootApp() {
  const { t } = useTranslation();
  const { locale, setLocale } = useLocale<AppLocale>();
  const [screen, setScreen] = useState<Screen>(() => readScreen());
  const [settingsSection, setSettingsSection] = useState<SettingsSection>(() => readSettingsSection());
  const [saveSummary, setSaveSummary] = useState<SavedGameSummary | null>(() =>
    readSavedGameSummary(),
  );
  const [settings, setSettings] = useState<PresentationSettings>(() =>
    readLegacyPresentationSettings(),
  );
  const initialSettingsRef = useRef(settings);
  const latestSettingsRef = useRef(settings);
  const userEditedBeforeFoundationReadyRef = useRef(false);
  const settingsFoundationRef = useRef<SettingsFoundationSession | null>(null);
  const [scenario, setScenario] = useState<ScenarioOptions>(() => createStandardScenarioOptions());
  const [confirmNew, setConfirmNew] = useState(false);
  const settingMessages: Record<string, string> = {
    "settings.touchGuidance.title": t("settings.touchGuidance.title"),
    "settings.touchGuidance.description": t("settings.touchGuidance.description"),
    "settings.reduceMotion.title": t("settings.reduceMotion.title"),
    "settings.reduceMotion.description": t("settings.reduceMotion.description"),
    "settings.compactStatus.title": t("settings.compactStatus.title"),
    "settings.compactStatus.description": t("settings.compactStatus.description"),
  };

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
    void createPresentationSettingsFoundation(initialSettingsRef.current)
      .then(({ session, settings: restoredSettings, diagnostics }) => {
        if (cancelled) {
          session.dispose();
          return;
        }
        settingsFoundationRef.current = session;
        if (diagnostics.length > 0) {
          console.info("Shared presentation settings recovered with diagnostics", diagnostics);
        }

        if (userEditedBeforeFoundationReadyRef.current) {
          persistPresentationSettingsFoundation(session, latestSettingsRef.current);
          userEditedBeforeFoundationReadyRef.current = false;
        } else {
          latestSettingsRef.current = restoredSettings;
          setSettings(restoredSettings);
        }
      })
      .catch((error) => {
        console.warn(
          "Shared settings foundation unavailable; continuing with the local presentation projection.",
          error,
        );
      });

    return () => {
      cancelled = true;
      settingsFoundationRef.current?.dispose();
      settingsFoundationRef.current = null;
    };
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
    latestSettingsRef.current = settings;
    const session = settingsFoundationRef.current;
    if (session) {
      persistPresentationSettingsFoundation(session, settings);
    } else {
      writeLegacyPresentationSettings(settings);
    }
    const root = document.documentElement;
    root.classList.toggle("hide-touch-hints", !settings.showTouchHints);
    root.classList.toggle("reduce-ui-motion", settings.reduceUiMotion);
    root.classList.toggle("compact-status", settings.compactStatus);
  }, [settings]);

  function updatePresentationSettings(
    update: (current: PresentationSettings) => PresentationSettings,
  ) {
    setSettings((current) => {
      const next = update(current);
      latestSettingsRef.current = next;
      if (!settingsFoundationRef.current) {
        userEditedBeforeFoundationReadyRef.current = true;
      }
      return next;
    });
  }

  function localizeSetting(key: string) {
    return settingMessages[key] ?? key;
  }

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

  function describeSave(save: SavedGameSummary) {
    const progress =
      save.completed_waves > 0
        ? t("menu.save.wavesCleared", { count: save.completed_waves })
        : save.wave > 0
          ? t("menu.save.wave", { wave: save.wave })
          : t("menu.save.openingSettlement");
    return t("menu.save.summary", { progress, tick: save.tick });
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
          {t("game.menu")}
        </button>
      </>
    );
  }

  if (screen === "settings") {
    return (
      <main className="front-door-shell settings-shell" data-testid="settings-screen">
        <section className="settings-panel" aria-labelledby="settings-title">
          <span className="menu-kicker">Raid Defense</span>
          <h1 id="settings-title">{t("settings.title")}</h1>

          <nav className="settings-sections" aria-label={t("settings.sectionsLabel")}>
            <button
              type="button"
              className={settingsSection === "presentation" ? "active" : ""}
              onClick={() => openSettingsSection("presentation")}
              data-testid="settings-presentation-tab"
            >
              {t("settings.presentation")}
            </button>
            <button
              type="button"
              className={settingsSection === "scenario" ? "active" : ""}
              onClick={() => openSettingsSection("scenario")}
              data-testid="settings-scenario-tab"
            >
              {t("settings.scenarioEditor")}
            </button>
          </nav>

          {settingsSection === "presentation" ? (
            <div data-testid="presentation-settings">
              <p className="menu-copy">{t("settings.intro")}</p>

              <label className="setting-row">
                <span>
                  <strong>{t("settings.language.title")}</strong>
                  <small>{t("settings.language.description")}</small>
                </span>
                <select
                  value={locale}
                  onChange={(event) => void setLocale(event.target.value as AppLocale)}
                  data-testid="setting-language"
                >
                  <option value="en">{t("settings.language.english")}</option>
                  <option value="de">{t("settings.language.german")}</option>
                </select>
              </label>

              <div className="shared-settings-fields" data-testid="shared-settings-fields">
                <SettingsBooleanField
                  definition={presentationSettingDefinitions.showTouchHints}
                  presentation={presentationEntries.showTouchHints}
                  value={{ type: "bool", value: settings.showTouchHints }}
                  localize={localizeSetting}
                  onValueChange={(checked) =>
                    updatePresentationSettings((current) => ({
                      ...current,
                      showTouchHints: checked,
                    }))
                  }
                  switchProps={{ "data-testid": "setting-touch-hints" }}
                />
                <SettingsBooleanField
                  definition={presentationSettingDefinitions.reduceUiMotion}
                  presentation={presentationEntries.reduceUiMotion}
                  value={{ type: "bool", value: settings.reduceUiMotion }}
                  localize={localizeSetting}
                  onValueChange={(checked) =>
                    updatePresentationSettings((current) => ({
                      ...current,
                      reduceUiMotion: checked,
                    }))
                  }
                  switchProps={{ "data-testid": "setting-reduce-motion" }}
                />
                <SettingsBooleanField
                  definition={presentationSettingDefinitions.compactStatus}
                  presentation={presentationEntries.compactStatus}
                  value={{ type: "bool", value: settings.compactStatus }}
                  localize={localizeSetting}
                  onValueChange={(checked) =>
                    updatePresentationSettings((current) => ({
                      ...current,
                      compactStatus: checked,
                    }))
                  }
                  switchProps={{ "data-testid": "setting-compact-status" }}
                />
              </div>
            </div>
          ) : (
            <div data-testid="scenario-settings">
              <ScenarioEditor scenario={scenario} onChange={setScenario} hasSave={Boolean(saveSummary)} />
            </div>
          )}

          <div className="settings-actions">
            <button
              type="button"
              className="menu-button primary"
              onClick={() => navigate("menu")}
              data-testid="settings-back"
            >
              {t("settings.actions.back")}
            </button>
            <button
              type="button"
              className="menu-button subtle"
              onClick={() =>
                settingsSection === "presentation"
                  ? updatePresentationSettings(() => DEFAULT_PRESENTATION_SETTINGS)
                  : resetScenario()
              }
            >
              {settingsSection === "presentation"
                ? t("settings.actions.resetPresentation")
                : t("settings.actions.resetScenario")}
            </button>
            {saveSummary ? (
              <button type="button" className="menu-button danger" onClick={deleteSave}>
                {t("settings.actions.deleteSavedGame")}
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
        <span className="menu-kicker">{t("menu.kicker")}</span>
        <h1 id="start-menu-title">{t("menu.title")}</h1>
        <p className="menu-copy">{t("menu.description")}</p>

        <div className="menu-actions">
          <button
            type="button"
            className="menu-button primary"
            disabled={!saveSummary}
            onClick={resumeGame}
            data-testid="resume-game"
          >
            {t("menu.resume")}
          </button>
          {saveSummary ? <p className="save-summary">{describeSave(saveSummary)}</p> : null}

          {!confirmNew ? (
            <button
              type="button"
              className="menu-button"
              onClick={() => (saveSummary ? setConfirmNew(true) : startNewGame())}
              data-testid="new-game"
            >
              {t("menu.newGame")}
            </button>
          ) : (
            <div className="new-game-confirm" data-testid="new-game-confirmation">
              <p>{t("menu.replaceWarning")}</p>
              <div>
                <button type="button" className="menu-button danger" onClick={startNewGame}>
                  {t("menu.replaceAndStart")}
                </button>
                <button type="button" className="menu-button subtle" onClick={() => setConfirmNew(false)}>
                  {t("menu.cancel")}
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
            {t("menu.settings")}
          </button>
        </div>
      </section>
    </main>
  );
}
