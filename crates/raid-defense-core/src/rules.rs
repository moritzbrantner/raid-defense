use crate::{TowerArchetype, TowerStats};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GameRules {
    pub economy: EconomyRules,
    pub population: PopulationRules,
    pub buildings: BuildingRules,
    pub towers: TowerRules,
    pub raids: RaidRules,
}

impl GameRules {
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
