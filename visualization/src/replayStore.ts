import {
  createStandardScenarioOptions,
  isScenarioOptions,
  type ScenarioOptions,
} from "./scenarioOptions";
import type { RaidDefenseCommand, SnapshotView } from "./simulationTypes";

type JsonObject = Record<string, unknown>;
type PlayerCommand = Exclude<RaidDefenseCommand, { type: "advance_tick" }>;
type LegacyReplayEntry =
  | { kind: "advance_ticks"; count: number }
  | { kind: "command"; command: RaidDefenseCommand };

type LegacySavedGame = {
  version: 1 | 2;
  contract_version: 10;
  seed: number;
  entries: LegacyReplayEntry[];
  checksum: string;
  tick: number;
  wave: number;
  completed_waves: number;
  saved_at: number;
  scenario?: ScenarioOptions;
};

export type RecordedPlayerAction = {
  tick: number;
  sequence: number;
  player_id: number;
  command: PlayerCommand;
};

export type ReplayRecord = {
  version: 3;
  contract_version: 10;
  seed: number;
  scenario: ScenarioOptions;
  recorded_through_tick: number;
  actions: RecordedPlayerAction[];
};

export type SavedGameSummary = {
  seed: number;
  checksum: string;
  tick: number;
  wave: number;
  completed_waves: number;
  saved_at: number;
  action_count: number;
};

export type LoadedReplay = {
  replay: ReplayRecord;
  legacy_checksum?: string;
  legacy_summary?: SavedGameSummary;
};

export const LOCAL_PLAYER_ID = 0;

const REPLAY_KEY = "raid-defense.replay.v3";
const SUMMARY_KEY = "raid-defense.save-summary.v1";
const LEGACY_V2_KEY = "raid-defense.save.v2";
const LEGACY_V1_KEY = "raid-defense.save.v1";
const SAVE_CONTRACT_VERSION = 10;

function storageAvailable() {
  return typeof window !== "undefined" && typeof window.localStorage !== "undefined";
}

