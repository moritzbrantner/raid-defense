export type ScenarioOptions = {
  starting_supplies: "lean" | "standard" | "rich";
  forest_density: "sparse" | "standard" | "dense";
  forest_regrowth: "slow" | "standard" | "fast";
  sawmill_throughput: "slow" | "standard" | "fast";
  raid_size: "small" | "standard" | "large";
  raider_strength: "gentle" | "standard" | "harsh";
  day_length: "short" | "standard" | "long";
  raid_timing: "standard" | "manual";
  raid_economy: "standard" | "continuous";
};

export const STANDARD_SCENARIO_OPTIONS: ScenarioOptions = {
  starting_supplies: "standard",
  forest_density: "standard",
  forest_regrowth: "standard",
  sawmill_throughput: "standard",
  raid_size: "standard",
  raider_strength: "standard",
  day_length: "standard",
  raid_timing: "standard",
  raid_economy: "standard",
};

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isOneOf<T extends string>(value: unknown, values: readonly T[]): value is T {
  return typeof value === "string" && values.includes(value as T);
}

export function isScenarioOptions(value: unknown): value is ScenarioOptions {
  if (!isObject(value)) return false;
  return (
    isOneOf(value.starting_supplies, ["lean", "standard", "rich"] as const) &&
    isOneOf(value.forest_density, ["sparse", "standard", "dense"] as const) &&
    isOneOf(value.forest_regrowth, ["slow", "standard", "fast"] as const) &&
    isOneOf(value.sawmill_throughput, ["slow", "standard", "fast"] as const) &&
    isOneOf(value.raid_size, ["small", "standard", "large"] as const) &&
    isOneOf(value.raider_strength, ["gentle", "standard", "harsh"] as const) &&
    isOneOf(value.day_length, ["short", "standard", "long"] as const) &&
    isOneOf(value.raid_timing, ["standard", "manual"] as const) &&
    isOneOf(value.raid_economy, ["standard", "continuous"] as const)
  );
}
