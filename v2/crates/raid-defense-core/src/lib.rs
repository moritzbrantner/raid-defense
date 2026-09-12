#![forbid(unsafe_code)]

pub type ProvinceId = u8;

const MAX_CONTROL: u8 = 100;
const MAX_FORT_LEVEL: u8 = 3;
const FORT_BASE_COST: u32 = 20;
const CLAIM_BASE_COST: u32 = 30;
const STARTING_TREASURY: u32 = 100;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
    seed: u64,
    day: u32,
    treasury: u32,
    influence: u32,
    capital_health: u16,
    provinces: Vec<Province>,
    active_raid: Option<Raid>,
    event_nonce: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Province {
    pub id: ProvinceId,
    pub name: &'static str,
    pub controlled: bool,
    pub fort_level: u8,
    pub garrison: u16,
    pub control: u8,
    pub threat: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Raid {
    pub target: ProvinceId,
    pub strength: u16,
    pub eta_days: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    BuildFort { province: ProvinceId },
    Recruit { province: ProvinceId, soldiers: u16 },
    ClaimProvince { province: ProvinceId },
    AdvanceDay,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    FortBuilt { province: ProvinceId, level: u8 },
    Recruited { province: ProvinceId, soldiers: u16 },
    ProvinceClaimed { province: ProvinceId },
    DayAdvanced { day: u32 },
    RaidSighted(Raid),
    RaidRepelled { province: ProvinceId, losses: u16 },
    RaidBreached { province: ProvinceId, damage: u16 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameError {
    UnknownProvince,
    ProvinceAlreadyControlled,
    ProvinceNotControlled,
    FrontierNotConnected,
    FortAtMaximumLevel,
    InvalidRecruitment,
    GarrisonCapacityExceeded,
    InsufficientTreasury,
}

impl GameState {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        let names = [
            "Capital",
            "North March",
            "River Ward",
            "Iron Hills",
            "Outer Reach",
        ];
        let provinces = names
            .into_iter()
            .enumerate()
            .map(|(index, name)| Province {
                id: index as ProvinceId,
                name,
                controlled: index == 0,
                fort_level: 0,
                garrison: if index == 0 { 8 } else { 0 },
                control: if index == 0 { 70 } else { 0 },
                threat: initial_threat(seed, index as ProvinceId),
            })
            .collect();

        Self {
            seed,
            day: 0,
            treasury: STARTING_TREASURY,
            influence: 0,
            capital_health: 100,
            provinces,
            active_raid: None,
            event_nonce: 0,
        }
    }

    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    #[must_use]
    pub const fn day(&self) -> u32 {
        self.day
    }

    #[must_use]
    pub const fn treasury(&self) -> u32 {
        self.treasury
    }

    #[must_use]
    pub const fn influence(&self) -> u32 {
        self.influence
    }

    #[must_use]
    pub const fn capital_health(&self) -> u16 {
        self.capital_health
    }

    #[must_use]
    pub fn provinces(&self) -> &[Province] {
        &self.provinces
    }

    pub const fn active_raid(&self) -> Option<Raid> {
        self.active_raid
    }

    pub fn apply(&mut self, command: Command) -> Result<Event, GameError> {
        match command {
            Command::BuildFort { province } => self.build_fort(province),
            Command::Recruit { province, soldiers } => self.recruit(province, soldiers),
            Command::ClaimProvince { province } => self.claim_province(province),
            Command::AdvanceDay => Ok(self.advance_day()),
        }
    }

    #[must_use]
    pub fn checksum(&self) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        feed_u64(&mut hash, self.seed);
        feed_u64(&mut hash, u64::from(self.day));
        feed_u64(&mut hash, u64::from(self.treasury));
        feed_u64(&mut hash, u64::from(self.influence));
        feed_u64(&mut hash, u64::from(self.capital_health));
        feed_u64(&mut hash, self.event_nonce);

        for province in &self.provinces {
            feed_u64(&mut hash, u64::from(province.id));
            for byte in province.name.bytes() {
                feed_byte(&mut hash, byte);
            }
            feed_byte(&mut hash, u8::from(province.controlled));
            feed_byte(&mut hash, province.fort_level);
            feed_u64(&mut hash, u64::from(province.garrison));
            feed_byte(&mut hash, province.control);
            feed_byte(&mut hash, province.threat);
        }

        match self.active_raid {
            Some(raid) => {
                feed_byte(&mut hash, 1);
                feed_byte(&mut hash, raid.target);
                feed_u64(&mut hash, u64::from(raid.strength));
                feed_byte(&mut hash, raid.eta_days);
            }
            None => feed_byte(&mut hash, 0),
        }

        hash
    }

    fn build_fort(&mut self, province_id: ProvinceId) -> Result<Event, GameError> {
        let index = self.province_index(province_id)?;
        let province = &self.provinces[index];
        if !province.controlled {
            return Err(GameError::ProvinceNotControlled);
        }
        if province.fort_level >= MAX_FORT_LEVEL {
            return Err(GameError::FortAtMaximumLevel);
        }

        let next_level = province.fort_level + 1;
        let cost = FORT_BASE_COST * u32::from(next_level);
        if self.treasury < cost {
            return Err(GameError::InsufficientTreasury);
        }

        self.treasury -= cost;
        let province = &mut self.provinces[index];
        province.fort_level = next_level;
        province.control = province.control.saturating_add(5).min(MAX_CONTROL);

        Ok(Event::FortBuilt {
            province: province_id,
            level: next_level,
        })
    }

    fn recruit(&mut self, province_id: ProvinceId, soldiers: u16) -> Result<Event, GameError> {
        if soldiers == 0 {
            return Err(GameError::InvalidRecruitment);
        }

        let index = self.province_index(province_id)?;
        let province = &self.provinces[index];
        if !province.controlled {
            return Err(GameError::ProvinceNotControlled);
        }

        let capacity = 20_u16 + u16::from(province.fort_level) * 20;
        if province.garrison.saturating_add(soldiers) > capacity {
            return Err(GameError::GarrisonCapacityExceeded);
        }

        let cost = u32::from(soldiers);
        if self.treasury < cost {
            return Err(GameError::InsufficientTreasury);
        }

        self.treasury -= cost;
        self.provinces[index].garrison += soldiers;

        Ok(Event::Recruited {
            province: province_id,
            soldiers,
        })
    }

    fn claim_province(&mut self, province_id: ProvinceId) -> Result<Event, GameError> {
        let index = self.province_index(province_id)?;
        let province = &self.provinces[index];
        if province.controlled {
            return Err(GameError::ProvinceAlreadyControlled);
        }

        let connected = self.provinces.iter().any(|candidate| {
            candidate.controlled
                && candidate.fort_level > 0
                && candidate.control >= 50
                && are_adjacent(candidate.id, province_id)
        });
        if !connected {
            return Err(GameError::FrontierNotConnected);
        }

        let cost = CLAIM_BASE_COST + u32::from(province.threat) / 2;
        if self.treasury < cost {
            return Err(GameError::InsufficientTreasury);
        }

        self.treasury -= cost;
        self.influence = self.influence.saturating_add(5);
        let province = &mut self.provinces[index];
        province.controlled = true;
        province.control = 40;

        Ok(Event::ProvinceClaimed {
            province: province_id,
        })
    }

    fn advance_day(&mut self) -> Event {
        self.day = self.day.saturating_add(1);
        self.produce_daily_resources();
        self.adjust_control();

        if let Some(mut raid) = self.active_raid.take() {
            raid.eta_days = raid.eta_days.saturating_sub(1);
            if raid.eta_days == 0 {
                return self.resolve_raid(raid);
            }
            self.active_raid = Some(raid);
            return Event::DayAdvanced { day: self.day };
        }

        if self.day.is_multiple_of(3) {
            let raid = self.spawn_raid();
            self.active_raid = Some(raid);
            return Event::RaidSighted(raid);
        }

        Event::DayAdvanced { day: self.day }
    }

    fn produce_daily_resources(&mut self) {
        let controlled = self
            .provinces
            .iter()
            .filter(|province| province.controlled)
            .count() as u32;
        self.treasury = self.treasury.saturating_add(2 + controlled * 3);
    }

    fn adjust_control(&mut self) {
        for province in &mut self.provinces {
            if !province.controlled {
                continue;
            }
            if province.garrison > 0 {
                province.control = province.control.saturating_add(1).min(MAX_CONTROL);
            } else {
                province.control = province.control.saturating_sub(1);
            }
        }
    }

    fn spawn_raid(&mut self) -> Raid {
        let frontier: Vec<ProvinceId> = self
            .provinces
            .iter()
            .filter(|province| {
                province.controlled
                    && self.provinces.iter().any(|candidate| {
                        !candidate.controlled && are_adjacent(province.id, candidate.id)
                    })
            })
            .map(|province| province.id)
            .collect();

        let targets = if frontier.is_empty() {
            vec![0]
        } else {
            frontier
        };
        let selector = mix64(self.seed ^ u64::from(self.day) ^ self.event_nonce);
        let target = targets[(selector as usize) % targets.len()];
        let strength =
            12 + (mix64(selector ^ 0xa5a5_a5a5_a5a5_a5a5) % 19) as u16 + (self.day / 4) as u16;
        self.event_nonce = self.event_nonce.wrapping_add(1);

        Raid {
            target,
            strength,
            eta_days: 2,
        }
    }

    fn resolve_raid(&mut self, raid: Raid) -> Event {
        let index = self
            .province_index(raid.target)
            .expect("raid targets are generated from existing provinces");
        let province = &self.provinces[index];
        let defense = province.garrison
            + u16::from(province.fort_level) * 12
            + u16::from(province.control / 5);

        if defense >= raid.strength {
            let losses = (raid.strength / 4).min(province.garrison);
            let province = &mut self.provinces[index];
            province.garrison -= losses;
            province.control = province.control.saturating_add(4).min(MAX_CONTROL);
            province.threat = province.threat.saturating_sub(3);
            self.influence = self.influence.saturating_add(3);
            Event::RaidRepelled {
                province: raid.target,
                losses,
            }
        } else {
            let damage = raid.strength - defense;
            let province = &mut self.provinces[index];
            province.garrison = 0;
            province.control = province.control.saturating_sub((damage / 2).min(40) as u8);
            province.threat = province.threat.saturating_add(5).min(MAX_CONTROL);
            if raid.target == 0 {
                self.capital_health = self.capital_health.saturating_sub(damage);
            }
            Event::RaidBreached {
                province: raid.target,
                damage,
            }
        }
    }

    fn province_index(&self, province_id: ProvinceId) -> Result<usize, GameError> {
        self.provinces
            .iter()
            .position(|province| province.id == province_id)
            .ok_or(GameError::UnknownProvince)
    }
}

pub fn replay(seed: u64, commands: &[Command]) -> Result<GameState, GameError> {
    let mut state = GameState::new(seed);
    for command in commands {
        state.apply(*command)?;
    }
    Ok(state)
}

const fn are_adjacent(left: ProvinceId, right: ProvinceId) -> bool {
    left.abs_diff(right) == 1
}

fn initial_threat(seed: u64, province: ProvinceId) -> u8 {
    20 + (mix64(seed ^ u64::from(province).wrapping_mul(0x9e37_79b9_7f4a_7c15)) % 41) as u8
}

fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn feed_u64(hash: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        feed_byte(hash, byte);
    }
}

