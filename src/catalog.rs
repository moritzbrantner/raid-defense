use super::*;

pub fn raid_defense_catalog() -> Catalog {
    let mut catalog = Catalog::new();

    for (id, name) in [
        (CROWNS, "Crowns"),
        (GRAIN, "Food"),
        (TIMBER, "Wood"),
        (STONE, "Stone"),
        (IRON, "Iron"),
        (INFLUENCE, "Influence"),
        (LEGION_STRENGTH, "Legion Strength"),
        (STABILITY, "Stability"),
        (INTELLIGENCE, "Intelligence"),
        (TRADE_GOODS, "Trade Goods"),
        (CITIZENS, "Citizens"),
    ] {
        catalog.add_resource(ResourceDefinition::new(id, name));
    }

    catalog.add_level(
        LevelDefinition::new(2, 180).with_effects(vec![Effect::UnlockUnit(ENVOY.into())]),
    );
    catalog.add_level(
        LevelDefinition::new(3, 460).with_effects(vec![Effect::UnlockBuilding(EMBASSY.into())]),
    );

    catalog.add_tile(TileDefinition::new(PLAINS, "Plains"));
    catalog.add_tile(TileDefinition::new(FOREST, "Forest"));
    catalog.add_tile(TileDefinition::new(HILLS, "Hills"));
    catalog.add_tile(
        TileDefinition::new(WATER_TILE, "Water")
            .with_buildable(false)
            .with_walkable(false),
    );

    catalog.add_path(PathDefinition::new(
        IMPERIAL_ROAD,
        "Imperial Road",
        vec![ResourceAmount::new(STONE, 1)],
    ));
    catalog.add_area(AreaDefinition::new(HEARTLAND, "Heartland", Vec::new()));
    catalog.add_area(AreaDefinition::new(BORDERLAND, "Borderland", Vec::new()));
    catalog.add_area(AreaDefinition::new(
        RESOURCE_FRONTIER,
        "Resource Frontier",
        Vec::new(),
    ));

    for (kind, name, cost) in [
        (PREFECT, "Prefect", vec![ResourceAmount::new(CROWNS, 28)]),
        (ENGINEER, "Worker", vec![ResourceAmount::new(GRAIN, 50)]),
        (
            LEGATE,
            "Legate",
            vec![
                ResourceAmount::new(CROWNS, 35),
                ResourceAmount::new(LEGION_STRENGTH, 3),
            ],
        ),
        (
            ENVOY,
            "Envoy",
            vec![
                ResourceAmount::new(CROWNS, 34),
                ResourceAmount::new(INFLUENCE, 6),
            ],
        ),
        (BASIC_RAIDER, "Raider", Vec::new()),
    ] {
        catalog.add_unit(UnitDefinition::new(kind, name, cost));
    }

    catalog.add_npc(NpcDefinition::new(PROVINCE, "Province"));
    catalog.add_npc(NpcDefinition::new(RIVAL_HOUSE, "Rival House"));
    catalog.add_npc(NpcDefinition::new(CARAVAN, "Caravan"));

    add_core_buildings(&mut catalog);
    add_extraction_buildings(&mut catalog);
    add_civic_and_military_buildings(&mut catalog);
    add_tech_and_upgrades(&mut catalog);

    catalog
}

fn add_core_buildings(catalog: &mut Catalog) {
    catalog.add_building(
        BuildingDefinition::new(CAPITAL, "Castle").with_level(
            BuildingLevelDefinition::new(1, 0, Vec::new())
                .with_height(4)
                .with_requirements(vec![Requirement::Not(Box::new(Requirement::HasBuilding {
                    kind: CAPITAL.into(),
                    min_level: 1,
                    count: 1,
                }))]),
        ),
    );

    catalog.add_building(resource_building(
        FARMSTEAD,
        "Farm",
        8,
        vec![ResourceAmount::new(TIMBER, 50)],
        GRAIN,
    ));
}

fn add_extraction_buildings(catalog: &mut Catalog) {
    catalog.add_building(resource_building(
        LUMBER_CAMP,
        "Lumber Camp",
        8,
        vec![ResourceAmount::new(TIMBER, 50)],
        TIMBER,
    ));
    catalog.add_building(resource_building(
        QUARRY,
        "Quarry",
        8,
        vec![ResourceAmount::new(TIMBER, 50)],
        STONE,
    ));
    catalog.add_building(
        frontier_building(
            IRON_MINE,
            "Iron Mine",
            18,
            vec![
                ResourceAmount::new(CROWNS, 50),
                ResourceAmount::new(TIMBER, 18),
                ResourceAmount::new(STONE, 12),
            ],
            ProductionRule::new(
                18,
                vec![ResourceAmount::new(CITIZENS, 5)],
                vec![ResourceAmount::new(IRON, 22)],
            ),
        )
        .with_requirements(vec![Requirement::HasTechNode(MILITARY_DRILL.into())]),
    );
}

