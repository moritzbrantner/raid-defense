declare module "./generated/raid-defense-wasm/raid_defense_wasm.js" {
  export default function init(input?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module): Promise<unknown>;

  export class WasmRaidDefense {
    constructor();
    static newWithSeed(seed: bigint): WasmRaidDefense;
    view_json(): string;
    apply_json(commandJson: string): string;
    advance(deltaSeconds: bigint): string;
    save_snapshot_json(): string;
    load_snapshot_json(snapshotJson: string): string;
    grantResource(resource: string, amount: bigint): string;
  }
}