fn feed_byte(hash: &mut u64, byte: u8) {
    *hash ^= u64::from(byte);
    *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_and_commands_replay_identically() {
        let commands = [
            Command::BuildFort { province: 0 },
            Command::Recruit {
                province: 0,
                soldiers: 10,
            },
            Command::AdvanceDay,
            Command::AdvanceDay,
            Command::AdvanceDay,
            Command::ClaimProvince { province: 1 },
            Command::BuildFort { province: 1 },
        ];

        let first = replay(7, &commands).expect("valid replay");
        let second = replay(7, &commands).expect("valid replay");

        assert_eq!(first, second);
        assert_eq!(first.checksum(), second.checksum());
    }

    #[test]
    fn rejected_command_does_not_mutate_state() {
        let mut state = GameState::new(11);
        let before = state.clone();

        let error = state
            .apply(Command::ClaimProvince { province: 2 })
            .expect_err("claim without a connected fort must fail");

        assert_eq!(error, GameError::FrontierNotConnected);
        assert_eq!(state, before);
    }

    #[test]
    fn claims_require_a_controlled_frontier_fort() {
        let mut state = GameState::new(19);
        assert_eq!(
            state.apply(Command::ClaimProvince { province: 1 }),
            Err(GameError::FrontierNotConnected)
        );

        state
            .apply(Command::BuildFort { province: 0 })
            .expect("capital fort should be affordable");
        let event = state
            .apply(Command::ClaimProvince { province: 1 })
            .expect("adjacent province should now be claimable");

        assert_eq!(event, Event::ProvinceClaimed { province: 1 });
        assert!(state.provinces()[1].controlled);
    }

    #[test]
    fn raid_schedule_is_seeded_and_repeatable() {
        let mut first = GameState::new(23);
        let mut second = GameState::new(23);

        for _ in 0..2 {
            assert_eq!(
                first.apply(Command::AdvanceDay),
                second.apply(Command::AdvanceDay)
            );
        }

        let first_raid = first.apply(Command::AdvanceDay).expect("advance succeeds");
        let second_raid = second.apply(Command::AdvanceDay).expect("advance succeeds");

        assert_eq!(first_raid, second_raid);
        assert!(matches!(first_raid, Event::RaidSighted(_)));
        assert_eq!(first.active_raid(), second.active_raid());
    }

    #[test]
    fn checksum_changes_when_authoritative_state_changes() {
        let mut state = GameState::new(29);
        let initial = state.checksum();

        state
            .apply(Command::BuildFort { province: 0 })
            .expect("fort build succeeds");

        assert_ne!(state.checksum(), initial);
    }
}
