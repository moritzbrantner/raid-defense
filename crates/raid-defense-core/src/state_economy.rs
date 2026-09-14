use super::*;

impl GameState {
    pub(super) fn place_tower(
        &mut self,
        cell: Cell,
        archetype: TowerArchetype,
    ) -> Result<Event, GameError> {
        self.validate_build_cell(cell)?;
        let cost = self.rules.tower(archetype).build_cost;
        self.require_wood(cost)?;
        self.validate_blocking_build(cell, None)?;

        let entity = self.allocate_entity();
        self.transforms
            .insert(entity_key(entity), Transform::at_cell(cell));
        self.health.insert(
            entity_key(entity),
            Health {
                current: self.rules.towers.max_health,
                maximum: self.rules.towers.max_health,
            },
        );
        self.buildings.insert(
            entity_key(entity),
            Building {
                kind: BuildingKind::Tower,
                cell,
            },
        );
        self.towers.insert(
            entity_key(entity),
            Tower {
                archetype,
                level: 0,
            },
        );
        self.storage.insert(
            entity_key(entity),
            ResourceStorage {
                wood: 0,
                wood_capacity: cost,
            },
        );

        Ok(Event::TowerConstructionStarted {
            entity,
            cell,
            archetype,
            wood_required: cost,
        })
    }

    pub(super) fn place_sawmill(&mut self, cell: Cell) -> Result<Event, GameError> {
        self.validate_build_cell(cell)?;
        let building_rules = self.rules.buildings.sawmill;
        let cost = building_rules.wood_cost;
        self.require_wood(cost)?;
        self.validate_blocking_build(cell, Some(cell))?;

        let entity = self.allocate_entity();
        self.transforms
            .insert(entity_key(entity), Transform::at_cell(cell));
        self.health.insert(
            entity_key(entity),
            Health {
                current: building_rules.max_health,
                maximum: building_rules.max_health,
            },
        );
        self.buildings.insert(
            entity_key(entity),
            Building {
                kind: BuildingKind::Sawmill,
                cell,
            },
        );
        self.producers.insert(
            entity_key(entity),
            ResourceProducer {
                resource: ResourceKind::Wood,
                amount: self.rules.economy.sawmill_output,
                interval_ticks: self.rules.economy.sawmill_interval_ticks,
                progress_ticks: 0,
            },
        );
        self.storage.insert(
            entity_key(entity),
            ResourceStorage {
                wood: 0,
                wood_capacity: self.rules.economy.sawmill_local_wood_capacity,
            },
        );
        self.spend_wood(cost);

        Ok(Event::SawmillBuilt {
            entity,
            cell,
            wood_cost: cost,
        })
    }

    pub(super) fn place_storage_house(&mut self, cell: Cell) -> Result<Event, GameError> {
        self.validate_build_cell(cell)?;
        let building_rules = self.rules.buildings.storage_house;
        let cost = building_rules.wood_cost;
        self.require_wood(cost)?;
        self.validate_blocking_build(cell, None)?;

        let entity = self.allocate_entity();
        self.transforms
            .insert(entity_key(entity), Transform::at_cell(cell));
        self.health.insert(
            entity_key(entity),
            Health {
                current: building_rules.max_health,
                maximum: building_rules.max_health,
            },
        );
        self.buildings.insert(
            entity_key(entity),
            Building {
                kind: BuildingKind::StorageHouse,
                cell,
            },
        );
        self.storage.insert(
            entity_key(entity),
            ResourceStorage {
                wood: 0,
                wood_capacity: self.rules.economy.storage_house_wood_capacity,
            },
        );
        self.spend_wood(cost);

        Ok(Event::StorageHouseBuilt {
            entity,
            cell,
            wood_cost: cost,
            wood_capacity: self.rules.economy.storage_house_wood_capacity,
        })
    }

