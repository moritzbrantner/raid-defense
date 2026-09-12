#![forbid(unsafe_code)]

use raid_defense_core::{Command, Event, GameError, GameState, Raid};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

const CONTRACT_VERSION: u8 = 1;

#[wasm_bindgen]
pub struct RaidDefenseGame {
    state: GameState,
}

#[wasm_bindgen]
impl RaidDefenseGame {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32) -> Self {
        Self {
            state: GameState::new(u64::from(seed)),
        }
    }

    pub fn snapshot(&self) -> String {
        serialize(&SnapshotDto::from(&self.state))
    }

    pub fn dispatch(&mut self, command_json: &str) -> String {
        dispatch_json(&mut self.state, command_json)
    }

    pub fn checksum(&self) -> String {
        self.state.checksum().to_string()
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum CommandDto {
    BuildFort { province: u8 },
    Recruit { province: u8, soldiers: u16 },
    ClaimProvince { province: u8 },
    AdvanceDay,
}

impl From<CommandDto> for Command {
    fn from(value: CommandDto) -> Self {
        match value {
            CommandDto::BuildFort { province } => Self::BuildFort { province },
            CommandDto::Recruit { province, soldiers } => Self::Recruit { province, soldiers },
            CommandDto::ClaimProvince { province } => Self::ClaimProvince { province },
            CommandDto::AdvanceDay => Self::AdvanceDay,
        }
    }
}

#[derive(Serialize)]
struct DispatchResponseDto {
    contract_version: u8,
    ok: bool,
    event: Option<EventDto>,
    error: Option<ErrorDto>,
    snapshot: SnapshotDto,
}

#[derive(Serialize)]
struct ErrorDto {
    code: &'static str,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum EventDto {
    FortBuilt { province: u8, level: u8 },
    Recruited { province: u8, soldiers: u16 },
    ProvinceClaimed { province: u8 },
    DayAdvanced { day: u32 },
    RaidSighted { raid: RaidDto },
    RaidRepelled { province: u8, losses: u16 },
    RaidBreached { province: u8, damage: u16 },
}

impl From<Event> for EventDto {
    fn from(value: Event) -> Self {
        match value {
            Event::FortBuilt { province, level } => Self::FortBuilt { province, level },
            Event::Recruited { province, soldiers } => Self::Recruited { province, soldiers },
            Event::ProvinceClaimed { province } => Self::ProvinceClaimed { province },
            Event::DayAdvanced { day } => Self::DayAdvanced { day },
            Event::RaidSighted(raid) => Self::RaidSighted {
                raid: RaidDto::from(raid),
            },
            Event::RaidRepelled { province, losses } => Self::RaidRepelled { province, losses },
            Event::RaidBreached { province, damage } => Self::RaidBreached { province, damage },
        }
    }
}

#[derive(Clone, Copy, Serialize)]
struct RaidDto {
    target: u8,
    strength: u16,
    eta_days: u8,
}

impl From<Raid> for RaidDto {
    fn from(value: Raid) -> Self {
        Self {
            target: value.target,
            strength: value.strength,
            eta_days: value.eta_days,
        }
    }
}

#[derive(Serialize)]
struct SnapshotDto {
    contract_version: u8,
    seed: String,
    day: u32,
    treasury: u32,
    influence: u32,
    capital_health: u16,
    checksum: String,
    active_raid: Option<RaidDto>,
    provinces: Vec<ProvinceDto>,
}

impl From<&GameState> for SnapshotDto {
    fn from(state: &GameState) -> Self {
        Self {
            contract_version: CONTRACT_VERSION,
            seed: state.seed().to_string(),
            day: state.day(),
            treasury: state.treasury(),
            influence: state.influence(),
            capital_health: state.capital_health(),
            checksum: state.checksum().to_string(),
            active_raid: state.active_raid().map(RaidDto::from),
            provinces: state
                .provinces()
                .iter()
                .map(|province| ProvinceDto {
                    id: province.id,
                    name: province.name,
                    controlled: province.controlled,
                    fort_level: province.fort_level,
                    garrison: province.garrison,
                    control: province.control,
                    threat: province.threat,
                })
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct ProvinceDto {
    id: u8,
    name: &'static str,
    controlled: bool,
    fort_level: u8,
    garrison: u16,
    control: u8,
    threat: u8,
}

fn dispatch_json(state: &mut GameState, command_json: &str) -> String {
    let command = match serde_json::from_str::<CommandDto>(command_json) {
        Ok(command) => Command::from(command),
        Err(_) => {
            return serialize(&DispatchResponseDto {
                contract_version: CONTRACT_VERSION,
                ok: false,
                event: None,
                error: Some(ErrorDto {
                    code: "invalid_command_json",
                }),
                snapshot: SnapshotDto::from(&*state),
            });
        }
    };

    match state.apply(command) {
        Ok(event) => serialize(&DispatchResponseDto {
            contract_version: CONTRACT_VERSION,
            ok: true,
            event: Some(EventDto::from(event)),
            error: None,
            snapshot: SnapshotDto::from(&*state),
        }),
        Err(error) => serialize(&DispatchResponseDto {
            contract_version: CONTRACT_VERSION,
            ok: false,
            event: None,
            error: Some(ErrorDto {
                code: error_code(error),
            }),
            snapshot: SnapshotDto::from(&*state),
        }),
    }
}

const fn error_code(error: GameError) -> &'static str {
    match error {
        GameError::UnknownProvince => "unknown_province",
        GameError::ProvinceAlreadyControlled => "province_already_controlled",
        GameError::ProvinceNotControlled => "province_not_controlled",
        GameError::FrontierNotConnected => "frontier_not_connected",
        GameError::FortAtMaximumLevel => "fort_at_maximum_level",
        GameError::InvalidRecruitment => "invalid_recruitment",
        GameError::GarrisonCapacityExceeded => "garrison_capacity_exceeded",
        GameError::InsufficientTreasury => "insufficient_treasury",
    }
}

fn serialize<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("raid-defense contract DTOs must serialize")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn checksum_from_response(response: &str) -> String {
        let value: Value = serde_json::from_str(response).expect("response should be JSON");
        value["snapshot"]["checksum"]
            .as_str()
            .expect("checksum should be a string")
            .to_owned()
    }

    #[test]
    fn invalid_json_is_fail_closed_and_does_not_mutate_state() {
        let mut state = GameState::new(42);
        let before = state.checksum();

        let response = dispatch_json(&mut state, "not json");
        let value: Value = serde_json::from_str(&response).expect("response should be JSON");

        assert_eq!(value["ok"], false);
        assert_eq!(value["error"]["code"], "invalid_command_json");
        assert_eq!(state.checksum(), before);
    }

    #[test]
    fn adapter_checksum_matches_native_core_after_same_command() {
        let mut adapted = GameState::new(7);
        let response = dispatch_json(&mut adapted, r#"{"type":"build_fort","province":0}"#);

        let mut native = GameState::new(7);
        native
            .apply(Command::BuildFort { province: 0 })
            .expect("native command should succeed");

        assert_eq!(
            checksum_from_response(&response),
            native.checksum().to_string()
        );
        assert_eq!(adapted, native);
    }

    #[test]
    fn rejected_rule_error_uses_stable_code_and_preserves_checksum() {
        let mut state = GameState::new(9);
        let before = state.checksum();

        let response = dispatch_json(&mut state, r#"{"type":"claim_province","province":2}"#);
        let value: Value = serde_json::from_str(&response).expect("response should be JSON");

        assert_eq!(value["ok"], false);
        assert_eq!(value["error"]["code"], "frontier_not_connected");
        assert_eq!(checksum_from_response(&response), before.to_string());
        assert_eq!(state.checksum(), before);
    }
}
