export type CellView = {
  x: number;
  z: number;
};

export type TowerArchetype = "arrow" | "cannon";
export type EntityKind = "town" | "tower" | "raider" | "projectile";

export type EntityView = {
  id: number;
  kind: EntityKind;
  x_milli: number;
  z_milli: number;
  cell: CellView;
  health: number;
  max_health: number;
  attack_damage: number;
  attack_range_milli: number;
  tower_archetype: TowerArchetype | null;
  tower_level: number;
  upgrade_cost: number | null;
  projectile_target: number | null;
};

export type SnapshotView = {
  contract_version: 3;
  seed: string;
  tick: number;
  gold: number;
  wave: number;
  town_health: number;
  town_max_health: number;
  grid_width: number;
  grid_height: number;
  arrow_tower_cost: number;
  cannon_tower_cost: number;
  max_tower_level: number;
  checksum: string;
  entities: EntityView[];
};

export type RaidDefenseCommand =
  | { type: "place_tower"; x: number; z: number; archetype: TowerArchetype }
  | { type: "upgrade_tower"; x: number; z: number }
  | { type: "start_wave" }
  | { type: "advance_tick" };

export type RaidDefenseEvent =
  | {
      type: "tower_built";
      entity: number;
      cell: CellView;
      archetype: TowerArchetype;
      level: number;
    }
  | {
      type: "tower_upgraded";
      entity: number;
      archetype: TowerArchetype;
      level: number;
      cost: number;
    }
  | { type: "wave_started"; wave: number; raiders: number }
  | {
      type: "tick_advanced";
      tick: number;
      shots: number;
      impacts: number;
      kills: number;
      town_damage: number;
    };

export type DispatchResponse = {
  contract_version: 3;
  ok: boolean;
  event: RaidDefenseEvent | null;
  error: { code: string } | null;
  snapshot: SnapshotView;
};
