use super::*;

#[cfg_attr(feature = "contracts", derive(schemars::JsonSchema, ts_rs::TS))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RaidDefenseCommandRequest {
    pub command_id: CommandId,
    pub world_id: WorldId,
    pub player_id: PlayerId,
    pub expected_version: u64,
    pub command: GameCommand,
}

#[cfg_attr(feature = "contracts", derive(schemars::JsonSchema, ts_rs::TS))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RaidDefenseCommand {
    Engine(GameCommand),
    SetTaxRate {
        building: BuildingId,
        value: i64,
    },
    ClaimProvince {
        kind: NpcKind,
        name: Option<String>,
        location: MapLocation,
    },
    RecruitEngineer {
        location: MapLocation,
    },
}

impl From<GameCommand> for RaidDefenseCommand {
    fn from(command: GameCommand) -> Self {
        Self::Engine(command)
    }
}

#[cfg_attr(feature = "contracts", derive(schemars::JsonSchema, ts_rs::TS))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RaidDefenseCommandResponse {
    pub accepted: bool,
    pub version: u64,
    pub checksum: String,
    pub events: Vec<GameEvent>,
    pub view: RaidDefenseView,
    pub error: Option<String>,
}

#[cfg_attr(feature = "contracts", derive(schemars::JsonSchema, ts_rs::TS))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RaidDefenseView {
    pub now_seconds: u64,
    pub resources: Vec<ResourceView>,
    pub food_logistics: FoodLogisticsView,
    pub buildings: Vec<BuildingView>,
    pub jobs: Vec<JobView>,
    pub paths: Vec<PathView>,
    pub areas: Vec<AreaView>,
    pub entities: Vec<EntityView>,
    pub tech_nodes: Vec<String>,
    pub available_tech_nodes: Vec<String>,
    pub upgrades: Vec<String>,
    pub encountered_units: Vec<EncounteredUnitView>,
    pub encountered_attack_waves: Vec<AttackWaveView>,
    pub alerts: Vec<AlertView>,
    pub objectives: Vec<ObjectiveView>,
    pub summary: RaidDefenseSummary,
}

#[cfg_attr(feature = "contracts", derive(schemars::JsonSchema, ts_rs::TS))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResourceView {
    pub id: String,
    pub label: String,
    pub amount: u64,
    pub capacity: Option<u64>,
}

#[cfg_attr(feature = "contracts", derive(schemars::JsonSchema, ts_rs::TS))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BuildingView {
    pub id: u64,
    pub kind: String,
    pub label: String,
    pub location: MapLocation,
    pub height: u32,
    pub footprint: BuildingFootprint,
    pub level: u32,
    pub required_workers: u32,
    pub assigned_workers: u32,
    pub manned: bool,
    pub status: String,
    pub production: String,
    pub inventory: Vec<ResourceView>,
    pub logistics: Option<BuildingLogisticsView>,
    pub stats: BTreeMap<String, i64>,
}

#[cfg_attr(feature = "contracts", derive(schemars::JsonSchema, ts_rs::TS))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BuildingLogisticsView {
    pub role: String,
    pub connected_storage_id: Option<u64>,
    pub active_routes: u32,
    pub route_slots: u32,
    pub service_radius: Option<u32>,
    pub blocked: bool,
    pub spoiling: bool,
}

#[cfg_attr(feature = "contracts", derive(schemars::JsonSchema, ts_rs::TS))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct JobView {
    pub id: u64,
    pub kind: String,
    pub completes_at_seconds: u64,
    pub assigned_entities: Vec<u64>,
}

#[cfg_attr(feature = "contracts", derive(schemars::JsonSchema, ts_rs::TS))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PathView {
    pub id: u64,
    pub kind: String,
    pub waypoints: Vec<MapLocation>,
}

#[cfg_attr(feature = "contracts", derive(schemars::JsonSchema, ts_rs::TS))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AreaView {
    pub id: u64,
    pub kind: String,
    pub tiles: Vec<MapLocation>,
}

#[cfg_attr(feature = "contracts", derive(schemars::JsonSchema, ts_rs::TS))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EntityView {
    pub id: u64,
    pub blueprint: EntityBlueprintRef,
    pub kind: String,
    pub label: String,
    pub location: MapLocation,
    pub assigned_building: Option<u64>,
    pub assigned_job: Option<u64>,
    pub stats: BTreeMap<String, i64>,
}

#[cfg_attr(feature = "contracts", derive(schemars::JsonSchema, ts_rs::TS))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EncounteredUnitView {
    pub kind: String,
    pub label: String,
    pub encountered_at_seconds: u64,
    pub current_count: u32,
}

#[cfg_attr(feature = "contracts", derive(schemars::JsonSchema, ts_rs::TS))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AttackWaveUnitView {
    pub kind: String,
    pub label: String,
    pub count: u32,
}

#[cfg_attr(feature = "contracts", derive(schemars::JsonSchema, ts_rs::TS))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AttackWaveView {
    pub id: u32,
    pub label: String,
    pub encountered_at_seconds: u64,
    pub entry: MapLocation,
    pub units: Vec<AttackWaveUnitView>,
}

#[cfg_attr(feature = "contracts", derive(schemars::JsonSchema, ts_rs::TS))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProvinceView {
    pub id: u64,
    pub name: String,
    pub location: MapLocation,
    pub control: i64,
    pub loyalty: i64,
    pub threat: i64,
    pub prosperity: i64,
    pub stats: BTreeMap<String, i64>,
}

