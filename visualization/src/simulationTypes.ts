export type CellView = {
  x: number;
  z: number;
};

export type TowerArchetype = "arrow" | "cannon";
export type ResourceKind = "wood";
export type PersonState = "idle_at_town_hall" | "to_sawmill" | "to_town_hall";
export type EntityKind =
  | "town_hall"
  | "tower"
  | "sawmill"
  | "house"
  | "person"
  | "raider"
  | "projectile";

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
  stored_wood: number;
  wood_capacity: number;
  production_resource: ResourceKind | null;
  production_amount: number;
  production_interval_ticks: number;
  production_progress_ticks: number;
  housing_capacity: number;
  person_state: PersonState | null;
  person_target_sawmill: number | null;
  cargo_wood: number;
  cargo_capacity: number;
};

export type SnapshotView = {
  contract_version: 6;
  seed: string;
  tick: number;
  wood: number;
  wood_capacity: number;
  wave: number;
  completed_waves: number;
  day_ticks_remaining: number;
  is_night: boolean;
  people: number;
  population_capacity: number;
  houses_unlocked: boolean;
  town_health: number;
  town_max_health: number;
  grid_width: number;
  grid_height: number;
  sawmill_cost: number;
  sawmill_output: number;
  sawmill_interval_ticks: number;
  sawmill_local_wood_capacity: number;
  house_cost: number;
  house_unlock_completed_waves: number;
  house_population_capacity: number;
  person_carry_capacity: number;
  arrow_tower_cost: number;
  cannon_tower_cost: number;
  max_tower_level: number;
  checksum: string;
  entities: EntityView[];
};

export type RaidDefenseCommand =
  | { type: "place_tower"; x: number; z: number; archetype: TowerArchetype }
  | { type: "place_sawmill"; x: number; z: number }
  | { type: "place_house"; x: number; z: number }
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
      wood_cost: number;
    }
  | {
      type: "sawmill_built";
      entity: number;
      cell: CellView;
      wood_cost: number;
    }
  | {
      type: "house_built";
      entity: number;
      cell: CellView;
      wood_cost: number;
      people_added: number;
      population_capacity: number;
    }
  | {
      type: "tower_upgraded";
      entity: number;
      archetype: TowerArchetype;
      level: number;
      wood_cost: number;
    }
  | { type: "wave_started"; wave: number; raiders: number }
  | {
      type: "tick_advanced";
      tick: number;
      shots: number;
      impacts: number;
      kills: number;
      wood_produced: number;
      wood_picked_up: number;
      wood_delivered: number;
      wood_stolen: number;
      town_damage: number;
      completed_wave: number | null;
    };

export type DispatchResponse = {
  contract_version: 6;
  ok: boolean;
  event: RaidDefenseEvent | null;
  error: { code: string } | null;
  snapshot: SnapshotView;
};
