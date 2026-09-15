use crate::{TowerArchetype, TowerStats};

pub const MAX_SCENARIO_WAVES: usize = 64;
pub const MAX_WAVE_GROUPS: usize = 8;

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
    StartingWoodExceedsCapacity,
    HousePopulationExceedsCapacity,
    ZeroForestCount,
    ZeroForestWood,
    ZeroForestRegrowthAmount,
    ZeroForestRegrowthInterval,
    ZeroSawmillInterval,
    ZeroStorageCapacity,
    InvalidTowerLevelCount,
    NegativeTowerRange,
    ZeroRaidersPerWave,
    ZeroRaidSpawnInterval,
    ZeroRaidDamageInterval,
    ZeroRaiderHealth,
    ZeroRaiderDamage,
    ZeroRaiderSpeed,
    ZeroRaiderSteal,
    InvalidWaveCount,
    InvalidWaveGroupCount,
    ZeroWaveGroupRaiders,
    TooManyRaidersInWave,
    ZeroRaidRally,
    ZeroDayLength,
}

impl GameRules {
    pub const fn validate(self) -> Result<(), RulesError> {
        if self.population.starting_people > self.population.base_capacity {
            return Err(RulesError::StartingPopulationExceedsCapacity);
        }
        if self.economy.starting_wood > self.economy.town_wood_capacity {
            return Err(RulesError::StartingWoodExceedsCapacity);
        }
        if self.buildings.house.people_added > self.buildings.house.population_capacity {
            return Err(RulesError::HousePopulationExceedsCapacity);
        }
        if self.economy.forest_tile_count == 0 {
            return Err(RulesError::ZeroForestCount);
        }
        if self.economy.forest_tile_wood == 0 {
            return Err(RulesError::ZeroForestWood);
        }
        if self.economy.forest_regrowth_amount == 0 {
            return Err(RulesError::ZeroForestRegrowthAmount);
        }
        if self.economy.forest_regrowth_interval_ticks == 0 {
            return Err(RulesError::ZeroForestRegrowthInterval);
        }
        if self.economy.sawmill_interval_ticks == 0 {
            return Err(RulesError::ZeroSawmillInterval);
        }
        if self.economy.storage_house_wood_capacity == 0 {
            return Err(RulesError::ZeroStorageCapacity);
        }
        if self.towers.max_level == 0 || self.towers.max_level > 3 {
            return Err(RulesError::InvalidTowerLevelCount);
        }
        if has_negative_active_range(self.towers.arrow, self.towers.max_level)
            || has_negative_active_range(self.towers.cannon, self.towers.max_level)
        {
            return Err(RulesError::NegativeTowerRange);
        }
        if self.raids.raiders_per_wave == 0 {
            return Err(RulesError::ZeroRaidersPerWave);
        }
        if self.raids.spawn_interval_ticks == 0 {
            return Err(RulesError::ZeroRaidSpawnInterval);
        }
        if self.raids.damage_increase_every_waves == 0 {
            return Err(RulesError::ZeroRaidDamageInterval);
        }
        if self.raids.basic.health == 0 || self.raids.advanced.health == 0 {
            return Err(RulesError::ZeroRaiderHealth);
        }
        if self.raids.basic.damage == 0 || self.raids.advanced.damage == 0 {
            return Err(RulesError::ZeroRaiderDamage);
        }
        if self.raids.basic.speed_milli == 0 || self.raids.advanced.speed_milli == 0 {
            return Err(RulesError::ZeroRaiderSpeed);
        }
        if self.raids.basic.wood_steal == 0 || self.raids.advanced.wood_steal == 0 {
            return Err(RulesError::ZeroRaiderSteal);
        }
        if self.raids.wave_plan.wave_count as usize > MAX_SCENARIO_WAVES {
            return Err(RulesError::InvalidWaveCount);
        }
        let mut wave_index = 0_usize;
        while wave_index < self.raids.wave_plan.wave_count as usize {
            let wave = self.raids.wave_plan.waves[wave_index];
            if wave.group_count == 0 || wave.group_count as usize > MAX_WAVE_GROUPS {
                return Err(RulesError::InvalidWaveGroupCount);
            }
            let mut group_index = 0_usize;
            let mut total = 0_u32;
            while group_index < wave.group_count as usize {
                let group = wave.groups[group_index];
                if group.count == 0 {
                    return Err(RulesError::ZeroWaveGroupRaiders);
                }
                total = total.saturating_add(u32::from(group.count));
                group_index += 1;
            }
            if total > u32::from(u16::MAX) {
                return Err(RulesError::TooManyRaidersInWave);
            }
            wave_index += 1;
        }
        if self.cycle.raid_rally_ticks == 0 {
            return Err(RulesError::ZeroRaidRally);
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
        if level == 0 || level >= self.towers.max_level {
            return None;
        }
        self.tower_level(archetype, level).upgrade_cost
    }

    #[must_use]
    pub const fn raider(self, archetype: RaiderArchetype) -> RaiderArchetypeRules {
        match archetype {
            RaiderArchetype::Basic => self.raids.basic,
            RaiderArchetype::Advanced => self.raids.advanced,
        }
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
        feed_u16(&mut hash, self.economy.forest_tile_count);
        feed_u64(&mut hash, u64::from(self.economy.forest_tile_wood));
        feed_u16(&mut hash, self.economy.forest_regrowth_amount);
        feed_u16(&mut hash, self.economy.forest_regrowth_interval_ticks);
        feed_u16(&mut hash, self.economy.sawmill_output);
        feed_u16(&mut hash, self.economy.sawmill_interval_ticks);
        feed_u64(
            &mut hash,
            u64::from(self.economy.sawmill_local_wood_capacity),
        );
        feed_u64(
            &mut hash,
            u64::from(self.economy.storage_house_wood_capacity),
        );

        feed_u16(&mut hash, self.population.starting_people);
        feed_u16(&mut hash, self.population.base_capacity);
        feed_u16(&mut hash, self.population.person_health);
        feed_u16(&mut hash, self.population.carry_capacity);
        feed_u16(&mut hash, self.population.speed_milli);

        feed_u16(&mut hash, self.buildings.town_hall.max_health);
        feed_u64(&mut hash, u64::from(self.buildings.sawmill.wood_cost));
        feed_u16(&mut hash, self.buildings.sawmill.max_health);
        feed_u64(&mut hash, u64::from(self.buildings.storage_house.wood_cost));
        feed_u16(&mut hash, self.buildings.storage_house.max_health);
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
        feed_u16(&mut hash, self.raids.spawn_interval_ticks);
        feed_u16(&mut hash, self.raids.base_health);
        feed_u16(&mut hash, self.raids.health_per_wave);
        feed_u16(&mut hash, self.raids.base_damage);
        feed_u64(&mut hash, u64::from(self.raids.damage_increase_every_waves));
        feed_u16(&mut hash, self.raids.damage_increase_amount);
        feed_u16(&mut hash, self.raids.speed_milli);
        feed_u64(&mut hash, u64::from(self.raids.base_wood_steal));
        feed_u64(&mut hash, u64::from(self.raids.wood_steal_per_wave));
        feed_raider(&mut hash, self.raids.basic);
        feed_raider(&mut hash, self.raids.advanced);
        feed_byte(&mut hash, self.raids.wave_plan.wave_count);
        for wave_index in 0..self.raids.wave_plan.wave_count as usize {
            let wave = self.raids.wave_plan.waves[wave_index];
            feed_byte(&mut hash, wave.group_count);
            for group_index in 0..wave.group_count as usize {
                let group = wave.groups[group_index];
                feed_byte(
                    &mut hash,
                    match group.archetype {
                        RaiderArchetype::Basic => 0,
                        RaiderArchetype::Advanced => 1,
                    },
                );
                feed_u16(&mut hash, group.count);
            }
        }

        feed_u16(&mut hash, self.cycle.day_length_ticks);
        feed_u16(&mut hash, self.cycle.raid_rally_ticks);
        feed_bool(&mut hash, self.cycle.automatic_raids);
        feed_bool(&mut hash, self.cycle.pause_economy_during_raids);

        hash
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EconomyRules {
    pub starting_wood: u32,
    pub town_wood_capacity: u32,
    pub forest_tile_count: u16,
    pub forest_tile_wood: u32,
    pub forest_regrowth_amount: u16,
    pub forest_regrowth_interval_ticks: u16,
    /// Wood a worker can cut in one completed forestry work cycle.
    pub sawmill_output: u16,
    /// Ticks a worker spends gathering a batch of wood at a forest.
    pub sawmill_interval_ticks: u16,
    pub sawmill_local_wood_capacity: u32,
    pub storage_house_wood_capacity: u32,
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
    pub storage_house: StorageHouseRules,
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
pub struct StorageHouseRules {
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
pub enum RaiderArchetype {
    Basic,
    Advanced,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RaiderArchetypeRules {
    pub health: u16,
    pub damage: u16,
    pub speed_milli: u16,
    pub wood_steal: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RaiderCatalogRules {
    pub basic: RaiderArchetypeRules,
    pub advanced: RaiderArchetypeRules,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WaveGroupRules {
    pub archetype: RaiderArchetype,
    pub count: u16,
}

impl WaveGroupRules {
    pub const EMPTY: Self = Self {
        archetype: RaiderArchetype::Basic,
        count: 0,
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WaveRules {
    pub groups: [WaveGroupRules; MAX_WAVE_GROUPS],
    pub group_count: u8,
}

impl WaveRules {
    pub const EMPTY: Self = Self {
        groups: [WaveGroupRules::EMPTY; MAX_WAVE_GROUPS],
        group_count: 0,
    };

    #[must_use]
    pub fn total_raiders(self) -> u16 {
        self.groups[..self.group_count as usize]
            .iter()
            .fold(0_u16, |total, group| total.saturating_add(group.count))
    }

    #[must_use]
    pub fn group(self, index: usize) -> Option<WaveGroupRules> {
        (index < self.group_count as usize).then_some(self.groups[index])
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WavePlanRules {
    pub waves: [WaveRules; MAX_SCENARIO_WAVES],
    pub wave_count: u8,
}

impl WavePlanRules {
    pub const EMPTY: Self = Self {
        waves: [WaveRules::EMPTY; MAX_SCENARIO_WAVES],
        wave_count: 0,
    };

    #[must_use]
    pub const fn is_explicit(self) -> bool {
        self.wave_count != 0
    }

    #[must_use]
    pub fn wave(self, wave_number: u32) -> Option<WaveRules> {
        let index = usize::try_from(wave_number.checked_sub(1)?).ok()?;
        (index < self.wave_count as usize).then_some(self.waves[index])
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RaidRules {
    /// Legacy generated-wave count used when `wave_plan` is empty.
    pub raiders_per_wave: u16,
    pub spawn_interval_ticks: u16,
    /// Legacy generated-wave health baseline used when `wave_plan` is empty.
    pub base_health: u16,
    pub health_per_wave: u16,
    /// Legacy generated-wave damage baseline used when `wave_plan` is empty.
    pub base_damage: u16,
    pub damage_increase_every_waves: u32,
    pub damage_increase_amount: u16,
    /// Legacy generated-wave movement speed used when `wave_plan` is empty.
    pub speed_milli: u16,
    /// Legacy generated-wave theft used when `wave_plan` is empty.
    pub base_wood_steal: u32,
    pub wood_steal_per_wave: u32,
    pub basic: RaiderArchetypeRules,
    pub advanced: RaiderArchetypeRules,
    pub wave_plan: WavePlanRules,
}

impl RaidRules {
    #[must_use]
    pub const fn catalog(self) -> RaiderCatalogRules {
        RaiderCatalogRules {
            basic: self.basic,
            advanced: self.advanced,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CycleRules {
    pub day_length_ticks: u16,
    pub raid_rally_ticks: u16,
    pub automatic_raids: bool,
    pub pause_economy_during_raids: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartingSupplies {
    Lean,
    Standard,
    Rich,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForestDensity {
    Sparse,
    Standard,
    Dense,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForestRegrowth {
    Slow,
    Standard,
    Fast,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SawmillThroughput {
    Slow,
    Standard,
    Fast,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RaidSize {
    Small,
    Standard,
    Large,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RaiderStrength {
    Gentle,
    Standard,
    Harsh,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DayLength {
    Short,
    Standard,
    Long,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RaidTiming {
    Standard,
    Manual,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RaidEconomy {
    Standard,
    Continuous,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScenarioOptions {
    pub starting_supplies: StartingSupplies,
    pub forest_density: ForestDensity,
    pub forest_regrowth: ForestRegrowth,
    pub sawmill_throughput: SawmillThroughput,
    /// Compatibility modifier for generated waves. Explicit wave plans do not use it.
    pub raid_size: RaidSize,
    /// Compatibility modifier for generated waves. Explicit raid units do not use it.
    pub raider_strength: RaiderStrength,
    pub day_length: DayLength,
    pub raid_timing: RaidTiming,
    pub raid_economy: RaidEconomy,
    pub raiders: RaiderCatalogRules,
    pub wave_plan: WavePlanRules,
    pub towers: TowerRules,
}

impl ScenarioOptions {
    #[must_use]
    pub const fn standard() -> Self {
        Self {
            starting_supplies: StartingSupplies::Standard,
            forest_density: ForestDensity::Standard,
            forest_regrowth: ForestRegrowth::Standard,
            sawmill_throughput: SawmillThroughput::Standard,
            raid_size: RaidSize::Standard,
            raider_strength: RaiderStrength::Standard,
            day_length: DayLength::Standard,
            raid_timing: RaidTiming::Standard,
            raid_economy: RaidEconomy::Standard,
            raiders: crate::STANDARD_RULES.raids.catalog(),
            wave_plan: crate::STANDARD_RULES.raids.wave_plan,
            towers: crate::STANDARD_RULES.towers,
        }
    }

    pub fn into_rules(self) -> Result<GameRules, RulesError> {
        let mut rules = crate::STANDARD_RULES;

        match self.starting_supplies {
            StartingSupplies::Lean => rules.economy.starting_wood /= 2,
            StartingSupplies::Standard => {}
            StartingSupplies::Rich => {
                rules.economy.starting_wood = rules
                    .economy
                    .starting_wood
                    .saturating_mul(2)
                    .min(rules.economy.town_wood_capacity);
            }
        }

        match self.forest_density {
            ForestDensity::Sparse => {
                rules.economy.forest_tile_count = half_nonzero(rules.economy.forest_tile_count);
            }
            ForestDensity::Standard => {}
            ForestDensity::Dense => {
                rules.economy.forest_tile_count = increase_half(rules.economy.forest_tile_count);
            }
        }

        match self.forest_regrowth {
            ForestRegrowth::Slow => {
                rules.economy.forest_regrowth_interval_ticks = rules
                    .economy
                    .forest_regrowth_interval_ticks
                    .saturating_mul(2);
            }
            ForestRegrowth::Standard => {}
            ForestRegrowth::Fast => {
                rules.economy.forest_regrowth_interval_ticks =
                    half_nonzero(rules.economy.forest_regrowth_interval_ticks);
            }
        }

        match self.sawmill_throughput {
            SawmillThroughput::Slow => {
                rules.economy.sawmill_output = half_nonzero(rules.economy.sawmill_output);
            }
            SawmillThroughput::Standard => {}
            SawmillThroughput::Fast => {
                rules.economy.sawmill_output = rules.economy.sawmill_output.saturating_mul(2);
            }
        }

        match self.raid_size {
            RaidSize::Small => {
                rules.raids.raiders_per_wave = half_nonzero(rules.raids.raiders_per_wave);
            }
            RaidSize::Standard => {}
            RaidSize::Large => {
                rules.raids.raiders_per_wave = rules.raids.raiders_per_wave.saturating_mul(2);
            }
        }

        match self.raider_strength {
            RaiderStrength::Gentle => {
                rules.raids.base_health = decrease_third(rules.raids.base_health);
                rules.raids.health_per_wave = decrease_third(rules.raids.health_per_wave);
                rules.raids.base_damage = decrease_third(rules.raids.base_damage);
                rules.raids.damage_increase_amount =
                    decrease_third(rules.raids.damage_increase_amount);
            }
            RaiderStrength::Standard => {}
            RaiderStrength::Harsh => {
                rules.raids.base_health = increase_half(rules.raids.base_health);
                rules.raids.health_per_wave = increase_half(rules.raids.health_per_wave);
                rules.raids.base_damage = increase_half(rules.raids.base_damage);
                rules.raids.damage_increase_amount =
                    increase_half(rules.raids.damage_increase_amount);
            }
        }

        match self.day_length {
            DayLength::Short => {
                rules.cycle.day_length_ticks = half_nonzero(rules.cycle.day_length_ticks);
            }
            DayLength::Standard => {}
            DayLength::Long => {
                rules.cycle.day_length_ticks = rules.cycle.day_length_ticks.saturating_mul(2);
            }
        }

        if self.raid_timing == RaidTiming::Manual {
            rules.cycle.automatic_raids = false;
        }
        if self.raid_economy == RaidEconomy::Continuous {
            rules.cycle.pause_economy_during_raids = false;
        }

        rules.raids.basic = self.raiders.basic;
        rules.raids.advanced = self.raiders.advanced;
        rules.raids.wave_plan = self.wave_plan;
        rules.towers = self.towers;

        rules.validate()?;
        Ok(rules)
    }
}

impl Default for ScenarioOptions {
    fn default() -> Self {
        Self::standard()
    }
}

fn half_nonzero(value: u16) -> u16 {
    (value / 2).max(1)
}

fn increase_half(value: u16) -> u16 {
    value.saturating_add((value / 2).max(1))
}

fn decrease_third(value: u16) -> u16 {
    u16::try_from((u32::from(value) * 2 / 3).max(1)).unwrap_or(u16::MAX)
}

const fn has_negative_active_range(tower: TowerArchetypeRules, max_level: u8) -> bool {
    let mut index = 0_usize;
    while index < max_level as usize {
        if tower.levels[index].range_milli < 0 {
            return true;
        }
        index += 1;
    }
    false
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

fn feed_raider(hash: &mut u64, raider: RaiderArchetypeRules) {
    feed_u16(hash, raider.health);
    feed_u16(hash, raider.damage);
    feed_u16(hash, raider.speed_milli);
    feed_u64(hash, u64::from(raider.wood_steal));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::STANDARD_RULES;

    #[test]
    fn rejects_starting_wood_above_town_capacity() {
        let mut rules = STANDARD_RULES;
        rules.economy.starting_wood = rules.economy.town_wood_capacity + 1;
        assert_eq!(
            rules.validate(),
            Err(RulesError::StartingWoodExceedsCapacity)
        );
    }

    #[test]
    fn rejects_house_population_growth_above_capacity_growth() {
        let mut rules = STANDARD_RULES;
        rules.buildings.house.people_added = rules.buildings.house.population_capacity + 1;
        assert_eq!(
            rules.validate(),
            Err(RulesError::HousePopulationExceedsCapacity)
        );
    }

    #[test]
    fn rejects_negative_active_tower_range() {
        let mut rules = STANDARD_RULES;
        rules.towers.arrow.levels[0].range_milli = -1;
        assert_eq!(rules.validate(), Err(RulesError::NegativeTowerRange));
    }

    #[test]
    fn rejects_zero_raid_spawn_interval() {
        let mut rules = STANDARD_RULES;
        rules.raids.spawn_interval_ticks = 0;
        assert_eq!(rules.validate(), Err(RulesError::ZeroRaidSpawnInterval));
    }

    #[test]
    fn rejects_zero_raid_rally() {
        let mut rules = STANDARD_RULES;
        rules.cycle.raid_rally_ticks = 0;
        assert_eq!(rules.validate(), Err(RulesError::ZeroRaidRally));
    }

    #[test]
    fn resource_rules_participate_in_rule_identity() {
        let standard = STANDARD_RULES.fingerprint();
        let mut changed = STANDARD_RULES;
        changed.economy.forest_regrowth_interval_ticks += 1;
        assert_ne!(standard, changed.fingerprint());
    }

    #[test]
    fn raid_spawn_interval_participates_in_rule_identity() {
        let standard = STANDARD_RULES.fingerprint();
        let mut changed = STANDARD_RULES;
        changed.raids.spawn_interval_ticks += 1;
        assert_ne!(standard, changed.fingerprint());
    }

    #[test]
    fn raid_rally_participates_in_rule_identity() {
        let standard = STANDARD_RULES.fingerprint();
        let mut changed = STANDARD_RULES;
        changed.cycle.raid_rally_ticks += 1;
        assert_ne!(standard, changed.fingerprint());
    }

    #[test]
    fn damage_increment_participates_in_rule_identity() {
        let standard = STANDARD_RULES.fingerprint();
        let mut changed = STANDARD_RULES;
        changed.raids.damage_increase_amount += 1;
        assert_ne!(standard, changed.fingerprint());
    }

    #[test]
    fn explicit_wave_composition_participates_in_rule_identity() {
        let standard = STANDARD_RULES.fingerprint();
        let mut changed = STANDARD_RULES;
        changed.raids.wave_plan.wave_count = 1;
        changed.raids.wave_plan.waves[0].group_count = 2;
        changed.raids.wave_plan.waves[0].groups[0] = WaveGroupRules {
            archetype: RaiderArchetype::Basic,
            count: 2,
        };
        changed.raids.wave_plan.waves[0].groups[1] = WaveGroupRules {
            archetype: RaiderArchetype::Advanced,
            count: 1,
        };
        assert_ne!(standard, changed.fingerprint());
        assert_eq!(changed.validate(), Ok(()));
    }

    #[test]
    fn rejects_empty_explicit_wave() {
        let mut rules = STANDARD_RULES;
        rules.raids.wave_plan.wave_count = 1;
        assert_eq!(rules.validate(), Err(RulesError::InvalidWaveGroupCount));
    }

    #[test]
    fn standard_scenario_is_exact_standard_rules() {
        assert_eq!(
            ScenarioOptions::standard()
                .into_rules()
                .expect("standard scenario must validate"),
            STANDARD_RULES
        );
    }

    #[test]
    fn scenario_modifiers_change_authoritative_rules_and_identity() {
        let scenario = ScenarioOptions {
            starting_supplies: StartingSupplies::Rich,
            forest_density: ForestDensity::Dense,
            forest_regrowth: ForestRegrowth::Fast,
            sawmill_throughput: SawmillThroughput::Fast,
            raid_size: RaidSize::Large,
            raider_strength: RaiderStrength::Harsh,
            day_length: DayLength::Long,
            raid_timing: RaidTiming::Manual,
            raid_economy: RaidEconomy::Continuous,
            ..ScenarioOptions::standard()
        };
        let rules = scenario.into_rules().expect("scenario must validate");

        assert!(rules.economy.starting_wood > STANDARD_RULES.economy.starting_wood);
        assert!(rules.economy.forest_tile_count > STANDARD_RULES.economy.forest_tile_count);
        assert!(
            rules.economy.forest_regrowth_interval_ticks
                < STANDARD_RULES.economy.forest_regrowth_interval_ticks
        );
        assert!(rules.economy.sawmill_output > STANDARD_RULES.economy.sawmill_output);
        assert!(rules.raids.raiders_per_wave > STANDARD_RULES.raids.raiders_per_wave);
        assert!(rules.raids.base_health > STANDARD_RULES.raids.base_health);
        assert!(rules.cycle.day_length_ticks > STANDARD_RULES.cycle.day_length_ticks);
        assert!(!rules.cycle.automatic_raids);
        assert!(!rules.cycle.pause_economy_during_raids);
        assert_ne!(rules.fingerprint(), STANDARD_RULES.fingerprint());
    }
}