    pub(super) fn place_house(&mut self, cell: Cell) -> Result<Event, GameError> {
        if !self.houses_unlocked() {
            return Err(GameError::HouseLocked);
        }
        self.validate_build_cell(cell)?;
        let house_rules = self.rules.buildings.house;
        let cost = house_rules.wood_cost;
        self.require_wood(cost)?;
        self.validate_blocking_build(cell, None)?;

        let entity = self.allocate_entity();
        self.transforms
            .insert(entity_key(entity), Transform::at_cell(cell));
        self.health.insert(
            entity_key(entity),
            Health {
                current: house_rules.max_health,
                maximum: house_rules.max_health,
            },
        );
        self.buildings.insert(
            entity_key(entity),
            Building {
                kind: BuildingKind::House,
                cell,
            },
        );
        self.housing.insert(
            entity_key(entity),
            Housing {
                capacity: house_rules.population_capacity,
            },
        );
        self.spend_wood(cost);
        for _ in 0..house_rules.people_added {
            self.spawn_person();
        }

        Ok(Event::HouseBuilt {
            entity,
            cell,
            wood_cost: cost,
            people_added: house_rules.people_added,
            population_capacity: self.population_capacity(),
        })
    }

    pub(super) fn upgrade_tower(&mut self, cell: Cell) -> Result<Event, GameError> {
        if self.town_health() == 0 {
            return Err(GameError::GameOver);
        }
        if !in_bounds(cell) {
            return Err(GameError::OutOfBounds);
        }
        let entity = self.tower_entity_at(cell).ok_or(GameError::NoTower)?;
        let tower = self
            .towers
            .get(entity_key(entity))
            .copied()
            .expect("tower buildings always have a tower component");
        let cost = self
            .rules
            .tower_upgrade_cost(tower.archetype, tower.level)
            .ok_or(GameError::MaxTowerLevel)?;
        self.require_wood(cost)?;

        let next_level = tower.level + 1;
        let cooldown_remaining = self
            .attacks
            .get(entity_key(entity))
            .map_or(0, |attack| attack.cooldown_remaining);
        let next_attack = attack_for(self.rules, tower.archetype, next_level, cooldown_remaining);
        self.towers.insert(
            entity_key(entity),
            Tower {
                archetype: tower.archetype,
                level: next_level,
            },
        );
        self.attacks.insert(entity_key(entity), next_attack);
        self.spend_wood(cost);

        Ok(Event::TowerUpgraded {
            entity,
            archetype: tower.archetype,
            level: next_level,
            wood_cost: cost,
        })
    }

    pub(super) fn validate_build_cell(&self, cell: Cell) -> Result<(), GameError> {
        if self.town_health() == 0 {
            return Err(GameError::GameOver);
        }
        if !in_bounds(cell) {
            return Err(GameError::OutOfBounds);
        }
        if is_town_cell(cell) || Edge::ALL.into_iter().any(|edge| edge.spawn_cell() == cell) {
            return Err(GameError::ProtectedCell);
        }
        if self.cell_has_building(cell) || self.unit_uses_cell(cell) {
            return Err(GameError::CellOccupied);
        }
        Ok(())
    }

    pub(super) fn validate_blocking_build(
        &self,
        cell: Cell,
        hypothetical_sawmill: Option<Cell>,
    ) -> Result<(), GameError> {
        if !self.routes_remain_open(Some(cell))
            || !self.all_sawmills_accessible(Some(cell), hypothetical_sawmill)
            || !self.worker_routes_remain_open(Some(cell))
        {
            return Err(GameError::PathBlocked);
        }
        Ok(())
    }

    pub(super) fn require_wood(&self, amount: u32) -> Result<(), GameError> {
        if self.wood() < amount {
            Err(GameError::InsufficientWood)
        } else {
            Ok(())
        }
    }

    pub(super) fn spend_wood(&mut self, amount: u32) {
        self.spend_settlement_wood(amount);
    }