function isObject(value: unknown): value is JsonObject {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isSeed(value: unknown): value is number {
  return (
    typeof value === "number" &&
    Number.isInteger(value) &&
    value >= 0 &&
    value <= 0xffff_ffff
  );
}

function isNonNegativeInteger(value: unknown): value is number {
  return typeof value === "number" && Number.isSafeInteger(value) && value >= 0;
}

function isPlayerCommand(value: unknown): value is PlayerCommand {
  return (
    isObject(value) &&
    typeof value.type === "string" &&
    value.type !== "advance_tick"
  );
}

function isLegacyReplayEntry(value: unknown): value is LegacyReplayEntry {
  if (!isObject(value) || typeof value.kind !== "string") return false;
  if (value.kind === "advance_ticks") {
    return typeof value.count === "number" && Number.isInteger(value.count) && value.count > 0;
  }
  return (
    value.kind === "command" &&
    isObject(value.command) &&
    typeof value.command.type === "string"
  );
}

function parseReplayRecord(raw: string): ReplayRecord | null {
  try {
    const value: unknown = JSON.parse(raw);
    if (!isObject(value)) return null;
    if (value.version !== 3 || value.contract_version !== SAVE_CONTRACT_VERSION) return null;
    if (!isSeed(value.seed) || !isScenarioOptions(value.scenario)) return null;
    if (!isNonNegativeInteger(value.recorded_through_tick) || !Array.isArray(value.actions)) {
      return null;
    }

    let previousTick = 0;
    for (let index = 0; index < value.actions.length; index += 1) {
      const action = value.actions[index];
      if (!isObject(action)) return null;
      if (
        !isNonNegativeInteger(action.tick) ||
        action.tick > value.recorded_through_tick ||
        action.tick < previousTick ||
        action.sequence !== index ||
        !isNonNegativeInteger(action.player_id) ||
        !isPlayerCommand(action.command)
      ) {
        return null;
      }
      previousTick = action.tick;
    }

    return value as ReplayRecord;
  } catch {
    return null;
  }
}

function parseSummary(raw: string): SavedGameSummary | null {
  try {
    const value: unknown = JSON.parse(raw);
    if (!isObject(value)) return null;
    if (
      !isSeed(value.seed) ||
      typeof value.checksum !== "string" ||
      !isNonNegativeInteger(value.tick) ||
      !isNonNegativeInteger(value.wave) ||
      !isNonNegativeInteger(value.completed_waves) ||
      typeof value.saved_at !== "number" ||
      !Number.isFinite(value.saved_at) ||
      !isNonNegativeInteger(value.action_count)
    ) {
      return null;
    }
    return value as SavedGameSummary;
  } catch {
    return null;
  }
}

function parseLegacySavedGame(raw: string): LegacySavedGame | null {
  try {
    const value: unknown = JSON.parse(raw);
    if (!isObject(value)) return null;
    if (
      (value.version !== 1 && value.version !== 2) ||
      value.contract_version !== SAVE_CONTRACT_VERSION ||
      !isSeed(value.seed) ||
      !Array.isArray(value.entries) ||
      !value.entries.every(isLegacyReplayEntry) ||
      typeof value.checksum !== "string" ||
      !isNonNegativeInteger(value.tick) ||
      !isNonNegativeInteger(value.wave) ||
      !isNonNegativeInteger(value.completed_waves) ||
      typeof value.saved_at !== "number" ||
      !Number.isFinite(value.saved_at)
    ) {
      return null;
    }
    if (value.version === 2 && !isScenarioOptions(value.scenario)) return null;
    return value as LegacySavedGame;
  } catch {
    return null;
  }
}

function migrateLegacySave(save: LegacySavedGame): LoadedReplay | null {
  let tick = 0;
  const actions: RecordedPlayerAction[] = [];

  for (const entry of save.entries) {
    if (entry.kind === "advance_ticks") {
      tick += entry.count;
      continue;
    }
    if (entry.command.type === "advance_tick") {
      tick += 1;
      continue;
    }
    actions.push({
      tick,
      sequence: actions.length,
      player_id: LOCAL_PLAYER_ID,
      command: entry.command,
    });
  }

  if (tick !== save.tick) return null;
  const scenario = save.version === 2 && save.scenario ? save.scenario : createStandardScenarioOptions();
  return {
    replay: {
      version: 3,
      contract_version: SAVE_CONTRACT_VERSION,
      seed: save.seed,
      scenario: JSON.parse(JSON.stringify(scenario)) as ScenarioOptions,
      recorded_through_tick: tick,
      actions,
    },
    legacy_checksum: save.checksum,
    legacy_summary: {
      seed: save.seed,
      checksum: save.checksum,
      tick: save.tick,
      wave: save.wave,
      completed_waves: save.completed_waves,
      saved_at: save.saved_at,
      action_count: actions.length,
    },
  };
}

export function createReplayRecord(seed: number, scenario: ScenarioOptions): ReplayRecord {
  return {
    version: 3,
    contract_version: SAVE_CONTRACT_VERSION,
    seed: seed >>> 0,
    scenario: JSON.parse(JSON.stringify(scenario)) as ScenarioOptions,
    recorded_through_tick: 0,
    actions: [],
  };
}

export function readStoredReplay(): LoadedReplay | null {
  if (!storageAvailable()) return null;

  const current = window.localStorage.getItem(REPLAY_KEY);
  if (current !== null) {
    const replay = parseReplayRecord(current);
    return replay ? { replay } : null;
  }

  const legacyV2 = window.localStorage.getItem(LEGACY_V2_KEY);
  if (legacyV2 !== null) {
    const parsed = parseLegacySavedGame(legacyV2);
    return parsed ? migrateLegacySave(parsed) : null;
  }

  const legacyV1 = window.localStorage.getItem(LEGACY_V1_KEY);
  if (legacyV1 !== null) {
    const parsed = parseLegacySavedGame(legacyV1);
    return parsed ? migrateLegacySave(parsed) : null;
  }

  return null;
}

export function readSavedGameSummary(): SavedGameSummary | null {
  const loaded = readStoredReplay();
  if (!loaded) return null;
  if (loaded.legacy_summary) return loaded.legacy_summary;

  if (storageAvailable()) {
    const raw = window.localStorage.getItem(SUMMARY_KEY);
    const cached = raw ? parseSummary(raw) : null;
    if (
      cached &&
      cached.seed === loaded.replay.seed &&
      cached.tick === loaded.replay.recorded_through_tick &&
      cached.action_count === loaded.replay.actions.length
    ) {
      return cached;
    }
  }

  return {
    seed: loaded.replay.seed,
    checksum: "",
    tick: loaded.replay.recorded_through_tick,
    wave: 0,
    completed_waves: 0,
    saved_at: 0,
    action_count: loaded.replay.actions.length,
  };
}

export function summaryFromSnapshot(
  replay: ReplayRecord,
  snapshot: SnapshotView,
): SavedGameSummary {
  return {
    seed: replay.seed,
    checksum: snapshot.checksum,
    tick: snapshot.tick,
    wave: snapshot.wave,
    completed_waves: snapshot.completed_waves,
    saved_at: Date.now(),
    action_count: replay.actions.length,
  };
}

export function writeStoredReplay(replay: ReplayRecord, summary: SavedGameSummary) {
  if (!storageAvailable()) return;
  try {
    window.localStorage.setItem(REPLAY_KEY, JSON.stringify(replay));
    window.localStorage.setItem(SUMMARY_KEY, JSON.stringify(summary));
    window.localStorage.removeItem(LEGACY_V2_KEY);
    window.localStorage.removeItem(LEGACY_V1_KEY);
  } catch (error) {
    console.warn("Unable to persist Raid Defense replay", error);
  }
}

export function clearStoredReplay() {
  if (!storageAvailable()) return;
  window.localStorage.removeItem(REPLAY_KEY);
  window.localStorage.removeItem(SUMMARY_KEY);
  window.localStorage.removeItem(LEGACY_V2_KEY);
  window.localStorage.removeItem(LEGACY_V1_KEY);
}
