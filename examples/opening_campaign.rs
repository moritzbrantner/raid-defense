use raid_defense_game::{
    CONTROL, CROWNS, FRONTIER_FORT, GRAIN, INFLUENCE, LEGION_STRENGTH, LOYALTY, PROVINCE,
    RAID_DEFENSE_SIZE, RaidDefenseLogic, STABILITY, THREAT, claim_named_province,
    new_raid_defense_state, raid_defense_view,
};
use std::error::Error;

use farm_engine::{EntityBlueprintRef, MapLocation};

fn main() -> Result<(), Box<dyn Error>> {
    let mut state = new_raid_defense_state()?;

    let fort = state.start_construction_at(FRONTIER_FORT, MapLocation::new(23, 18))?;
    state.advance_time(24)?;
    state.set_building_stat(fort, "security", 70)?;
    claim_named_province(
        &mut state,
        PROVINCE,
        "Vesper March",
        MapLocation::new(23, 18),
    )?;

    let mut logic = RaidDefenseLogic;
    state.advance_time_with_logic(60, &mut logic)?;

    let view = raid_defense_view(&state);
    println!(
        "Raid Defense campaign reached {}s on a {}x{} map",
        view.now_seconds, RAID_DEFENSE_SIZE, RAID_DEFENSE_SIZE
    );
    println!();
    println!("Treasury");
    for resource in [CROWNS, GRAIN, INFLUENCE, LEGION_STRENGTH, STABILITY] {
        let amount = state.inventory().amount(resource);
        match state.inventory().capacity(resource) {
            Some(capacity) => println!("- {resource}: {amount} / {capacity}"),
            None => println!("- {resource}: {amount}"),
        }
    }

    println!();
    println!("Provinces");
    for province in state
        .entities()
        .filter(|entity| entity.blueprint == EntityBlueprintRef::Npc(PROVINCE.into()))
    {
        println!(
            "- {} at ({}, {}): control {}, loyalty {}, threat {}",
            province.name.unwrap_or_else(|| "Province".to_owned()),
            province.location.x,
            province.location.y,
            state.entity_stat(province.id, CONTROL)?,
            state.entity_stat(province.id, LOYALTY)?,
            state.entity_stat(province.id, THREAT)?
        );
    }

    println!();
    println!(
        "Summary: {} province(s), average control {}, average loyalty {}, rank {}",
        view.summary.province_count,
        view.summary.average_control,
        view.summary.average_loyalty,
        view.summary.influence_rank
    );

    Ok(())
}