#[cfg_attr(feature = "contracts", derive(schemars::JsonSchema, ts_rs::TS))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NpcView {
    pub id: u64,
    pub kind: String,
    pub label: String,
    pub location: MapLocation,
    pub stats: BTreeMap<String, i64>,
}

#[cfg_attr(feature = "contracts", derive(schemars::JsonSchema, ts_rs::TS))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AlertView {
    pub severity: String,
    pub message: String,
}

#[cfg_attr(feature = "contracts", derive(schemars::JsonSchema, ts_rs::TS))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FoodLogisticsView {
    pub delivered_last_minute: u64,
    pub spoiled_last_minute: u64,
    pub blocked_farms: u32,
    pub strained_storage: u32,
    pub reserve_state: String,
}

#[cfg_attr(feature = "contracts", derive(schemars::JsonSchema, ts_rs::TS))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ObjectiveView {
    pub id: String,
    pub label: String,
    pub current: i64,
    pub target: i64,
    pub complete: bool,
}

#[cfg_attr(feature = "contracts", derive(schemars::JsonSchema, ts_rs::TS))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RaidDefenseSummary {
    pub province_count: u32,
    pub average_control: i64,
    pub average_loyalty: i64,
    pub total_threat: i64,
    pub tax_rate: i64,
    pub active_legions: u64,
    pub influence_rank: u32,
    pub won: bool,
    pub critical: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvinceClaimRequirements {
    pub outpost_kinds: Vec<&'static str>,
    pub min_level: u32,
    pub min_security: i64,
    pub claim_cost: Vec<ResourceAmount>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProvinceClaimError {
    UnknownProvinceKind(String),
    NoOutpostAtLocation {
        province_kind: String,
        location: MapLocation,
    },
    OutpostUnavailable {
        province_kind: String,
        outpost: BuildingId,
    },
    RequirementsNotMet {
        province_kind: String,
        outpost: BuildingId,
        requirements: ProvinceClaimRequirements,
    },
    AlreadyClaimed {
        outpost: BuildingId,
    },
}

impl fmt::Display for ProvinceClaimError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProvinceClaimError::UnknownProvinceKind(kind) => {
                write!(formatter, "unknown province kind {kind}")
            }
            ProvinceClaimError::NoOutpostAtLocation {
                province_kind,
                location,
            } => write!(
                formatter,
                "no claim outpost at ({}, {}) for {province_kind}",
                location.x, location.y
            ),
            ProvinceClaimError::OutpostUnavailable {
                province_kind,
                outpost,
            } => write!(
                formatter,
                "outpost {outpost} is unavailable for {province_kind}"
            ),
            ProvinceClaimError::RequirementsNotMet {
                province_kind,
                outpost,
                requirements,
            } => write!(
                formatter,
                "outpost {outpost} does not meet {province_kind} requirements: level {}, security {}, one of {:?}",
                requirements.min_level, requirements.min_security, requirements.outpost_kinds
            ),
            ProvinceClaimError::AlreadyClaimed { outpost } => {
                write!(formatter, "outpost {outpost} already anchors a province")
            }
        }
    }
}

impl Error for ProvinceClaimError {}

#[derive(Debug)]
pub enum RaidDefenseError {
    Engine(EngineError),
    World(GameWorldError),
    Serde(serde_json::Error),
    Province(ProvinceClaimError),
    InvalidCommand(String),
    MissingPlayer(PlayerId),
}

impl fmt::Display for RaidDefenseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RaidDefenseError::Engine(error) => error.fmt(formatter),
            RaidDefenseError::World(error) => error.fmt(formatter),
            RaidDefenseError::Serde(error) => error.fmt(formatter),
            RaidDefenseError::Province(error) => error.fmt(formatter),
            RaidDefenseError::InvalidCommand(message) => formatter.write_str(message),
            RaidDefenseError::MissingPlayer(player) => write!(formatter, "missing player {player}"),
        }
    }
}

impl Error for RaidDefenseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RaidDefenseError::Engine(error) => Some(error),
            RaidDefenseError::World(error) => Some(error),
            RaidDefenseError::Serde(error) => Some(error),
            RaidDefenseError::Province(error) => Some(error),
            RaidDefenseError::InvalidCommand(_) => None,
            RaidDefenseError::MissingPlayer(_) => None,
        }
    }
}

impl From<EngineError> for RaidDefenseError {
    fn from(value: EngineError) -> Self {
        Self::Engine(value)
    }
}

impl From<GameWorldError> for RaidDefenseError {
    fn from(value: GameWorldError) -> Self {
        Self::World(value)
    }
}

impl From<serde_json::Error> for RaidDefenseError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}

impl From<ProvinceClaimError> for RaidDefenseError {
    fn from(value: ProvinceClaimError) -> Self {
        Self::Province(value)
    }
}

pub fn raid_defense_checksum(view: &RaidDefenseView) -> Result<String, RaidDefenseError> {
    let bytes = serde_json::to_vec(view)?;
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    Ok(format!("{hash:016x}"))
}

pub fn command_response(
    accepted: bool,
    version: u64,
    events: Vec<GameEvent>,
    state: &GameState,
    error: Option<String>,
) -> Result<RaidDefenseCommandResponse, RaidDefenseError> {
    let view = raid_defense_view(state);
    let checksum = raid_defense_checksum(&view)?;
    Ok(RaidDefenseCommandResponse {
        accepted,
        version,
        checksum,
        events,
        view,
        error,
    })
}