fn add_civic_and_military_buildings(catalog: &mut Catalog) {
    catalog.add_building(
        BuildingDefinition::new(MARKET, "Storage House")
            .with_placement_rules(heartland_rules())
            .with_level(
                BuildingLevelDefinition::new(
                    1,
                    8,
                    vec![
                        ResourceAmount::new(TIMBER, 25),
                        ResourceAmount::new(STONE, 20),
                    ],
                )
                .with_height(2)
                .with_storage_bonus(vec![
                    ResourceAmount::new(TIMBER, 150),
                    ResourceAmount::new(STONE, 150),
                    ResourceAmount::new(GRAIN, 150),
                ]),
            )
            .with_level(
                BuildingLevelDefinition::new(
                    2,
                    8,
                    vec![
                        ResourceAmount::new(TIMBER, 25),
                        ResourceAmount::new(STONE, 20),
                    ],
                )
                .with_height(3)
                .with_storage_bonus(vec![
                    ResourceAmount::new(TIMBER, 150),
                    ResourceAmount::new(STONE, 150),
                    ResourceAmount::new(GRAIN, 150),
                ]),
            ),
    );
    catalog.add_building(economic_building(
        SENATE_HALL,
        "Senate Hall",
        18,
        vec![
            ResourceAmount::new(CROWNS, 70),
            ResourceAmount::new(STONE, 28),
        ],
        ProductionRule::new(
            18,
            vec![ResourceAmount::new(CROWNS, 30)],
            vec![
                ResourceAmount::new(INFLUENCE, 16),
                ResourceAmount::new(STABILITY, 4),
            ],
        )
        .with_worker_requirement(1, vec![PREFECT.into()]),
    ));
    catalog.add_building(economic_building(
        BARRACKS,
        "Barracks",
        14,
        vec![
            ResourceAmount::new(CROWNS, 48),
            ResourceAmount::new(TIMBER, 18),
            ResourceAmount::new(STONE, 10),
        ],
        ProductionRule::new(
            16,
            vec![ResourceAmount::new(GRAIN, 12), ResourceAmount::new(IRON, 5)],
            vec![ResourceAmount::new(LEGION_STRENGTH, 16)],
        )
        .with_worker_requirement(1, vec![LEGATE.into()]),
    ));
    catalog.add_building(
        BuildingDefinition::new(WATCHTOWER, "Tower")
            .with_placement_rules(border_rules())
            .with_level(
                BuildingLevelDefinition::new(
                    1,
                    10,
                    vec![
                        ResourceAmount::new(TIMBER, 50),
                        ResourceAmount::new(STONE, 50),
                    ],
                )
                .with_height(3)
                .with_footprint(BuildingFootprint::new(2, 2))
                .with_worker_requirement(1, vec![ENGINEER.into()]),
            ),
    );
    catalog.add_building(
        border_building(
            FRONTIER_FORT,
            "Frontier Fort",
            24,
            vec![
                ResourceAmount::new(CROWNS, 86),
                ResourceAmount::new(STONE, 34),
                ResourceAmount::new(TIMBER, 20),
            ],
            ProductionRule::new(
                20,
                vec![ResourceAmount::new(GRAIN, 10)],
                vec![
                    ResourceAmount::new(INTELLIGENCE, 10),
                    ResourceAmount::new(STABILITY, 3),
                ],
            )
            .with_worker_requirement(1, vec![LEGATE.into()]),
        )
        .with_level(
            BuildingLevelDefinition::new(
                2,
                36,
                vec![
                    ResourceAmount::new(CROWNS, 100),
                    ResourceAmount::new(STONE, 42),
                ],
            )
            .with_height(4)
            .with_inventory_capacity(vec![ResourceAmount::new(GRAIN, 120)])
            .with_production_queue(ProductionQueueConfig::new(3))
            .with_production(
                ProductionRule::new(
                    16,
                    vec![ResourceAmount::new(GRAIN, 12)],
                    vec![
                        ResourceAmount::new(INTELLIGENCE, 14),
                        ResourceAmount::new(STABILITY, 5),
                    ],
                )
                .with_worker_requirement(1, vec![LEGATE.into()]),
            ),
        ),
    );
    catalog.add_building(
        border_building(
            EMBASSY,
            "Embassy",
            22,
            vec![
                ResourceAmount::new(CROWNS, 90),
                ResourceAmount::new(STONE, 24),
                ResourceAmount::new(INFLUENCE, 10),
            ],
            ProductionRule::new(
                18,
                vec![ResourceAmount::new(CROWNS, 20)],
                vec![
                    ResourceAmount::new(INFLUENCE, 14),
                    ResourceAmount::new(STABILITY, 2),
                ],
            )
            .with_worker_requirement(1, vec![ENVOY.into()]),
        )
        .with_requirements(vec![Requirement::PlayerLevelAtLeast(3)]),
    );
}

