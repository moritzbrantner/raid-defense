export type PresentationSettings = {
  showTouchHints: boolean;
  reduceUiMotion: boolean;
  compactStatus: boolean;
};

export const DEFAULT_PRESENTATION_SETTINGS: PresentationSettings = {
  showTouchHints: true,
  reduceUiMotion: false,
  compactStatus: false,
};

export const LEGACY_PRESENTATION_SETTINGS_STORAGE_KEY = "raid-defense.settings.v1";
export const SHARED_PRESENTATION_SETTINGS_STORAGE_KEY = "raid-defense.settings.user.v2";

// Pinned to an immutable generated settings browser distribution. This is deliberately
// not the mutable browser-dist branch or Pages URL.
export const SETTINGS_BROWSER_BUNDLE_URL =
  "https://cdn.jsdelivr.net/gh/moritzbrantner/settings@5e6b7383b14f1549154d2100f5df3d39d42d66ab/settings-browser.js";

type WireSettingValue =
  | { type: "bool"; value: boolean }
  | { type: "integer"; value: number }
  | { type: "number"; value: number }
  | { type: "text"; value: string }
  | { type: "choice"; value: string };

type WireSettingDefinition = {
  id: string;
  kind:
    | { type: "bool" }
    | { type: "integer"; min: number; max: number }
    | { type: "number"; min: number; max: number }
    | { type: "text"; min_chars: number; max_chars: number }
    | { type: "choice"; options: string[] };
  default: WireSettingValue;
  scope: "session" | "save" | "device" | "user";
  apply_mode: "immediate" | "apply" | "restart" | "reconnect";
  availability?: unknown;
};

type WirePresentationEntry = {
  id: string;
  metadata: {
    label_key: string;
    description_key?: string;
    category_key: string;
    group_key?: string;
    order?: number;
    discoverability?: "primary" | "advanced" | "search_only";
    search_keys?: string[];
  };
};

export type SettingsFoundationSession = {
  presentation(): WirePresentationEntry[];
  effectiveValues(): Record<string, WireSettingValue>;
  set(id: string, value: WireSettingValue): void;
  reset(id: string): void;
  importScope(scope: "save" | "device" | "user", snapshot: string): unknown[];
  exportScope(scope: "save" | "device" | "user"): string;
  dispose(): void;
};

type SettingsBrowserModule = {
  createSettingsSession(
    definitions: readonly WireSettingDefinition[],
    presentation?: readonly WirePresentationEntry[],
  ): Promise<SettingsFoundationSession>;
};

type StorageLike = Pick<Storage, "getItem" | "setItem">;

export const presentationSettingIds = {
  showTouchHints: "presentation.show_touch_hints",
  reduceUiMotion: "accessibility.reduce_motion",
  compactStatus: "presentation.compact_status",
} as const;

const presentationSettingDefinitions: readonly WireSettingDefinition[] = [
  boolDefinition(presentationSettingIds.showTouchHints, true),
  boolDefinition(presentationSettingIds.reduceUiMotion, false),
  boolDefinition(presentationSettingIds.compactStatus, false),
];

const presentationMetadata: readonly WirePresentationEntry[] = [
  presentationEntry(
    presentationSettingIds.showTouchHints,
    "settings.touchGuidance.title",
    "settings.touchGuidance.description",
    10,
  ),
  presentationEntry(
    presentationSettingIds.reduceUiMotion,
    "settings.reduceMotion.title",
    "settings.reduceMotion.description",
    20,
  ),
  presentationEntry(
    presentationSettingIds.compactStatus,
    "settings.compactStatus.title",
    "settings.compactStatus.description",
    30,
  ),
];

let browserModulePromise: Promise<SettingsBrowserModule> | undefined;

export async function createPresentationSettingsFoundation(
  initialSettings: PresentationSettings,
  storage?: StorageLike,
): Promise<{
  session: SettingsFoundationSession;
  settings: PresentationSettings;
  diagnostics: unknown[];
}> {
  const browserModule = await loadSettingsBrowserModule();
  const session = await browserModule.createSettingsSession(
    presentationSettingDefinitions,
    presentationMetadata,
  );
  const resolvedStorage = storage ?? getBrowserStorage();
  let diagnostics: unknown[] = [];
  let storedSnapshot: string | null = null;

  if (resolvedStorage) {
    try {
      storedSnapshot = resolvedStorage.getItem(SHARED_PRESENTATION_SETTINGS_STORAGE_KEY);
    } catch (error) {
      console.warn("Shared presentation settings storage is unavailable", error);
    }
  }

  if (storedSnapshot) {
    try {
      diagnostics = session.importScope("user", storedSnapshot);
    } catch (error) {
      console.warn(
        "Ignoring an unreadable shared settings snapshot and migrating the legacy presentation preferences.",
        error,
      );
      syncPresentationSettingsToFoundation(session, initialSettings);
    }
  } else {
    syncPresentationSettingsToFoundation(session, initialSettings);
  }

  const settings = materializePresentationSettings(session.effectiveValues(), initialSettings);
  persistPresentationSettingsFoundation(session, settings, resolvedStorage);
  return { session, settings, diagnostics };
}

