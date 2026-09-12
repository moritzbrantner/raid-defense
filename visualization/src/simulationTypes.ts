export type ProvinceView = {
  id: number;
  name: string;
  controlled: boolean;
  fort_level: number;
  garrison: number;
  control: number;
  threat: number;
};

export type RaidView = {
  target: number;
  strength: number;
  eta_days: number;
};

export type SnapshotView = {
  contract_version: 1;
  seed: string;
  day: number;
  treasury: number;
  influence: number;
  capital_health: number;
  checksum: string;
  active_raid: RaidView | null;
  provinces: ProvinceView[];
};

export type RaidDefenseCommand =
  | { type: "build_fort"; province: number }
  | { type: "recruit"; province: number; soldiers: number }
  | { type: "claim_province"; province: number }
  | { type: "advance_day" };

export type RaidDefenseEvent =
  | { type: "fort_built"; province: number; level: number }
  | { type: "recruited"; province: number; soldiers: number }
  | { type: "province_claimed"; province: number }
  | { type: "day_advanced"; day: number }
  | { type: "raid_sighted"; raid: RaidView }
  | { type: "raid_repelled"; province: number; losses: number }
  | { type: "raid_breached"; province: number; damage: number };

export type DispatchResponse = {
  contract_version: 1;
  ok: boolean;
  event: RaidDefenseEvent | null;
  error: { code: string } | null;
  snapshot: SnapshotView;
};