fn add_tech_and_upgrades(catalog: &mut Catalog) {
    catalog.add_tech_node(
        TechNodeDefinition::new(IMPERIAL_CHARTER, "Imperial Charter", Vec::new()).with_grants(
            vec![
                Effect::UnlockBuilding(MARKET.into()),
                Effect::UnlockBuilding(SENATE_HALL.into()),
                Effect::UnlockBuilding(WATCHTOWER.into()),
                Effect::UnlockBuilding(FRONTIER_FORT.into()),
            ],
        ),
    );
    catalog.add_tech_node(
        TechNodeDefinition::new(
            CURRENCY_REFORM,
            "Currency Reform",
            vec![ResourceAmount::new(INFLUENCE, 18)],
        )
        .with_requirements(vec![Requirement::HasTechNode(IMPERIAL_CHARTER.into())])
        .with_grants(vec![Effect::UnlockUpgrade(STANDARDIZED_WEIGHTS.into())]),
    );
    catalog.add_tech_node(
        TechNodeDefinition::new(
            ROAD_NETWORKS,
            "Road Networks",
            vec![
                ResourceAmount::new(INFLUENCE, 20),
                ResourceAmount::new(STONE, 20),
            ],
        )
        .with_requirements(vec![Requirement::HasTechNode(IMPERIAL_CHARTER.into())])
        .with_grants(vec![Effect::UnlockUpgrade(SURVEYORS.into())]),
    );
    catalog.add_tech_node(
        TechNodeDefinition::new(
            MILITARY_DRILL,
            "Military Drill",
            vec![
                ResourceAmount::new(INFLUENCE, 24),
                ResourceAmount::new(IRON, 18),
            ],
        )
        .with_requirements(vec![Requirement::HasTechNode(IMPERIAL_CHARTER.into())])
        .with_grants(vec![
            Effect::UnlockBuilding(IRON_MINE.into()),
            Effect::UnlockUpgrade(PROFESSIONAL_LEGIONS.into()),
        ]),
    );
    catalog.add_tech_node(
        TechNodeDefinition::new(
            CIVIC_SERVICE,
            "Civic Service",
            vec![
                ResourceAmount::new(INFLUENCE, 28),
                ResourceAmount::new(STABILITY, 18),
            ],
        )
        .with_requirements(vec![Requirement::HasTechNode(CURRENCY_REFORM.into())])
        .with_grants(vec![Effect::UnlockUpgrade(CODEX_ADMINISTRATION.into())]),
    );
    catalog.add_tech_node(
        TechNodeDefinition::new(
            BORDER_DIPLOMACY,
            "Border Diplomacy",
            vec![
                ResourceAmount::new(INFLUENCE, 36),
                ResourceAmount::new(INTELLIGENCE, 18),
            ],
        )
        .with_requirements(vec![Requirement::HasTechNode(ROAD_NETWORKS.into())])
        .with_grants(vec![Effect::UnlockBuilding(EMBASSY.into())]),
    );

    catalog.add_upgrade(
        UpgradeDefinition::new(
            STANDARDIZED_WEIGHTS,
            "Standardized Weights",
            vec![ResourceAmount::new(INFLUENCE, 20)],
        )
        .with_effects(vec![Effect::MultiplyProductionOutput {
            resource: CROWNS.into(),
            numerator: 6,
            denominator: 5,
        }]),
    );
    catalog.add_upgrade(
        UpgradeDefinition::new(
            SURVEYORS,
            "Surveyors",
            vec![ResourceAmount::new(INFLUENCE, 18)],
        )
        .with_effects(vec![Effect::MultiplyBuildDuration {
            numerator: 4,
            denominator: 5,
        }]),
    );
    catalog.add_upgrade(
        UpgradeDefinition::new(
            PROFESSIONAL_LEGIONS,
            "Professional Legions",
            vec![ResourceAmount::new(INFLUENCE, 26)],
        )
        .with_effects(vec![Effect::MultiplyProductionOutput {
            resource: LEGION_STRENGTH.into(),
            numerator: 5,
            denominator: 4,
        }]),
    );
    catalog.add_upgrade(
        UpgradeDefinition::new(
            GRAIN_DOLES,
            "Grain Doles",
            vec![
                ResourceAmount::new(INFLUENCE, 22),
                ResourceAmount::new(GRAIN, 40),
            ],
        )
        .with_effects(vec![Effect::AddStorageCapacity(ResourceAmount::new(
            STABILITY, 40,
        ))]),
    );
    catalog.add_upgrade(
        UpgradeDefinition::new(
            CODEX_ADMINISTRATION,
            "Codex Administration",
            vec![ResourceAmount::new(INFLUENCE, 34)],
        )
        .with_requirements(vec![Requirement::HasTechNode(CIVIC_SERVICE.into())])
        .with_effects(vec![Effect::MultiplyProductionDuration {
            numerator: 9,
            denominator: 10,
        }]),
    );
}

