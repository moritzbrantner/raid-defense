export type RaiderArchetype = "basic" | "advanced";

export type RaiderRules = {
  health: number;
  damage: number;
  speed_milli: number;
  wood_steal: number;
};

export type RaiderCatalog = {
  basic: RaiderRules;
  advanced: RaiderRules;
};

export type WaveGroup = {
  archetype: RaiderArchetype;
  count: number;
};

export type ScenarioWave = {
  groups: WaveGroup[];
};

export type TowerLevelRules = {
  damage: number;
  range_milli: number;
  cooldown_ticks: number;
  projectile_speed_milli: number;
  upgrade_cost: number | null;
};

export type TowerArchetypeRules = {
  build_cost: number;
  levels: [TowerLevelRules, TowerLevelRules, TowerLevelRules];
};

export type ScenarioTowerRules = {
  max_level: number;
  max_health: number;
  arrow: TowerArchetypeRules;
  cannon: TowerArchetypeRules;
};

export type ScenarioWorldRules = {
  starting_wood: number;
  forest_tile_count: number;
  forest_tile_wood: number;
  forest_regrowth_amount: number;
  forest_regrowth_interval_ticks: number;
  sawmill_output: number;
  sawmill_interval_ticks: number;
  sawmill_local_wood_capacity: number;
  day_length_ticks: number;
  raid_rally_ticks: number;
  automatic_raids: boolean;
  pause_economy_during_raids: boolean;
};

export type ScenarioOptions = {
  /** Compatibility fields for callers that still use named world presets. Explicit world values win when present. */
  starting_supplies: "lean" | "standard" | "rich";
  forest_density: "sparse" | "standard" | "dense";
  forest_regrowth: "slow" | "standard" | "fast";
  sawmill_throughput: "slow" | "standard" | "fast";
  /** Compatibility fields for non-editor callers. Explicit waves are authoritative. */
  raid_size: "standard";
  raider_strength: "standard";
  day_length: "short" | "standard" | "long";
  raid_timing: "standard" | "manual";
  raid_economy: "standard" | "continuous";
  world?: ScenarioWorldRules;
  raiders: RaiderCatalog;
  waves: ScenarioWave[];
  towers: ScenarioTowerRules;
};

const defaultTowerLevels = {
  arrow: [
    { damage: 8, range_milli: 3_200, cooldown_ticks: 3, projectile_speed_milli: 900, upgrade_cost: 20 },
    { damage: 12, range_milli: 3_500, cooldown_ticks: 3, projectile_speed_milli: 1_000, upgrade_cost: 30 },
    { damage: 17, range_milli: 3_800, cooldown_ticks: 2, projectile_speed_milli: 1_100, upgrade_cost: null },
  ],
  cannon: [
    { damage: 18, range_milli: 4_200, cooldown_ticks: 7, projectile_speed_milli: 500, upgrade_cost: 30 },
    { damage: 27, range_milli: 4_500, cooldown_ticks: 6, projectile_speed_milli: 550, upgrade_cost: 45 },
    { damage: 40, range_milli: 4_800, cooldown_ticks: 5, projectile_speed_milli: 600, upgrade_cost: null },
  ],
} as const;

export const STANDARD_SCENARIO_WORLD: ScenarioWorldRules = {
  starting_wood: 120,
  forest_tile_count: 18,
  forest_tile_wood: 80,
  forest_regrowth_amount: 1,
  forest_regrowth_interval_ticks: 20,
  sawmill_output: 4,
  sawmill_interval_ticks: 10,
  sawmill_local_wood_capacity: 24,
  day_length_ticks: 600,
  raid_rally_ticks: 50,
  automatic_raids: true,
  pause_economy_during_raids: true,
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
  world: { ...STANDARD_SCENARIO_WORLD },
  raiders: {
    basic: { health: 30, damage: 10, speed_milli: 250, wood_steal: 15 },
    advanced: { health: 60, damage: 18, speed_milli: 220, wood_steal: 25 },
  },
  waves: [
    { groups: [{ archetype: "basic", count: 2 }] },
    { groups: [{ archetype: "basic", count: 3 }] },
    { groups: [{ archetype: "basic", count: 4 }] },
    { groups: [{ archetype: "basic", count: 4 }, { archetype: "advanced", count: 1 }] },
    { groups: [{ archetype: "basic", count: 2 }, { archetype: "advanced", count: 1 }] },
    { groups: [{ archetype: "basic", count: 2 }, { archetype: "advanced", count: 2 }] },
    { groups: [{ archetype: "basic", count: 4 }, { archetype: "advanced", count: 2 }] },
    { groups: [{ archetype: "basic", count: 4 }, { archetype: "advanced", count: 3 }] },
    { groups: [{ archetype: "basic", count: 6 }, { archetype: "advanced", count: 3 }] },
    { groups: [{ archetype: "basic", count: 6 }, { archetype: "advanced", count: 4 }] },
    { groups: [{ archetype: "basic", count: 8 }, { archetype: "advanced", count: 4 }] },
    { groups: [{ archetype: "basic", count: 8 }, { archetype: "advanced", count: 5 }] },
  ],
  towers: {
    max_level: 3,
    max_health: 100,
    arrow: {
      build_cost: 25,
      levels: defaultTowerLevels.arrow.map((level) => ({ ...level })) as ScenarioTowerRules["arrow"]["levels"],
    },
    cannon: {
      build_cost: 45,
      levels: defaultTowerLevels.cannon.map((level) => ({ ...level })) as ScenarioTowerRules["cannon"]["levels"],
    },
  },
};

