export type RuntimeMapLocation = {
  x: number;
  y: number;
  elevation: number;
};

export type RuntimeBuildingView = {
  id: number;
  kind: string;
  label: string;
  location: RuntimeMapLocation;
  height: number;
  footprint: { width: number; depth: number };
  level: number;
  required_workers: number;
  assigned_workers: number;
  manned: boolean;
  status: string;
  production: string;
  inventory: RuntimeResourceView[];
  logistics: RuntimeBuildingLogisticsView | null;
  stats: Record<string, number>;
};

export type RuntimeBuildingLogisticsView = {
  role: string;
  connected_storage_id: number | null;
  active_routes: number;
  route_slots: number;
  service_radius: number | null;
  routes: RuntimeLogisticsRouteView[];
  blocked: boolean;
  spoiling: boolean;
};

export type RuntimeLogisticsRouteView = {
  source_building_id: number;
  target_building_id: number;
  amount: number;
  started_at_seconds: number;
  completes_at_seconds: number;
  waypoints: RuntimeMapLocation[];
};

export type RuntimeResourceView = {
  id: string;
  label: string;
  amount: number;
  capacity: number | null;
};

export type RuntimeJobView = {
  id: number;
  kind: string;
  completes_at_seconds: number;
  assigned_entities: number[];
};

export type RuntimePathView = {
  id: number;
  kind: string;
  waypoints: RuntimeMapLocation[];
};

export type RuntimeAreaView = {
  id: number;
  kind: string;
  tiles: RuntimeMapLocation[];
};

export type RuntimeEntityView = {
  id: number;
  blueprint: { Unit?: string; Npc?: string };
  kind: string;
  label: string;
  location: RuntimeMapLocation;
  assigned_building: number | null;
  assigned_job: number | null;
  stats: Record<string, number>;
};

export type RuntimeEncounteredUnitView = {
  kind: string;
  label: string;
  encountered_at_seconds: number;
  current_count: number;
};

export type RuntimeAttackWaveUnitView = {
  kind: string;
  label: string;
  count: number;
};

export type RuntimeAttackWaveView = {
  id: number;
  label: string;
  encountered_at_seconds: number;
  entry: RuntimeMapLocation;
  units: RuntimeAttackWaveUnitView[];
};

export type RuntimeProvinceView = {
  id: number;
  name: string;
  location: RuntimeMapLocation;
  control: number;
  loyalty: number;
  threat: number;
  prosperity: number;
  stats: Record<string, number>;
};

export type RuntimeRivalView = {
  id: number;
  kind: string;
  label: string;
  location: RuntimeMapLocation;
  stats: Record<string, number>;
};

export type RuntimeAlertView = {
  severity: string;
  message: string;
};

export type RuntimeObjectiveView = {
  id: string;
  label: string;
  current: number;
  target: number;
  complete: boolean;
};

export type RuntimeRaidDefenseSummary = {
  province_count: number;
  average_control: number;
  average_loyalty: number;
  total_threat: number;
  tax_rate: number;
  active_legions: number;
  influence_rank: number;
  won: boolean;
  lost: boolean;
  critical: boolean;
};

export type RuntimeScenarioConfig = {
  buildings: Record<string, Record<string, number>>;
  units: Record<string, Record<string, number>>;
};

export type RuntimeRaidDefenseView = {
  now_seconds: number;
  scenario_config: RuntimeScenarioConfig;
  resources: RuntimeResourceView[];
  food_logistics: {
    delivered_last_minute: number;
    spoiled_last_minute: number;
    blocked_farms: number;
    strained_storage: number;
    reserve_state: string;
  };
  buildings: RuntimeBuildingView[];
  jobs: RuntimeJobView[];
  paths: RuntimePathView[];
  areas: RuntimeAreaView[];
  entities: RuntimeEntityView[];
  tech_nodes: string[];
  available_tech_nodes: string[];
  upgrades: string[];
  encountered_units: RuntimeEncounteredUnitView[];
  encountered_attack_waves: RuntimeAttackWaveView[];
  alerts: RuntimeAlertView[];
  objectives: RuntimeObjectiveView[];
  summary: RuntimeRaidDefenseSummary;
};

export type RuntimeRaidDefenseCommandResponse = {
  accepted: boolean;
  version: number;
  checksum: string;
  events: unknown[];
  view: RuntimeRaidDefenseView;
  error: string | null;
};

export type SimulationSaveState = {
  id: string;
  name: string;
  seed: string;
  created_at: string;
  now_seconds: number;
  scenario_config?: RuntimeScenarioConfig;
  snapshot_json: string;
};

export function provincesFromView(view: RuntimeRaidDefenseView): RuntimeProvinceView[] {
  return view.entities
    .filter((entity) => entity.kind === "province")
    .map((entity) => ({
      id: entity.id,
      name: entity.label,
      location: entity.location,
      control: entity.stats.control ?? 0,
      loyalty: entity.stats.loyalty ?? 0,
      threat: entity.stats.threat ?? 0,
      prosperity: entity.stats.prosperity ?? 0,
      stats: entity.stats,
    }));
}

export function rivalsFromView(view: RuntimeRaidDefenseView): RuntimeRivalView[] {
  return view.entities
    .filter((entity) => entity.kind === "rival_house")
    .map((entity) => ({
      id: entity.id,
      kind: entity.kind,
      label: entity.label,
      location: entity.location,
      stats: entity.stats,
    }));
}
