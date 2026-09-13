import type {
  DispatchResponse,
  RaidDefenseCommand,
  SnapshotView,
} from "./simulationTypes";

type WasmModule = typeof import("./generated/raid-defense-wasm/raid_defense_wasm.js");
type JsonObject = Record<string, unknown>;
type SessionIntent = { kind: "new"; seed: number } | { kind: "resume" };
type ReplayEntry =
  | { kind: "advance_ticks"; count: number }
  | { kind: "command"; command: RaidDefenseCommand };

type SavedGameV1 = {
  version: 1;
  contract_version: 8;
  seed: number;
  entries: ReplayEntry[];
  checksum: string;
  tick: number;
  wave: number;
  completed_waves: number;
  saved_at: number;
};

export type SavedGameSummary = Pick<
  SavedGameV1,
  "seed" | "checksum" | "tick" | "wave" | "completed_waves" | "saved_at"
>;

const SAVE_KEY = "raid-defense.save.v1";
const CONTRACT_VERSION = 8;
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
  if (value.contract_version !== CONTRACT_VERSION) {
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
  if (typeof value.day_ticks_remaining !== "number" || typeof value.is_night !== "boolean") {
    throw new Error("snapshot day/night state must be authoritative and typed");
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

function parseResponse(json: string): DispatchResponse {
  const value = parseObject(json, "dispatch response");
  if (value.contract_version !== CONTRACT_VERSION || typeof value.ok !== "boolean") {
    throw new Error("dispatch response has an invalid contract envelope");
  }

  const snapshot = parseSnapshotValue(value.snapshot);
  return { ...(value as Omit<DispatchResponse, "snapshot">), snapshot };
}

function storageAvailable() {
  return typeof window !== "undefined" && typeof window.localStorage !== "undefined";
}

function isReplayEntry(value: unknown): value is ReplayEntry {
  if (!isObject(value) || typeof value.kind !== "string") return false;
  if (value.kind === "advance_ticks") {
    return typeof value.count === "number" && Number.isInteger(value.count) && value.count > 0;
  }
  return value.kind === "command" && isObject(value.command) && typeof value.command.type === "string";
}

function parseSavedGame(raw: string): SavedGameV1 | null {
  try {
    const value: unknown = JSON.parse(raw);
    if (!isObject(value)) return null;
    if (value.version !== 1 || value.contract_version !== CONTRACT_VERSION) return null;
    if (
      typeof value.seed !== "number" ||
      !Number.isInteger(value.seed) ||
      value.seed < 0 ||
      value.seed > 0xffff_ffff
    ) {
      return null;
    }
    if (!Array.isArray(value.entries) || !value.entries.every(isReplayEntry)) return null;
    if (typeof value.checksum !== "string") return null;
    if (
      typeof value.tick !== "number" ||
      typeof value.wave !== "number" ||
      !Number.isInteger(value.tick) ||
      !Number.isInteger(value.wave)
    ) {
      return null;
    }
    if (
      typeof value.completed_waves !== "number" ||
      typeof value.saved_at !== "number" ||
      !Number.isInteger(value.completed_waves) ||
      !Number.isFinite(value.saved_at)
    ) {
      return null;
    }
    return value as SavedGameV1;
  } catch {
    return null;
  }
}

function readSavedGame() {
  if (!storageAvailable()) return null;
  const raw = window.localStorage.getItem(SAVE_KEY);
  return raw ? parseSavedGame(raw) : null;
}

function writeSavedGame(save: SavedGameV1) {
  if (!storageAvailable()) return;
  try {
    window.localStorage.setItem(SAVE_KEY, JSON.stringify(save));
  } catch (error) {
    console.warn("Unable to persist Raid Defense save", error);
  }
}

function summaryFromSave(save: SavedGameV1): SavedGameSummary {
  return {
    seed: save.seed,
    checksum: save.checksum,
    tick: save.tick,
    wave: save.wave,
    completed_waves: save.completed_waves,
    saved_at: save.saved_at,
  };
}

function seedFromUrl(fallback: number) {
  if (typeof window === "undefined") return fallback;
  const value = Number(new URLSearchParams(window.location.search).get("seed"));
  return Number.isInteger(value) && value >= 0 && value <= 0xffff_ffff ? value : fallback;
}

function installFlushListeners() {
  if (flushListenersInstalled || typeof window === "undefined") return;
  flushListenersInstalled = true;
  window.addEventListener("pagehide", () => activeClient?.flushSave());
  document.addEventListener("visibilitychange", () => {
    if (document.visibilityState === "hidden") activeClient?.flushSave();
  });
}

export function prepareNewGame(seed: number) {
  clearSavedGame();
  pendingSession = { kind: "new", seed: seed >>> 0 };
}

export function prepareResume() {
  pendingSession = { kind: "resume" };
}

export function clearSavedGame() {
  activeClient = null;
  pendingSession = null;
  if (storageAvailable()) window.localStorage.removeItem(SAVE_KEY);
}

export function flushActiveSession() {
  activeClient?.flushSave();
}

export function readSavedGameSummary(): SavedGameSummary | null {
  const save = readSavedGame();
  return save ? summaryFromSave(save) : null;
}

export class RaidDefenseSimulationClient {
  private readonly engine: InstanceType<WasmModule["RaidDefenseGame"]>;
  private readonly save: SavedGameV1;
  private dirtyTicks = 0;

  private constructor(
    engine: InstanceType<WasmModule["RaidDefenseGame"]>,
    save: SavedGameV1,
  ) {
    this.engine = engine;
    this.save = save;
  }

  static async create(fallbackSeed: number) {
    const module = await loadWasmModule();
    installFlushListeners();

    const intent = pendingSession;
    pendingSession = null;
    const params = typeof window === "undefined" ? null : new URLSearchParams(window.location.search);
    const directNew = params?.get("new") === "1";
    const shouldResume = intent?.kind === "resume" || (!intent && !directNew && params?.get("screen") === "game");

    if (shouldResume) {
      const saved = readSavedGame();
      if (saved) {
        const client = await RaidDefenseSimulationClient.fromSavedGame(module, saved);
        activeClient = client;
        return client;
      }
      if (intent?.kind === "resume") {
        throw new Error("No compatible saved game is available to resume.");
      }
    }

    const seed = intent?.kind === "new" ? intent.seed : seedFromUrl(fallbackSeed);
    const engine = new module.RaidDefenseGame(seed);
    const snapshot = parseSnapshot(engine.snapshot());
    const save: SavedGameV1 = {
      version: 1,
      contract_version: CONTRACT_VERSION,
      seed,
      entries: [],
      checksum: snapshot.checksum,
      tick: snapshot.tick,
      wave: snapshot.wave,
      completed_waves: snapshot.completed_waves,
      saved_at: Date.now(),
    };
    const client = new RaidDefenseSimulationClient(engine, save);
    activeClient = client;
    client.flushSave();
    return client;
  }

  private static async fromSavedGame(module: WasmModule, saved: SavedGameV1) {
    const engine = new module.RaidDefenseGame(saved.seed);
    let replayed = 0;

    const replayCommand = async (command: RaidDefenseCommand) => {
      const response = parseResponse(engine.dispatch(JSON.stringify(command)));
      if (!response.ok) {
        throw new Error(`Saved replay rejected command: ${response.error?.code ?? "unknown"}`);
      }
      replayed += 1;
      if (replayed % 250 === 0) {
        await new Promise<void>((resolve) => window.setTimeout(resolve, 0));
      }
    };

    for (const entry of saved.entries) {
      if (entry.kind === "advance_ticks") {
        for (let tick = 0; tick < entry.count; tick += 1) {
          await replayCommand({ type: "advance_tick" });
        }
      } else {
        await replayCommand(entry.command);
      }
    }

    const checksum = engine.checksum();
    if (checksum !== saved.checksum) {
      throw new Error("Saved game replay checksum does not match the recorded authoritative state.");
    }

    const cloned = JSON.parse(JSON.stringify(saved)) as SavedGameV1;
    return new RaidDefenseSimulationClient(engine, cloned);
  }

  snapshot() {
    return parseSnapshot(this.engine.snapshot());
  }

  dispatch(command: RaidDefenseCommand) {
    const response = parseResponse(this.engine.dispatch(JSON.stringify(command)));
    if (response.ok) this.recordAcceptedCommand(command, response.snapshot);
    return response;
  }

  checksum() {
    return this.engine.checksum();
  }

  flushSave() {
    this.save.saved_at = Date.now();
    writeSavedGame(this.save);
    this.dirtyTicks = 0;
  }

  private recordAcceptedCommand(command: RaidDefenseCommand, snapshot: SnapshotView) {
    if (command.type === "advance_tick") {
      const last = this.save.entries.at(-1);
      if (last?.kind === "advance_ticks") {
        last.count += 1;
      } else {
        this.save.entries.push({ kind: "advance_ticks", count: 1 });
      }
      this.dirtyTicks += 1;
    } else {
      this.save.entries.push({ kind: "command", command });
    }

    this.save.checksum = snapshot.checksum;
    this.save.tick = snapshot.tick;
    this.save.wave = snapshot.wave;
    this.save.completed_waves = snapshot.completed_waves;

    if (command.type !== "advance_tick" || this.dirtyTicks >= TICKS_PER_PERSIST) {
      this.flushSave();
    }
  }
}
