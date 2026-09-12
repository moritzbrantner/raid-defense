#![forbid(unsafe_code)]

use ecs_sparse_set::SparseWorld;
use ecs_workload::{EntityId, Operation, Position};

pub use raid_defense_rules::{Command, Event, GameError, Province, ProvinceId, Raid};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProvinceLayout {
    pub x: i64,
    pub y: i64,
    pub topology: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    rules: raid_defense_rules::GameState,
    ecs: SparseWorld,
}

impl GameState {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        let rules = raid_defense_rules::GameState::new(seed);
        let mut ecs = SparseWorld::new();

        for province in rules.provinces() {
            let entity = province_entity(province.id);
            ecs.apply(Operation::Spawn(entity))
                .expect("raid-defense province entity ids are unique");
            let layout = canonical_layout(province.id);
            ecs.apply(Operation::SetPosition(
                entity,
                Position::new3(layout.x, layout.y, layout.topology),
            ))
            .expect("newly spawned province entities accept layout positions");
        }

        Self { rules, ecs }
    }

    #[must_use]
    pub fn seed(&self) -> u64 {
        self.rules.seed()
    }

    #[must_use]
    pub fn day(&self) -> u32 {
        self.rules.day()
    }

    #[must_use]
    pub fn treasury(&self) -> u32 {
        self.rules.treasury()
    }

    #[must_use]
    pub fn influence(&self) -> u32 {
        self.rules.influence()
    }

    #[must_use]
    pub fn capital_health(&self) -> u16 {
        self.rules.capital_health()
    }

    #[must_use]
    pub fn provinces(&self) -> &[Province] {
        self.rules.provinces()
    }

    #[must_use]
    pub fn active_raid(&self) -> Option<Raid> {
        self.rules.active_raid()
    }

    #[must_use]
    pub fn ecs_entity_count(&self) -> usize {
        self.ecs.snapshot().entities().len()
    }

    #[must_use]
    pub fn province_layout(&self, province: ProvinceId) -> Option<ProvinceLayout> {
        let entity = province_entity(province);
        self.ecs
            .snapshot()
            .entities()
            .iter()
            .find(|snapshot| snapshot.id == entity)
            .and_then(|snapshot| snapshot.position)
            .map(|position| ProvinceLayout {
                x: position.x,
                y: position.y,
                topology: position.z,
            })
    }

    pub fn apply(&mut self, command: Command) -> Result<Event, GameError> {
        if let Some(province) = command_province(command)
            && self.province_layout(province).is_none()
        {
            return Err(GameError::UnknownProvince);
        }

        self.rules.apply(command)
    }

    #[must_use]
    pub fn checksum(&self) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        feed_u64(&mut hash, self.rules.checksum());

        for entity in self.ecs.snapshot().entities() {
            feed_u64(&mut hash, u64::from(entity.id.0));
            match entity.position {
                Some(position) => {
                    feed_byte(&mut hash, 1);
                    feed_i64(&mut hash, position.x);
                    feed_i64(&mut hash, position.y);
                    feed_i64(&mut hash, position.z);
                }
                None => feed_byte(&mut hash, 0),
            }
            match entity.velocity {
                Some(velocity) => {
                    feed_byte(&mut hash, 1);
                    feed_i32(&mut hash, velocity.x);
                    feed_i32(&mut hash, velocity.y);
                    feed_i32(&mut hash, velocity.z);
                }
                None => feed_byte(&mut hash, 0),
            }
        }

        hash
    }
}

pub fn replay(seed: u64, commands: &[Command]) -> Result<GameState, GameError> {
    let mut state = GameState::new(seed);
    for command in commands {
        state.apply(*command)?;
    }
    Ok(state)
}

const fn province_entity(province: ProvinceId) -> EntityId {
    EntityId(province as u32)
}

const fn canonical_layout(province: ProvinceId) -> ProvinceLayout {
    let topology = province as i64;
    ProvinceLayout {
        x: 12 + topology * 19,
        y: if province.is_multiple_of(2) { 58 } else { 34 },
        topology,
    }
}

const fn command_province(command: Command) -> Option<ProvinceId> {
    match command {
        Command::BuildFort { province }
        | Command::Recruit { province, .. }
        | Command::ClaimProvince { province } => Some(province),
        Command::AdvanceDay => None,
    }
}

fn feed_u64(hash: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        feed_byte(hash, byte);
    }
}

fn feed_i64(hash: &mut u64, value: i64) {
    for byte in value.to_le_bytes() {
        feed_byte(hash, byte);
    }
}

fn feed_i32(hash: &mut u64, value: i32) {
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
    fn runtime_creates_one_sparse_world_entity_per_province() {
        let state = GameState::new(0x5eed);

        assert_eq!(state.ecs_entity_count(), state.provinces().len());
        for province in state.provinces() {
            let layout = state
                .province_layout(province.id)
                .expect("every province must have an ECS entity and layout");
            assert_eq!(layout.topology, i64::from(province.id));
        }
    }

    #[test]
    fn ecs_runtime_preserves_rule_core_outcomes() {
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
        ];
        let mut runtime = GameState::new(7);
        let mut rules = raid_defense_rules::GameState::new(7);

        for command in commands {
            assert_eq!(runtime.apply(command), rules.apply(command));
        }

        assert_eq!(runtime.day(), rules.day());
        assert_eq!(runtime.treasury(), rules.treasury());
        assert_eq!(runtime.influence(), rules.influence());
        assert_eq!(runtime.capital_health(), rules.capital_health());
        assert_eq!(runtime.provinces(), rules.provinces());
        assert_eq!(runtime.active_raid(), rules.active_raid());
    }

    #[test]
    fn ecs_runtime_replay_is_deterministic() {
        let commands = [
            Command::BuildFort { province: 0 },
            Command::AdvanceDay,
            Command::AdvanceDay,
            Command::AdvanceDay,
        ];

        let first = replay(17, &commands).expect("valid replay");
        let second = replay(17, &commands).expect("same replay");

        assert_eq!(first, second);
        assert_eq!(first.checksum(), second.checksum());
    }
}
