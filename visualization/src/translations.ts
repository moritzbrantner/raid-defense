import type { TranslationResources } from "@moritzbrantner/i18n";

export const supportedLocales = ["en", "de"] as const;
export type AppLocale = (typeof supportedLocales)[number];

export const translations = {
  en: {
    game: {
      menu: "Menu",
    },
    menu: {
      cancel: "Cancel",
      description:
        "Build the supply chain before the raiders arrive. Forests, workers, storage, construction, and combat all run through the authoritative Rust simulation.",
      kicker: "Deterministic settlement defense",
      newGame: "New game",
      replaceAndStart: "Replace and start",
      replaceWarning: "Starting a new game replaces the current saved run.",
      resume: "Resume",
      save: {
        openingSettlement: "Opening settlement",
        summary: "{{progress}} · tick {{tick}}",
        wave: "Wave {{wave}}",
        wavesCleared_one: "{{count}} wave cleared",
        wavesCleared_other: "{{count}} waves cleared",
      },
      settings: "Settings",
      title: "Raid Defense",
    },
    settings: {
      actions: {
        back: "Back",
        deleteSavedGame: "Delete saved game",
        resetPresentation: "Reset presentation",
        resetScenario: "Reset scenario",
      },
      compactStatus: {
        description: "Use tighter spacing for settlement status information.",
        title: "Compact status strip",
      },
      intro:
        "Presentation preferences stay outside the authoritative simulation and can be changed at any time.",
      language: {
        description: "Choose the interface language. The choice is kept in the URL so the page remains shareable.",
        german: "Deutsch",
        english: "English",
        title: "Language",
      },
      presentation: "Presentation",
      reduceMotion: {
        description: "Disable decorative UI transitions while keeping simulation timing unchanged.",
        title: "Reduce UI motion",
      },
      scenarioEditor: "Scenario editor",
      sectionsLabel: "Settings sections",
      title: "Settings",
      touchGuidance: {
        description: "Show the tap, drag, and pinch hint over the battlefield on phones.",
        title: "Touch guidance",
      },
    },
  },
  de: {
    game: {
      menu: "Menü",
    },
    menu: {
      cancel: "Abbrechen",
      description:
        "Baue die Versorgungskette auf, bevor die Plünderer eintreffen. Wälder, Arbeiter, Lager, Bau und Kampf laufen vollständig über die maßgebliche Rust-Simulation.",
      kicker: "Deterministische Siedlungsverteidigung",
      newGame: "Neues Spiel",
      replaceAndStart: "Ersetzen und starten",
      replaceWarning: "Ein neues Spiel ersetzt den derzeit gespeicherten Spielstand.",
      resume: "Fortsetzen",
      save: {
        openingSettlement: "Siedlung im Aufbau",
        summary: "{{progress}} · Tick {{tick}}",
        wave: "Welle {{wave}}",
        wavesCleared_one: "{{count}} Welle abgeschlossen",
        wavesCleared_other: "{{count}} Wellen abgeschlossen",
      },
      settings: "Einstellungen",
      title: "Raid Defense",
    },
    settings: {
      actions: {
        back: "Zurück",
        deleteSavedGame: "Spielstand löschen",
        resetPresentation: "Darstellung zurücksetzen",
        resetScenario: "Szenario zurücksetzen",
      },
      compactStatus: {
        description: "Verwende engere Abstände für die Statusinformationen der Siedlung.",
        title: "Kompakte Statusleiste",
      },
      intro:
        "Darstellungseinstellungen bleiben außerhalb der maßgeblichen Simulation und können jederzeit geändert werden.",
      language: {
        description: "Wähle die Sprache der Oberfläche. Die Auswahl bleibt in der URL erhalten, damit die Seite teilbar bleibt.",
        german: "Deutsch",
        english: "English",
        title: "Sprache",
      },
      presentation: "Darstellung",
      reduceMotion: {
        description: "Deaktiviere dekorative UI-Übergänge, ohne das Timing der Simulation zu verändern.",
        title: "UI-Bewegung reduzieren",
      },
      scenarioEditor: "Szenarioeditor",
      sectionsLabel: "Einstellungsbereiche",
      title: "Einstellungen",
      touchGuidance: {
        description: "Zeige auf Smartphones Hinweise für Tippen, Ziehen und Zoomen über dem Spielfeld.",
        title: "Touch-Hinweise",
      },
    },
  },
} satisfies TranslationResources<AppLocale>;
