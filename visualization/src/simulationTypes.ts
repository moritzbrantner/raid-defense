export type CellView = {
  x: number;
  z: number;
};

export type EntityKind = "town" | "tower" | "raider";

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
};

export type SnapshotView = {
  contract_version: 2;
  seed: string;
  tick: number;
  gold: number;
  wave: number;
  town_health: number;
  town_max_health: number;
  grid_width: number;
  grid_height: number;
  tower_cost: number;
  checksum: string;
  entities: EntityView[];
};

export type RaidDefenseCommand =
  | { type: "place_tower"; x: number; z: number }
  | { type: "start_wave" }
  | { type: "advance_tick" };

export type RaidDefenseEvent =
  | { type: "tower_built"; entity: number; cell: CellView }
  | { type: "wave_started"; wave: number; raiders: number }
  | {
      type: "tick_advanced";
      tick: number;
      shots: number;
      kills: number;
      town_damage: number;
    };

export type DispatchResponse = {
  contract_version: 2;
  ok: boolean;
  event: RaidDefenseEvent | null;
  error: { code: string } | null;
  snapshot: SnapshotView;
};
