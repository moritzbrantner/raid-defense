use crate::{TowerArchetype, TowerStats};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GameRules {
    pub economy: EconomyRules,
    pub population: PopulationRules,
    pub buildings: BuildingRules,
    pub towers: TowerRules,
    pub raids: RaidRules,
    pub cycle: CycleRules,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RulesError {
    StartingPopulationExceedsCapacity,
    ZeroSawmillInterval,
    InvalidTowerLevelCount,
    ZeroRaidersPerWave,
    ZeroRaidDamageInterval,
    ZeroDayLength,
}

impl GameRules {
    #[must_use]
    pub const fn validate(self) -> Result<(), RulesError> {
        if self.population.starting_people > self.population.base_capacity {
            return Err(RulesError::StartingPopulationExceedsCapacity);
        }
        if self.economy.sawmill_interval_ticks == 0 {
            return Err(RulesError::ZeroSawmillInterval);
        }
        if self.towers.max_level == 0 || self.towers.max_level > 3 {
            return Err(RulesError::InvalidTowerLevelCount);
        }
        if self.raids.raiders_per_wave == 0 {
            return Err(RulesError::ZeroRaidersPerWave);
        }
        if self.raids.damage_increase_every_waves == 0 {
            return Err(RulesError::ZeroRaidDamageInterval);
        }
        if self.cycle.automatic_raids && self.cycle.day_length_ticks == 0 {
            return Err(RulesError::ZeroDayLength);
        }
        Ok(())
    }

    #[must_use]
    pub const fn tower(self, archetype: TowerArchetype) -> TowerArchetypeRules {
        match archetype {
            TowerArchetype::Arrow => self.towers.arrow,
            TowerArchetype::Cannon => self.towers.cannon,
        }
    }

    #[must_use]
    pub const fn tower_level(self, archetype: TowerArchetype, level: u8) -> TowerLevelRules {
        let tower = self.tower(archetype);
        match level {
            1 => tower.levels[0],
            2 => tower.levels[1],
            _ => tower.levels[2],
        }
    }

    #[must_use]
    pub const fn tower_upgrade_cost(self, archetype: TowerArchetype, level: u8) -> Option<u32> {
        if level >= self.towers.max_level {
            return None;
        }
        self.tower_level(archetype, level).upgrade_cost
    }

    #[must_use]
    pub const fn houses_unlocked(self, completed_waves: u32) -> bool {
        completed_waves >= self.buildings.house.unlock_completed_waves
    }

    /// Stable deterministic identity for a complete rule set.
    ///
    /// A game checksum includes this value so simulations created with different
    /// rules can never be mistaken for the same authoritative state.
    #[must_use]
    pub fn fingerprint(self) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;

        feed_u64(&mut hash, u64::from(self.economy.starting_wood));
        feed_u64(&mut hash, u64::from(self.economy.town_wood_capacity));
        feed_u16(&mut hash, self.economy.sawmill_output);
        feed_u16(&mut hash, self.economy.sawmill_interval_ticks);
        feed_u64(
            &mut hash,
            u64::from(self.economy.sawmill_local_wood_capacity),
        );

        feed_u16(&mut hash, self.population.starting_people);
        feed_u16(&mut hash, self.population.base_capacity);
        feed_u16(&mut hash, self.population.person_health);
        feed_u16(&mut hash, self.population.carry_capacity);
        feed_u16(&mut hash, self.population.speed_milli);

        feed_u16(&mut hash, self.buildings.town_hall.max_health);
        feed_u64(&mut hash, u64::from(self.buildings.sawmill.wood_cost));
        feed_u16(&mut hash, self.buildings.sawmill.max_health);
        feed_u64(&mut hash, u64::from(self.buildings.house.wood_cost));
        feed_u16(&mut hash, self.buildings.house.max_health);
        feed_u64(
            &mut hash,
            u64::from(self.buildings.house.unlock_completed_waves),
        );
        feed_u16(&mut hash, self.buildings.house.population_capacity);
        feed_u16(&mut hash, self.buildings.house.people_added);

        feed_byte(&mut hash, self.towers.max_level);
        feed_u16(&mut hash, self.towers.max_health);
        feed_tower(&mut hash, self.towers.arrow);
        feed_tower(&mut hash, self.towers.cannon);

        feed_u16(&mut hash, self.raids.raiders_per_wave);
        feed_u16(&mut hash, self.raids.base_health);
        feed_u16(&mut hash, self.raids.health_per_wave);
        feed_u16(&mut hash, self.raids.base_damage);
        feed_u64(&mut hash, u64::from(self.raids.damage_increase_every_waves));
        feed_u16(&mut hash, self.raids.speed_milli);
        feed_u64(&mut hash, u64::from(self.raids.base_wood_steal));
        feed_u64(&mut hash, u64::from(self.raids.wood_steal_per_wave));

        feed_u16(&mut hash, self.cycle.day_length_ticks);
        feed_bool(&mut hash, self.cycle.automatic_raids);
        feed_bool(&mut hash, self.cycle.pause_economy_during_raids);

        hash
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EconomyRules {
    pub starting_wood: u32,
    pub town_wood_capacity: u32,
    pub sawmill_output: u16,
    pub sawmill_interval_ticks: u16,
    pub sawmill_local_wood_capacity: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PopulationRules {
    pub starting_people: u16,
    pub base_capacity: u16,
    pub person_health: u16,
    pub carry_capacity: u16,
    pub speed_milli: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuildingRules {
    pub town_hall: TownHallRules,
    pub sawmill: SawmillRules,
    pub house: HouseRules,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TownHallRules {
    pub max_health: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SawmillRules {
    pub wood_cost: u32,
    pub max_health: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HouseRules {
    pub wood_cost: u32,
    pub max_health: u16,
    pub unlock_completed_waves: u32,
    pub population_capacity: u16,
    pub people_added: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TowerRules {
    pub max_level: u8,
    pub max_health: u16,
    pub arrow: TowerArchetypeRules,
    pub cannon: TowerArchetypeRules,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TowerArchetypeRules {
    pub build_cost: u32,
    pub levels: [TowerLevelRules; 3],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TowerLevelRules {
    pub damage: u16,
    pub range_milli: i32,
    pub cooldown_ticks: u8,
    pub projectile_speed_milli: u16,
    pub upgrade_cost: Option<u32>,
}

impl TowerLevelRules {
    #[must_use]
    pub const fn stats(self) -> TowerStats {
        TowerStats {
            damage: self.damage,
            range_milli: self.range_milli,
            cooldown_ticks: self.cooldown_ticks,
            projectile_speed_milli: self.projectile_speed_milli,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RaidRules {
    pub raiders_per_wave: u16,
    pub base_health: u16,
    pub health_per_wave: u16,
    pub base_damage: u16,
    pub damage_increase_every_waves: u32,
    pub speed_milli: u16,
    pub base_wood_steal: u32,
    pub wood_steal_per_wave: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CycleRules {
    pub day_length_ticks: u16,
    pub automatic_raids: bool,
    pub pause_economy_during_raids: bool,
}

fn feed_tower(hash: &mut u64, tower: TowerArchetypeRules) {
    feed_u64(hash, u64::from(tower.build_cost));
    for level in tower.levels {
        feed_u16(hash, level.damage);
        feed_i32(hash, level.range_milli);
        feed_byte(hash, level.cooldown_ticks);
        feed_u16(hash, level.projectile_speed_milli);
        match level.upgrade_cost {
            Some(cost) => {
                feed_byte(hash, 1);
                feed_u64(hash, u64::from(cost));
            }
            None => feed_byte(hash, 0),
        }
    }
}

fn feed_bool(hash: &mut u64, value: bool) {
    feed_byte(hash, u8::from(value));
}

fn feed_u64(hash: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        feed_byte(hash, byte);
    }
}

fn feed_i32(hash: &mut u64, value: i32) {
    for byte in value.to_le_bytes() {
        feed_byte(hash, byte);
    }
}

fn feed_u16(hash: &mut u64, value: u16) {
    for byte in value.to_le_bytes() {
        feed_byte(hash, byte);
    }
}

fn feed_byte(hash: &mut u64, byte: u8) {
    *hash ^= u64::from(byte);
    *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
}
