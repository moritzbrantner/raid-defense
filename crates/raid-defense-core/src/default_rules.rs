use crate::rules::{
    BuildingRules, CycleRules, EconomyRules, GameRules, HouseRules, PopulationRules, RaidRules,
    SawmillRules, StorageHouseRules, TowerArchetypeRules, TowerLevelRules, TowerRules,
    TownHallRules,
};

/// The standard Raid Defense balance profile.
///
/// Ordinary gameplay tuning belongs here. Simulation systems should consume
/// `GameRules` and must not grow their own balance constants or formulas.
pub const STANDARD_RULES: GameRules = GameRules {
    economy: EconomyRules {
        starting_wood: 120,
        town_wood_capacity: 500,
        forest_tile_count: 18,
        forest_tile_wood: 80,
        sawmill_harvest_radius: 5,
        sawmill_output: 4,
        sawmill_interval_ticks: 10,
        sawmill_local_wood_capacity: 24,
        storage_house_wood_capacity: 160,
    },
    population: PopulationRules {
        starting_people: 2,
        base_capacity: 2,
        person_health: 20,
        carry_capacity: 8,
        speed_milli: 500,
    },
    buildings: BuildingRules {
        town_hall: TownHallRules { max_health: 250 },
        sawmill: SawmillRules {
            wood_cost: 40,
            max_health: 80,
        },
        storage_house: StorageHouseRules {
            wood_cost: 50,
            max_health: 110,
        },
        house: HouseRules {
            wood_cost: 60,
            max_health: 90,
            unlock_completed_waves: 10,
            population_capacity: 2,
            people_added: 2,
        },
    },
    towers: TowerRules {
        max_level: 3,
        max_health: 100,
        arrow: TowerArchetypeRules {
            build_cost: 25,
            levels: [
                TowerLevelRules {
                    damage: 8,
                    range_milli: 3_200,
                    cooldown_ticks: 3,
                    projectile_speed_milli: 900,
                    upgrade_cost: Some(20),
                },
                TowerLevelRules {
                    damage: 12,
                    range_milli: 3_500,
                    cooldown_ticks: 3,
                    projectile_speed_milli: 1_000,
                    upgrade_cost: Some(30),
                },
                TowerLevelRules {
                    damage: 17,
                    range_milli: 3_800,
                    cooldown_ticks: 2,
                    projectile_speed_milli: 1_100,
                    upgrade_cost: None,
                },
            ],
        },
        cannon: TowerArchetypeRules {
            build_cost: 45,
            levels: [
                TowerLevelRules {
                    damage: 18,
                    range_milli: 4_200,
                    cooldown_ticks: 7,
                    projectile_speed_milli: 500,
                    upgrade_cost: Some(30),
                },
                TowerLevelRules {
                    damage: 27,
                    range_milli: 4_500,
                    cooldown_ticks: 6,
                    projectile_speed_milli: 550,
                    upgrade_cost: Some(45),
                },
                TowerLevelRules {
                    damage: 40,
                    range_milli: 4_800,
                    cooldown_ticks: 5,
                    projectile_speed_milli: 600,
                    upgrade_cost: None,
                },
            ],
        },
    },
    raids: RaidRules {
        raiders_per_wave: 4,
        base_health: 30,
        health_per_wave: 3,
        base_damage: 10,
        damage_increase_every_waves: 3,
        damage_increase_amount: 1,
        speed_milli: 250,
        base_wood_steal: 15,
        wood_steal_per_wave: 2,
    },
    cycle: CycleRules {
        day_length_ticks: 600,
        automatic_raids: true,
        pause_economy_during_raids: true,
    },
};