fn economic_building(
    kind: &str,
    name: &str,
    build_time: u64,
    cost: Vec<ResourceAmount>,
    production: ProductionRule,
) -> BuildingDefinition {
    let level_one_cost = cost.clone();
    BuildingDefinition::new(kind, name)
        .with_placement_rules(heartland_rules())
        .with_level(
            BuildingLevelDefinition::new(1, build_time, level_one_cost)
                .with_height(2)
                .with_inventory_capacity(vec![ResourceAmount::new(GRAIN, 60)])
                .with_production_queue(ProductionQueueConfig::new(3))
                .with_production(production.clone()),
        )
        .with_level(
            BuildingLevelDefinition::new(2, build_time, cost)
                .with_height(3)
                .with_inventory_capacity(vec![ResourceAmount::new(GRAIN, 120)])
                .with_production_queue(ProductionQueueConfig::new(3))
                .with_production(production),
        )
}

fn resource_building(
    kind: &str,
    name: &str,
    build_time: u64,
    cost: Vec<ResourceAmount>,
    output_resource: &str,
) -> BuildingDefinition {
    let mut level = BuildingLevelDefinition::new(1, build_time, cost)
        .with_height(2)
        .with_footprint(BuildingFootprint::new(4, 4))
        .with_production(
            ProductionRule::new(
                if kind == FARMSTEAD { 60 } else { 300 },
                Vec::new(),
                vec![ResourceAmount::new(
                    output_resource,
                    if kind == FARMSTEAD { 8 } else { 5 },
                )],
            )
            .with_worker_requirement(1, vec![ENGINEER.into()]),
        );
    if kind == FARMSTEAD {
        level = level.with_inventory_capacity(vec![ResourceAmount::new(GRAIN, 24)]);
    }
    BuildingDefinition::new(kind, name)
        .with_placement_rules(heartland_rules())
        .with_level(level)
}

fn frontier_building(
    kind: &str,
    name: &str,
    build_time: u64,
    cost: Vec<ResourceAmount>,
    production: ProductionRule,
) -> BuildingDefinition {
    let level_one_cost = cost.clone();
    BuildingDefinition::new(kind, name)
        .with_placement_rules(frontier_rules())
        .with_level(
            BuildingLevelDefinition::new(1, build_time, level_one_cost)
                .with_height(2)
                .with_inventory_capacity(vec![ResourceAmount::new(CITIZENS, 80)])
                .with_production_queue(ProductionQueueConfig::new(3))
                .with_production(production.clone()),
        )
        .with_level(
            BuildingLevelDefinition::new(2, build_time, cost)
                .with_height(3)
                .with_inventory_capacity(vec![ResourceAmount::new(CITIZENS, 160)])
                .with_production_queue(ProductionQueueConfig::new(3))
                .with_production(production),
        )
}

fn border_building(
    kind: &str,
    name: &str,
    build_time: u64,
    cost: Vec<ResourceAmount>,
    production: ProductionRule,
) -> BuildingDefinition {
    let level_one_cost = cost.clone();
    BuildingDefinition::new(kind, name)
        .with_placement_rules(border_rules())
        .with_level(
            BuildingLevelDefinition::new(1, build_time, level_one_cost)
                .with_height(3)
                .with_inventory_capacity(vec![ResourceAmount::new(GRAIN, 60)])
                .with_production_queue(ProductionQueueConfig::new(3))
                .with_production(production.clone()),
        )
        .with_level(
            BuildingLevelDefinition::new(2, build_time, cost)
                .with_height(4)
                .with_inventory_capacity(vec![ResourceAmount::new(GRAIN, 120)])
                .with_production_queue(ProductionQueueConfig::new(3))
                .with_production(production),
        )
}

fn heartland_rules() -> Vec<PlacementRule> {
    basic_build_rules()
}

fn frontier_rules() -> Vec<PlacementRule> {
    basic_build_rules()
}

fn border_rules() -> Vec<PlacementRule> {
    basic_build_rules()
}

fn basic_build_rules() -> Vec<PlacementRule> {
    vec![PlacementRule::WithinBounds, PlacementRule::NoOverlap]
}
