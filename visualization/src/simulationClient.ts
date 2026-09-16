import type {
  DispatchResponse,
  RaidDefenseCommand,
  SnapshotView,
} from "./simulationTypes";
import {
  createStandardScenarioOptions,
  type ScenarioOptions,
  type ScenarioWorldRules,
} from "./scenarioOptions";
import {
  clearStoredReplay,
  createReplayRecord,
  LOCAL_PLAYER_ID,
  readSavedGameSummary as readReplaySummary,
  readStoredReplay,
  summaryFromSnapshot,
  writeStoredReplay,
  type LoadedReplay,
  type ReplayRecord,
  type SavedGameSummary,
} from "./replayStore";

export type { SavedGameSummary } from "./replayStore";

type WasmModule = typeof import("./generated/raid-defense-wasm/raid_defense_wasm.js");
type JsonObject = Record<string, unknown>;
type SessionIntent =
  | { kind: "new"; seed: number; scenario: ScenarioOptions }
  | { kind: "resume" };

const WASM_CONTRACT_VERSION = 10;
const TICKS_PER_PERSIST = 10;

let wasmModulePromise: Promise<WasmModule> | null = null;
let pendingSession: SessionIntent | null = null;
let activeClient: RaidDefenseSimulationClient | null = null;
let flushListenersInstalled = false;

async function loadWasmModule() {
  if (!wasmModulePromise) {
    wasmModulePromise = import("./generated/raid-defense-wasm/raid_defense_wasm.js").then(
      async (module) => {
        await module.default();
        return module;
      },
    );
  }

  return wasmModulePromise;
}