export function createStandardScenarioOptions(): ScenarioOptions {
  return {
    ...STANDARD_SCENARIO_OPTIONS,
    world: { ...STANDARD_SCENARIO_WORLD },
    raiders: {
      basic: { ...STANDARD_SCENARIO_OPTIONS.raiders.basic },
      advanced: { ...STANDARD_SCENARIO_OPTIONS.raiders.advanced },
    },
    waves: STANDARD_SCENARIO_OPTIONS.waves.map((wave) => ({
      groups: wave.groups.map((group) => ({ ...group })),
    })),
    towers: {
      ...STANDARD_SCENARIO_OPTIONS.towers,
      arrow: {
        ...STANDARD_SCENARIO_OPTIONS.towers.arrow,
        levels: STANDARD_SCENARIO_OPTIONS.towers.arrow.levels.map((level) => ({ ...level })) as ScenarioTowerRules["arrow"]["levels"],
      },
      cannon: {
        ...STANDARD_SCENARIO_OPTIONS.towers.cannon,
        levels: STANDARD_SCENARIO_OPTIONS.towers.cannon.levels.map((level) => ({ ...level })) as ScenarioTowerRules["cannon"]["levels"],
      },
    },
  };
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isOneOf<T extends string>(value: unknown, values: readonly T[]): value is T {
  return typeof value === "string" && values.includes(value as T);
}

function isInteger(value: unknown, min: number, max: number): value is number {
  return typeof value === "number" && Number.isInteger(value) && value >= min && value <= max;
}

function isRaiderRules(value: unknown): value is RaiderRules {
  if (!isObject(value)) return false;
  return (
    isInteger(value.health, 1, 0xffff) &&
    isInteger(value.damage, 1, 0xffff) &&
    isInteger(value.speed_milli, 1, 0xffff) &&
    isInteger(value.wood_steal, 1, 0xffff_ffff)
  );
}

function isWave(value: unknown): value is ScenarioWave {
  if (!isObject(value) || !Array.isArray(value.groups) || value.groups.length < 1 || value.groups.length > 8) {
    return false;
  }
  return value.groups.every(
    (group) =>
      isObject(group) &&
      isOneOf(group.archetype, ["basic", "advanced"] as const) &&
      isInteger(group.count, 1, 0xffff),
  );
}

function isTowerLevel(value: unknown): value is TowerLevelRules {
  if (!isObject(value)) return false;
  return (
    isInteger(value.damage, 0, 0xffff) &&
    isInteger(value.range_milli, 0, 0x7fff_ffff) &&
    isInteger(value.cooldown_ticks, 0, 0xff) &&
    isInteger(value.projectile_speed_milli, 0, 0xffff) &&
    (value.upgrade_cost === null || isInteger(value.upgrade_cost, 0, 0xffff_ffff))
  );
}

function isTowerArchetype(value: unknown): value is TowerArchetypeRules {
  return (
    isObject(value) &&
    isInteger(value.build_cost, 0, 0xffff_ffff) &&
    Array.isArray(value.levels) &&
    value.levels.length === 3 &&
    value.levels.every(isTowerLevel)
  );
}

function isTowerRules(value: unknown): value is ScenarioTowerRules {
  return (
    isObject(value) &&
    isInteger(value.max_level, 1, 3) &&
    isInteger(value.max_health, 1, 0xffff) &&
    isTowerArchetype(value.arrow) &&
    isTowerArchetype(value.cannon)
  );
}

function isWorldRules(value: unknown): value is ScenarioWorldRules {
  if (!isObject(value)) return false;
  return (
    isInteger(value.starting_wood, 0, 500) &&
    isInteger(value.forest_tile_count, 1, 0xffff) &&
    isInteger(value.forest_tile_wood, 1, 0xffff_ffff) &&
    isInteger(value.forest_regrowth_amount, 1, 0xffff) &&
    isInteger(value.forest_regrowth_interval_ticks, 1, 0xffff) &&
    isInteger(value.sawmill_output, 1, 0xffff) &&
    isInteger(value.sawmill_interval_ticks, 1, 0xffff) &&
    isInteger(value.sawmill_local_wood_capacity, 1, 0xffff_ffff) &&
    isInteger(value.day_length_ticks, 1, 0xffff) &&
    isInteger(value.raid_rally_ticks, 1, 0xffff) &&
    typeof value.automatic_raids === "boolean" &&
    typeof value.pause_economy_during_raids === "boolean"
  );
}

export function isScenarioOptions(value: unknown): value is ScenarioOptions {
  if (!isObject(value)) return false;
  if (!isObject(value.raiders)) return false;
  if (!Array.isArray(value.waves) || value.waves.length < 1 || value.waves.length > 64) return false;
  if (value.world !== undefined && !isWorldRules(value.world)) return false;
  return (
    isOneOf(value.starting_supplies, ["lean", "standard", "rich"] as const) &&
    isOneOf(value.forest_density, ["sparse", "standard", "dense"] as const) &&
    isOneOf(value.forest_regrowth, ["slow", "standard", "fast"] as const) &&
    isOneOf(value.sawmill_throughput, ["slow", "standard", "fast"] as const) &&
    value.raid_size === "standard" &&
    value.raider_strength === "standard" &&
    isOneOf(value.day_length, ["short", "standard", "long"] as const) &&
    isOneOf(value.raid_timing, ["standard", "manual"] as const) &&
    isOneOf(value.raid_economy, ["standard", "continuous"] as const) &&
    isRaiderRules(value.raiders.basic) &&
    isRaiderRules(value.raiders.advanced) &&
    value.waves.every(isWave) &&
    isTowerRules(value.towers)
  );
}
