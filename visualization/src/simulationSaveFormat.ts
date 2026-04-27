import type { SimulationSaveState } from "./simulationTypes";

export const simulationFileFormat = "raid-defense-simulation";
export const simulationFileVersion = 1;

export type SimulationJsonFile = {
  format: typeof simulationFileFormat;
  version: typeof simulationFileVersion;
  mode: "simulation";
  name: string;
  seed: string;
  exported_at: string;
  now_seconds: number;
  snapshot_json: string;
};

type LegacySaveStateLike = {
  name?: string;
  seed: string;
  now_seconds: number;
  snapshot_json: string;
  created_at?: string;
  updated_at?: string;
};

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isSaveStateLike(value: unknown): value is LegacySaveStateLike {
  return (
    isRecord(value) &&
    typeof value.seed === "string" &&
    typeof value.now_seconds === "number" &&
    typeof value.snapshot_json === "string"
  );
}

export function simulationFileFromSaveState(
  saveState: Pick<SimulationSaveState, "name" | "seed" | "now_seconds" | "snapshot_json">,
) {
  return {
    format: simulationFileFormat,
    version: simulationFileVersion,
    mode: "simulation",
    name: saveState.name,
    seed: saveState.seed,
    exported_at: new Date().toISOString(),
    now_seconds: saveState.now_seconds,
    snapshot_json: saveState.snapshot_json,
  };
}

export function serializeSimulationFile(
  saveState: Pick<SimulationSaveState, "name" | "seed" | "now_seconds" | "snapshot_json">,
) {
  const file = simulationFileFromSaveState(saveState);

  return JSON.stringify(file, null, 2);
}

export function parseSimulationFileText(text: string): SimulationJsonFile {
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch {
    throw new Error("Simulation file is not valid JSON.");
  }

  if (
    isRecord(parsed) &&
    parsed.format === simulationFileFormat &&
    parsed.version === simulationFileVersion &&
    parsed.mode === "simulation" &&
    typeof parsed.name === "string" &&
    typeof parsed.seed === "string" &&
    typeof parsed.exported_at === "string" &&
    typeof parsed.now_seconds === "number" &&
    typeof parsed.snapshot_json === "string"
  ) {
    return parsed as SimulationJsonFile;
  }

  if (isSaveStateLike(parsed)) {
    return {
      format: simulationFileFormat,
      version: simulationFileVersion,
      mode: "simulation",
      name: parsed.name ?? "Imported Simulation",
      seed: parsed.seed,
      exported_at: parsed.created_at ?? parsed.updated_at ?? new Date().toISOString(),
      now_seconds: parsed.now_seconds,
      snapshot_json: parsed.snapshot_json,
    };
  }

  throw new Error("Simulation file does not match the expected Raid Defense save format.");
}

export function toImportedSimulationSaveState(
  file: SimulationJsonFile,
  createId: () => string,
): SimulationSaveState {
  return {
    id: createId(),
    name: file.name,
    seed: file.seed,
    created_at: file.exported_at,
    now_seconds: file.now_seconds,
    snapshot_json: file.snapshot_json,
  };
}

export function sanitizeSimulationFileName(name: string) {
  const normalized = name.trim().toLowerCase().replace(/[^a-z0-9]+/g, "-");
  return normalized.replace(/^-+|-+$/g, "") || "simulation";
}