    pub(super) fn start_wave(&mut self) -> Result<Event, GameError> {
        if self.town_health() == 0 {
            return Err(GameError::GameOver);
        }
        if self.is_night() {
            return Err(GameError::RaidersStillActive);
        }

        self.day_ticks_remaining = 0;
        self.wave = self.wave.saturating_add(1);
        self.wave_schedule = WaveSchedule {
            remaining_raiders: self.rules.raids.raiders_per_wave,
            spawn_ticks_remaining: 0,
        };
        self.spawn_next_scheduled_raider();

        Ok(Event::WaveStarted {
            wave: self.wave,
            raiders: self.rules.raids.raiders_per_wave,
        })
    }

    fn spawn_next_scheduled_raider(&mut self) {
        if self.wave_schedule.remaining_raiders == 0 {
            self.wave_schedule.spawn_ticks_remaining = 0;
            return;
        }

        let spawned = self
            .rules
            .raids
            .raiders_per_wave
            .saturating_sub(self.wave_schedule.remaining_raiders);
        let offset = (mix64(self.seed ^ u64::from(self.wave)) % 4) as usize;
        let edge = Edge::ALL[(usize::from(spawned) + offset) % Edge::ALL.len()];
        self.spawn_raider(edge);
        self.wave_schedule.remaining_raiders -= 1;
        self.wave_schedule.spawn_ticks_remaining = if self.wave_schedule.remaining_raiders == 0 {
            0
        } else {
            self.rules.raids.spawn_interval_ticks
        };
    }

    fn advance_wave_spawn_schedule(&mut self) {
        if self.wave_schedule.remaining_raiders == 0 {
            return;
        }

        self.wave_schedule.spawn_ticks_remaining =
            self.wave_schedule.spawn_ticks_remaining.saturating_sub(1);
        if self.wave_schedule.spawn_ticks_remaining == 0 {
            self.spawn_next_scheduled_raider();
        }
    }

    pub(super) fn advance_tick(&mut self) -> Event {
        self.tick = self.tick.saturating_add(1);
        self.run_forest_regrowth_system();
        let had_active_wave = self.is_night();
        let economy_paused = had_active_wave && self.rules.cycle.pause_economy_during_raids;
        let (wood_produced, wood_picked_up, wood_delivered, towers_completed) = if economy_paused {
            (0, 0, 0, 0)
        } else {
            let wood_produced = self.run_resource_production_system();
            let (wood_picked_up, wood_delivered, towers_completed) =
                self.run_person_logistics_system();
            (
                wood_produced,
                wood_picked_up,
                wood_delivered,
                towers_completed,
            )
        };

        let mut wave_active_this_tick = had_active_wave;
        if !had_active_wave && self.rules.cycle.automatic_raids {
            self.day_ticks_remaining = self.day_ticks_remaining.saturating_sub(1);
            if self.day_ticks_remaining == 0 && self.town_health() > 0 {
                self.start_wave()
                    .expect("an expired peaceful day can always begin its next wave");
                wave_active_this_tick = true;
            }
        } else if had_active_wave {
            self.advance_wave_spawn_schedule();
        }

        let shots = self.run_tower_attack_system();
        let (impacts, killed) = self.run_projectile_system();
        let kills = u16::try_from(killed.len()).unwrap_or(u16::MAX);
        for entity in killed {
            self.despawn_raider(entity);
        }
        let (wood_stolen, town_damage) = self.run_raider_movement_system();
        let completed_wave =
            if wave_active_this_tick && !self.is_night() && self.completed_waves < self.wave {
                self.completed_waves = self.wave;
                self.day_ticks_remaining = self.rules.cycle.day_length_ticks;
                Some(self.wave)
            } else {
                None
            };

        Event::TickAdvanced {
            tick: self.tick,
            shots,
            impacts,
            kills,
            wood_produced,
            wood_picked_up,
            wood_delivered,
            towers_completed,
            wood_stolen,
            town_damage,
            completed_wave,
        }
    }
}