function isObject(value: unknown): value is JsonObject {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function parseObject(json: string, label: string) {
  const value: unknown = JSON.parse(json);
  if (!isObject(value)) {
    throw new Error(`${label} must be a JSON object`);
  }
  return value;
}

function parseSnapshotValue(value: unknown): SnapshotView {
  if (!isObject(value)) {
    throw new Error("snapshot must be an object");
  }
  if (value.contract_version !== WASM_CONTRACT_VERSION) {
    throw new Error(`unsupported contract version: ${String(value.contract_version)}`);
  }
  if (!Array.isArray(value.entities)) {
    throw new Error("snapshot entities must be an array");
  }
  if (typeof value.checksum !== "string") {
    throw new Error("snapshot checksum must be a string");
  }
  if (typeof value.grid_width !== "number" || typeof value.grid_height !== "number") {
    throw new Error("snapshot grid dimensions must be numbers");
  }
  if (typeof value.wood !== "number" || typeof value.wood_capacity !== "number") {
    throw new Error("snapshot wood storage must be numeric");
  }
  if (typeof value.people !== "number" || typeof value.population_capacity !== "number") {
    throw new Error("snapshot population must be numeric");
  }
  if (typeof value.houses_unlocked !== "boolean") {
    throw new Error("snapshot house unlock state must be boolean");
  }
  if (
    typeof value.day_ticks_remaining !== "number" ||
    typeof value.raid_rally_ticks_remaining !== "number" ||
    typeof value.raid_rally_ticks !== "number" ||
    typeof value.is_rallying !== "boolean" ||
    typeof value.is_night !== "boolean" ||
    typeof value.automatic_raids !== "boolean" ||
    typeof value.pause_economy_during_raids !== "boolean"
  ) {
    throw new Error("snapshot day, rally, and raid state must be authoritative and typed");
  }
  if (
    typeof value.forest_tile_count !== "number" ||
    typeof value.forest_regrowth_amount !== "number" ||
    typeof value.forest_regrowth_interval_ticks !== "number" ||
    typeof value.storage_house_wood_capacity !== "number"
  ) {
    throw new Error("snapshot resource logistics rules must be numeric");
  }

  return value as SnapshotView;
}

function parseSnapshot(json: string) {
  return parseSnapshotValue(parseObject(json, "snapshot"));
}

export async function loadStandardScenarioWorld(): Promise<ScenarioWorldRules> {
  const module = await loadWasmModule();
  const engine = new module.RaidDefenseGame(0);
  try {
    const snapshot = parseSnapshot(engine.snapshot());
    return {
      starting_wood: snapshot.wood,
      forest_tile_count: snapshot.forest_tile_count,
      forest_tile_wood: snapshot.forest_tile_wood,
      forest_regrowth_amount: snapshot.forest_regrowth_amount,
      forest_regrowth_interval_ticks: snapshot.forest_regrowth_interval_ticks,
      sawmill_output: snapshot.sawmill_output,
      sawmill_interval_ticks: snapshot.sawmill_interval_ticks,
      sawmill_local_wood_capacity: snapshot.sawmill_local_wood_capacity,
      day_length_ticks: snapshot.day_ticks_remaining,
      raid_rally_ticks: snapshot.raid_rally_ticks,
      automatic_raids: snapshot.automatic_raids,
      pause_economy_during_raids: snapshot.pause_economy_during_raids,
    };
  } finally {
    engine.free();
  }
}

function parseResponse(json: string): DispatchResponse {
  const value = parseObject(json, "dispatch response");
  if (value.contract_version !== WASM_CONTRACT_VERSION || typeof value.ok !== "boolean") {
    throw new Error("dispatch response has an invalid contract envelope");
  }

  const snapshot = parseSnapshotValue(value.snapshot);
  return { ...(value as Omit<DispatchResponse, "snapshot">), snapshot };
}

function seedFromUrl(fallback: number) {
  if (typeof window === "undefined") return fallback;
  const value = Number(new URLSearchParams(window.location.search).get("seed"));
  return Number.isInteger(value) && value >= 0 && value <= 0xffff_ffff ? value : fallback;
}

function cloneScenario(scenario: ScenarioOptions): ScenarioOptions {
  return JSON.parse(JSON.stringify(scenario)) as ScenarioOptions;
}

function cloneReplay(replay: ReplayRecord): ReplayRecord {
  return JSON.parse(JSON.stringify(replay)) as ReplayRecord;
}

function createScenarioEngine(module: WasmModule, seed: number, scenario: ScenarioOptions) {
  return module.create_game(seed, JSON.stringify(scenario));
}

function installFlushListeners() {
  if (flushListenersInstalled || typeof window === "undefined") return;
  flushListenersInstalled = true;
  window.addEventListener("pagehide", () => activeClient?.flushSave());
  document.addEventListener("visibilitychange", () => {
    if (document.visibilityState === "hidden") activeClient?.flushSave();
  });
}

export function prepareNewGame(
  seed: number,
  scenario: ScenarioOptions = createStandardScenarioOptions(),
) {
  clearSavedGame();
  pendingSession = { kind: "new", seed: seed >>> 0, scenario: cloneScenario(scenario) };
}

export function prepareResume() {
  pendingSession = { kind: "resume" };
}

export function clearSavedGame() {
  activeClient = null;
  pendingSession = null;
  clearStoredReplay();
}

export function flushActiveSession() {
  activeClient?.flushSave();
}

export function readSavedGameSummary(): SavedGameSummary | null {
  return readReplaySummary();
}

export class RaidDefenseSimulationClient {
  private readonly engine: InstanceType<WasmModule["RaidDefenseGame"]>;
  private readonly replay: ReplayRecord;
  private lastSnapshot: SnapshotView;
  private dirtyTicks = 0;

  private constructor(
    engine: InstanceType<WasmModule["RaidDefenseGame"]>,
    replay: ReplayRecord,
    lastSnapshot: SnapshotView,
  ) {
    this.engine = engine;
    this.replay = replay;
    this.lastSnapshot = lastSnapshot;
  }

  static async create(fallbackSeed: number) {
    const module = await loadWasmModule();
    installFlushListeners();

    const intent = pendingSession;
    pendingSession = null;
    const params = typeof window === "undefined" ? null : new URLSearchParams(window.location.search);
    const directNew = params?.get("new") === "1";
    const shouldResume =
      intent?.kind === "resume" || (!intent && !directNew && params?.get("screen") === "game");

    if (shouldResume) {
      const stored = readStoredReplay();
      if (stored) {
        const client = await RaidDefenseSimulationClient.fromStoredReplay(module, stored);
        activeClient = client;
        return client;
      }
      if (intent?.kind === "resume") {
        throw new Error("No compatible saved game is available to resume.");
      }
    }

    const seed = intent?.kind === "new" ? intent.seed : seedFromUrl(fallbackSeed);
    const scenario = intent?.kind === "new" ? intent.scenario : createStandardScenarioOptions();
    const engine = createScenarioEngine(module, seed, scenario);
    const snapshot = parseSnapshot(engine.snapshot());
    const replay = createReplayRecord(seed, scenario);
    const client = new RaidDefenseSimulationClient(engine, replay, snapshot);
    activeClient = client;
    client.flushSave();
    return client;
  }

  private static async fromStoredReplay(module: WasmModule, loaded: LoadedReplay) {
    const replay = loaded.replay;
    const engine = createScenarioEngine(module, replay.seed, replay.scenario);
    let currentTick = 0;
    let replayedOperations = 0;

    const replayCommand = async (command: RaidDefenseCommand) => {
      const response = parseResponse(engine.dispatch(JSON.stringify(command)));
      if (!response.ok) {
        throw new Error(`Saved replay rejected command: ${response.error?.code ?? "unknown"}`);
      }
      currentTick = response.snapshot.tick;
      replayedOperations += 1;
      if (replayedOperations % 250 === 0) {
        await new Promise<void>((resolve) => window.setTimeout(resolve, 0));
      }
    };

    const advanceToTick = async (targetTick: number) => {
      while (currentTick < targetTick) {
        await replayCommand({ type: "advance_tick" });
      }
      if (currentTick !== targetTick) {
        throw new Error("Saved replay action tick is behind the reconstructed simulation.");
      }
    };

    for (const recorded of replay.actions) {
      await advanceToTick(recorded.tick);
      await replayCommand(recorded.command);
      if (currentTick !== recorded.tick) {
        throw new Error("Saved replay player action unexpectedly advanced simulation time.");
      }
    }
    await advanceToTick(replay.recorded_through_tick);

    const snapshot = parseSnapshot(engine.snapshot());
    if (loaded.legacy_checksum && snapshot.checksum !== loaded.legacy_checksum) {
      throw new Error("Legacy saved-game checksum does not match the reconstructed action log.");
    }

    const client = new RaidDefenseSimulationClient(engine, cloneReplay(replay), snapshot);
    client.flushSave();
    return client;
  }

  snapshot() {
    return parseSnapshot(this.engine.snapshot());
  }

  dispatch(command: RaidDefenseCommand) {
    const response = parseResponse(this.engine.dispatch(JSON.stringify(command)));
    if (response.ok) {
      this.lastSnapshot = response.snapshot;
      this.recordAcceptedCommand(command, response.snapshot.tick);
    }
    return response;
  }

  checksum() {
    return this.engine.checksum();
  }

  flushSave() {
    writeStoredReplay(this.replay, summaryFromSnapshot(this.replay, this.lastSnapshot));
    this.dirtyTicks = 0;
  }

  private recordAcceptedCommand(command: RaidDefenseCommand, tick: number) {
    this.replay.recorded_through_tick = tick;
    if (command.type === "advance_tick") {
      this.dirtyTicks += 1;
    } else {
      this.replay.actions.push({
        tick,
        sequence: this.replay.actions.length,
        player_id: LOCAL_PLAYER_ID,
        command,
      });
    }

    if (command.type !== "advance_tick" || this.dirtyTicks >= TICKS_PER_PERSIST) {
      this.flushSave();
    }
  }
}
