import type {
  DispatchResponse,
  RaidDefenseCommand,
  SnapshotView,
} from "./simulationTypes";

type WasmModule = typeof import("./generated/raid-defense-wasm/raid_defense_wasm.js");

type JsonObject = Record<string, unknown>;

let wasmModulePromise: Promise<WasmModule> | null = null;

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
  if (value.contract_version !== 4) {
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

  return value as SnapshotView;
}

function parseSnapshot(json: string) {
  return parseSnapshotValue(parseObject(json, "snapshot"));
}

function parseResponse(json: string): DispatchResponse {
  const value = parseObject(json, "dispatch response");
  if (value.contract_version !== 4 || typeof value.ok !== "boolean") {
    throw new Error("dispatch response has an invalid contract envelope");
  }

  const snapshot = parseSnapshotValue(value.snapshot);
  return { ...(value as Omit<DispatchResponse, "snapshot">), snapshot };
}

export class RaidDefenseSimulationClient {
  private readonly engine: InstanceType<WasmModule["RaidDefenseGame"]>;

  private constructor(engine: InstanceType<WasmModule["RaidDefenseGame"]>) {
    this.engine = engine;
  }

  static async create(seed: number) {
    const module = await loadWasmModule();
    return new RaidDefenseSimulationClient(new module.RaidDefenseGame(seed));
  }

  snapshot() {
    return parseSnapshot(this.engine.snapshot());
  }

  dispatch(command: RaidDefenseCommand) {
    return parseResponse(this.engine.dispatch(JSON.stringify(command)));
  }

  checksum() {
    return this.engine.checksum();
  }
}
