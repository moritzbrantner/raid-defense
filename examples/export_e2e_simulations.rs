use farm_engine::MapLocation;
use raid_defense_game::{
    FRONTIER_FORT, PROVINCE, RaidDefenseLogic, claim_named_province,
    new_raid_defense_state_with_seed,
};
use serde_json::json;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let fixtures_dir = PathBuf::from("visualization/e2e/fixtures");
    fs::create_dir_all(&fixtures_dir)?;

    let seed = 42_u64;

    let fresh_state = new_raid_defense_state_with_seed(seed)?;
    write_fixture(&fixtures_dir, "fresh-frontier", seed, "Fresh Frontier", &fresh_state)?;

    let mut province_state = new_raid_defense_state_with_seed(seed)?;
    province_state.inventory_mut().set_capacity("crowns", 200);
    province_state.inventory_mut().add("crowns", 200)?;
    province_state.inventory_mut().set_capacity("influence", 100);
    province_state.inventory_mut().add("influence", 100)?;
    province_state
        .inventory_mut()
        .set_capacity("legion_strength", 100);
    province_state.inventory_mut().add("legion_strength", 100)?;
    let fort = province_state.start_construction_at(FRONTIER_FORT, MapLocation::new(23, 18))?;
    province_state.advance_time(24)?;
    province_state.set_building_stat(fort, "security", 70)?;
    claim_named_province(
        &mut province_state,
        PROVINCE,
        "Vesper March",
        MapLocation::new(23, 18),
    )?;
    let mut logic = RaidDefenseLogic;
    province_state.advance_time_with_logic(60, &mut logic)?;
    write_fixture(
        &fixtures_dir,
        "claimed-province",
        seed,
        "Claimed Province",
        &province_state,
    )?;

    Ok(())
}

fn write_fixture(
    fixtures_dir: &PathBuf,
    slug: &str,
    seed: u64,
    name: &str,
    state: &farm_engine::GameState,
) -> Result<(), Box<dyn Error>> {
    let file = json!({
        "format": "raid-defense-simulation",
        "version": 1,
        "mode": "simulation",
        "name": name,
        "seed": seed.to_string(),
        "exported_at": "2026-04-27T00:00:00.000Z",
        "now_seconds": state.now_seconds(),
        "snapshot_json": serde_json::to_string(&state.save_snapshot())?,
    });

    fs::write(
        fixtures_dir.join(format!("{slug}.json")),
        serde_json::to_string_pretty(&file)?,
    )?;

    Ok(())
}
