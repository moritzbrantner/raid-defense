import type {
  RuntimeRaidDefenseCommandResponse,
  RuntimeRaidDefenseView,
} from "./simulationTypes";

type WasmModule = typeof import("./generated/raid-defense-wasm/raid_defense_wasm.js");

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

function parseView(viewJson: string) {
  return JSON.parse(viewJson) as RuntimeRaidDefenseView;
}

function parseResponse(responseJson: string) {
  return JSON.parse(responseJson) as RuntimeRaidDefenseCommandResponse;
}

export class RaidDefenseSimulationClient {
  readonly seed: bigint;
  readonly engine: InstanceType<WasmModule["WasmRaidDefense"]>;

  private constructor(engine: InstanceType<WasmModule["WasmRaidDefense"]>, seed: bigint) {
    this.engine = engine;
    this.seed = seed;
  }

  static async create(seed: bigint | number | string) {
    const module = await loadWasmModule();
    const normalizedSeed = BigInt(seed);
    return new RaidDefenseSimulationClient(
      module.WasmRaidDefense.newWithSeed(normalizedSeed),
      normalizedSeed,
    );
  }

  view() {
    return parseView(this.engine.view_json());
  }

  apply(command: unknown) {
    return parseResponse(this.engine.apply_json(JSON.stringify(command)));
  }

  advance(deltaSeconds: bigint | number) {
    return parseResponse(this.engine.advance(BigInt(deltaSeconds)));
  }

  grantResource(resource: string, amount: bigint | number) {
    return parseResponse(this.engine.grantResource(resource, BigInt(amount)));
  }

  saveSnapshot() {
    return this.engine.save_snapshot_json();
  }

  loadSnapshot(snapshotJson: string) {
    return parseResponse(this.engine.load_snapshot_json(snapshotJson));
  }
}
