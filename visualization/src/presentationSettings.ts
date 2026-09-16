import {
  createSettingsSession,
  type BrowserSettingsSession,
  type PresentationEntry,
  type SettingDefinition,
  type SettingValue,
} from "@moritzbrantner/settings-browser";

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

export type SettingsFoundationSession = BrowserSettingsSession;
type StorageLike = Pick<Storage, "getItem" | "setItem">;
type BooleanSettingDefinition = SettingDefinition & {
  kind: { type: "bool" };
  default: Extract<SettingValue, { type: "bool" }>;
};

export const presentationSettingIds = {
  showTouchHints: "presentation.show_touch_hints",
  reduceUiMotion: "accessibility.reduce_motion",
  compactStatus: "presentation.compact_status",
} as const;

export const presentationSettingDefinitions = {
  showTouchHints: boolDefinition(presentationSettingIds.showTouchHints, true),
  reduceUiMotion: boolDefinition(presentationSettingIds.reduceUiMotion, false),
  compactStatus: boolDefinition(presentationSettingIds.compactStatus, false),
} as const;

export const presentationEntries = {
  showTouchHints: presentationEntry(
    presentationSettingIds.showTouchHints,
    "settings.touchGuidance.title",
    "settings.touchGuidance.description",
    10,
  ),
  reduceUiMotion: presentationEntry(
    presentationSettingIds.reduceUiMotion,
    "settings.reduceMotion.title",
    "settings.reduceMotion.description",
    20,
  ),
  compactStatus: presentationEntry(
    presentationSettingIds.compactStatus,
    "settings.compactStatus.title",
    "settings.compactStatus.description",
    30,
  ),
} as const;

export async function createPresentationSettingsFoundation(
  initialSettings: PresentationSettings,
  storage?: StorageLike,
): Promise<{
  session: SettingsFoundationSession;
  settings: PresentationSettings;
  diagnostics: string[];
}> {
  const session = await createSettingsSession(
    Object.values(presentationSettingDefinitions),
    Object.values(presentationEntries),
  );
  const resolvedStorage = storage ?? getBrowserStorage();
  let diagnostics: string[] = [];
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
  values: Readonly<Record<string, SettingValue>>,
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

function getBrowserStorage(): StorageLike | undefined {
  try {
    return window.localStorage;
  } catch (error) {
    console.warn("Browser storage is unavailable for presentation settings", error);
    return undefined;
  }
}

function boolDefinition(id: string, value: boolean): BooleanSettingDefinition {
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
): PresentationEntry {
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

function bool(value: boolean): Extract<SettingValue, { type: "bool" }> {
  return { type: "bool", value };
}

function boolValue(value: SettingValue | undefined, fallback: boolean) {
  return value?.type === "bool" ? value.value : fallback;
}