export function readLegacyPresentationSettings(
  storage?: Pick<Storage, "getItem">,
): PresentationSettings {
  try {
    const resolvedStorage = storage ?? getBrowserStorage();
    const raw = resolvedStorage?.getItem(LEGACY_PRESENTATION_SETTINGS_STORAGE_KEY);
    if (!raw) return DEFAULT_PRESENTATION_SETTINGS;
    const value = JSON.parse(raw) as Partial<PresentationSettings>;
    return {
      showTouchHints:
        typeof value.showTouchHints === "boolean"
          ? value.showTouchHints
          : DEFAULT_PRESENTATION_SETTINGS.showTouchHints,
      reduceUiMotion:
        typeof value.reduceUiMotion === "boolean"
          ? value.reduceUiMotion
          : DEFAULT_PRESENTATION_SETTINGS.reduceUiMotion,
      compactStatus:
        typeof value.compactStatus === "boolean"
          ? value.compactStatus
          : DEFAULT_PRESENTATION_SETTINGS.compactStatus,
    };
  } catch {
    return DEFAULT_PRESENTATION_SETTINGS;
  }
}

export function persistPresentationSettingsFoundation(
  session: SettingsFoundationSession,
  settings: PresentationSettings,
  storage?: Pick<Storage, "setItem">,
) {
  syncPresentationSettingsToFoundation(session, settings);
  const resolvedStorage = storage ?? getBrowserStorage();
  let sharedPersisted = false;

  if (resolvedStorage) {
    try {
      resolvedStorage.setItem(
        SHARED_PRESENTATION_SETTINGS_STORAGE_KEY,
        session.exportScope("user"),
      );
      sharedPersisted = true;
    } catch (error) {
      console.warn("Shared presentation settings snapshot could not be persisted", error);
    }
  }

  const legacyPersisted = writeLegacyPresentationSettings(settings, resolvedStorage);
  return sharedPersisted && legacyPersisted;
}

export function writeLegacyPresentationSettings(
  settings: PresentationSettings,
  storage?: Pick<Storage, "setItem">,
) {
  try {
    const resolvedStorage = storage ?? getBrowserStorage();
    if (!resolvedStorage) return false;
    // Retained only as a degraded-mode/downgrade projection. The shared snapshot is
    // authoritative whenever the settings foundation is available.
    resolvedStorage.setItem(LEGACY_PRESENTATION_SETTINGS_STORAGE_KEY, JSON.stringify(settings));
    return true;
  } catch (error) {
    console.warn("Local presentation settings projection could not be persisted", error);
    return false;
  }
}

export function materializePresentationSettings(
  values: Readonly<Record<string, WireSettingValue>>,
  fallback: PresentationSettings = DEFAULT_PRESENTATION_SETTINGS,
): PresentationSettings {
  return {
    showTouchHints: boolValue(values[presentationSettingIds.showTouchHints], fallback.showTouchHints),
    reduceUiMotion: boolValue(values[presentationSettingIds.reduceUiMotion], fallback.reduceUiMotion),
    compactStatus: boolValue(values[presentationSettingIds.compactStatus], fallback.compactStatus),
  };
}

function syncPresentationSettingsToFoundation(
  session: SettingsFoundationSession,
  settings: PresentationSettings,
) {
  session.set(presentationSettingIds.showTouchHints, bool(settings.showTouchHints));
  session.set(presentationSettingIds.reduceUiMotion, bool(settings.reduceUiMotion));
  session.set(presentationSettingIds.compactStatus, bool(settings.compactStatus));
}

async function loadSettingsBrowserModule(): Promise<SettingsBrowserModule> {
  browserModulePromise ??= import(
    /* @vite-ignore */ SETTINGS_BROWSER_BUNDLE_URL
  ) as Promise<SettingsBrowserModule>;
  return browserModulePromise;
}

function getBrowserStorage(): StorageLike | undefined {
  try {
    return window.localStorage;
  } catch (error) {
    console.warn("Browser storage is unavailable for presentation settings", error);
    return undefined;
  }
}

function boolDefinition(id: string, value: boolean): WireSettingDefinition {
  return {
    id,
    kind: { type: "bool" },
    default: bool(value),
    scope: "user",
    apply_mode: "immediate",
  };
}

function presentationEntry(
  id: string,
  labelKey: string,
  descriptionKey: string,
  order: number,
): WirePresentationEntry {
  return {
    id,
    metadata: {
      label_key: labelKey,
      description_key: descriptionKey,
      category_key: "settings.presentation",
      order,
      discoverability: "primary",
    },
  };
}

function bool(value: boolean): WireSettingValue {
  return { type: "bool", value };
}

function boolValue(value: WireSettingValue | undefined, fallback: boolean) {
  return value?.type === "bool" ? value.value : fallback;
}
