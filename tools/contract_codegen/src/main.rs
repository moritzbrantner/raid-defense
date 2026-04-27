use anyhow::{Context, Result, bail};
use farm_engine::{
    AdvanceTimeOutcome, Area, AreaDefinition, AreaId, AreaKind, Building, BuildingDefinition,
    BuildingFootprint, BuildingId, BuildingKind, BuildingLevelDefinition, BuildingStatus, Catalog,
    CatalogDocument, CommandEnvelope, CommandId, CommandJournalEntry, CommandOutcome, Effect,
    EntityAssignment, EntityBlueprintRef, EntityId, EntityRecord, Fence, FenceDefinition, FenceId,
    FenceKind, GameCommand, GameEvent, GameMapBounds, GameStateData, GameStateDocument,
    GameStateSave, GameWorldData, GameWorldDocument, GameWorldSave, InventoryScope, Job,
    JobCompletion, JobId, JobKind, JobStatus, LevelDefinition, LevelUp, MapLocation, MapTopology,
    NpcDefinition, NpcKind, Path, PathDefinition, PathId, PathKind, PlacementRule, PlacementTarget,
    PlayerId, PlayerProgress, ProductionIo, ProductionOrder, ProductionOrderId,
    ProductionOrderStatus, ProductionQueueConfig, ProductionRule, ProductionRuleId,
    ProductionStatus, QueuedProduction, QueuedProductionStatus, Requirement, ResourceAmount,
    ResourceDefinition, ResourceError, ResourceId, ResourceStorage, StatId, StoredWorldDocument,
    TechNode, TechNodeDefinition, TechNodeKind, Tile, TileDefinition, TileKind, UnitDefinition,
    UnitKind, Upgrade, UpgradeDefinition, UpgradeKind, WorldId,
};
use farm_scenario::{
    ScenarioArea, ScenarioBuilding, ScenarioDefinition, ScenarioFence, ScenarioGroundElevation,
    ScenarioMapBounds, ScenarioNpc, ScenarioPath, ScenarioStat, ScenarioTile, ScenarioUnit,
    WorldScenarioDefinition,
};
use raid_defense_game::{
    AlertView, AreaView, BuildingLogisticsView, BuildingView, EntityView, FoodLogisticsView,
    JobView, NpcView, ObjectiveView, PathView, ProvinceView, RaidDefenseCommand,
    RaidDefenseCommandRequest, RaidDefenseCommandResponse, RaidDefenseSummary, RaidDefenseView,
    ResourceView,
};
use schemars::schema_for;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path as FsPath, PathBuf};
use ts_rs::TS;

struct GeneratedFile {
    relative_path: &'static str,
    contents: String,
}

fn main() -> Result<()> {
    let check = std::env::args().skip(1).any(|arg| arg == "--check");
    let files = generated_files()?;
    let output_root = game_root().join("contracts/generated");

    if check {
        check_files(&output_root, &files)
    } else {
        write_files(&output_root, &files)
    }
}

fn game_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(FsPath::parent)
        .expect("raid-defense tool should live under tools/")
        .to_path_buf()
}

fn generated_files() -> Result<Vec<GeneratedFile>> {
    Ok(vec![
        schema_file::<RaidDefenseCommand>("json/RaidDefenseCommand.schema.json")?,
        schema_file::<RaidDefenseCommandRequest>("json/RaidDefenseCommandRequest.schema.json")?,
        schema_file::<RaidDefenseCommandResponse>("json/RaidDefenseCommandResponse.schema.json")?,
        schema_file::<RaidDefenseView>("json/RaidDefenseView.schema.json")?,
        GeneratedFile {
            relative_path: "ts/engine.d.ts",
            contents: render_engine_ts(),
        },
        GeneratedFile {
            relative_path: "ts/raid-defense.d.ts",
            contents: render_raid_defense_ts(),
        },
    ])
}

fn schema_file<T>(relative_path: &'static str) -> Result<GeneratedFile>
where
    T: schemars::JsonSchema,
{
    let root = schema_for!(T);
    Ok(GeneratedFile {
        relative_path,
        contents: serde_json::to_string_pretty(&root)
            .with_context(|| format!("serialize schema for {}", std::any::type_name::<T>()))?,
    })
}

fn render_engine_ts() -> String {
    let mut out = header();
    for decl in [
        ResourceId::decl(),
        BuildingKind::decl(),
        UpgradeKind::decl(),
        UnitKind::decl(),
        NpcKind::decl(),
        FenceKind::decl(),
        AreaKind::decl(),
        PathKind::decl(),
        PlayerId::decl(),
        TechNodeKind::decl(),
        TileKind::decl(),
        ProductionRuleId::decl(),
        StatId::decl(),
        WorldId::decl(),
        CommandId::decl(),
        BuildingId::decl(),
        EntityId::decl(),
        FenceId::decl(),
        AreaId::decl(),
        PathId::decl(),
        JobId::decl(),
        ProductionOrderId::decl(),
        MapTopology::decl(),
        MapLocation::decl(),
        ResourceAmount::decl(),
        ResourceError::decl(),
        ResourceStorage::decl(),
        Requirement::decl(),
        Effect::decl(),
        ProductionQueueConfig::decl(),
        ProductionRule::decl(),
        PlacementTarget::decl(),
        PlacementRule::decl(),
        ResourceDefinition::decl(),
        BuildingFootprint::decl(),
        BuildingLevelDefinition::decl(),
        BuildingDefinition::decl(),
        UpgradeDefinition::decl(),
        TechNodeDefinition::decl(),
        UnitDefinition::decl(),
        NpcDefinition::decl(),
        FenceDefinition::decl(),
        AreaDefinition::decl(),
        PathDefinition::decl(),
        TileDefinition::decl(),
        LevelDefinition::decl(),
        Catalog::decl(),
        CatalogDocument::decl(),
        BuildingStatus::decl(),
        ProductionStatus::decl(),
        QueuedProductionStatus::decl(),
        QueuedProduction::decl(),
        InventoryScope::decl(),
        ProductionIo::decl(),
        ProductionOrderStatus::decl(),
        ProductionOrder::decl(),
        Building::decl(),
        Upgrade::decl(),
        TechNode::decl(),
        EntityBlueprintRef::decl(),
        EntityAssignment::decl(),
        EntityRecord::decl(),
        Fence::decl(),
        Area::decl(),
        Path::decl(),
        Tile::decl(),
        JobKind::decl(),
        JobStatus::decl(),
        Job::decl(),
        PlayerProgress::decl(),
        LevelUp::decl(),
        JobCompletion::decl(),
        GameCommand::decl(),
        GameEvent::decl(),
        CommandEnvelope::decl(),
        CommandJournalEntry::decl(),
        CommandOutcome::decl(),
        AdvanceTimeOutcome::decl(),
        GameMapBounds::decl(),
        GameStateData::decl(),
        GameWorldData::decl(),
        GameStateSave::decl(),
        GameWorldSave::decl(),
        GameStateDocument::decl(),
        GameWorldDocument::decl(),
        StoredWorldDocument::decl(),
        ScenarioMapBounds::decl(),
        ScenarioGroundElevation::decl(),
        ScenarioTile::decl(),
        ScenarioStat::decl(),
        ScenarioBuilding::decl(),
        ScenarioUnit::decl(),
        ScenarioNpc::decl(),
        ScenarioFence::decl(),
        ScenarioArea::decl(),
        ScenarioPath::decl(),
        ScenarioDefinition::decl(),
        WorldScenarioDefinition::decl(),
    ] {
        out.push_str("export ");
        out.push_str(&decl);
        out.push_str("\n\n");
    }
    out
}

fn render_raid_defense_ts() -> String {
    let mut out = header();
    for decl in [
        RaidDefenseCommand::decl(),
        RaidDefenseCommandRequest::decl(),
        RaidDefenseCommandResponse::decl(),
        RaidDefenseView::decl(),
        FoodLogisticsView::decl(),
        ResourceView::decl(),
        BuildingView::decl(),
        BuildingLogisticsView::decl(),
        JobView::decl(),
        PathView::decl(),
        AreaView::decl(),
        EntityView::decl(),
        ProvinceView::decl(),
        NpcView::decl(),
        AlertView::decl(),
        ObjectiveView::decl(),
        RaidDefenseSummary::decl(),
    ] {
        out.push_str("export ");
        out.push_str(&decl);
        out.push_str("\n\n");
    }
    out
}

fn header() -> String {
    "// Generated by games/raid-defense/tools/contract_codegen. Do not edit.\n\n".to_owned()
}

fn write_files(output_root: &FsPath, files: &[GeneratedFile]) -> Result<()> {
    write_or_check_files(output_root, files, false)
}

fn check_files(output_root: &FsPath, files: &[GeneratedFile]) -> Result<()> {
    write_or_check_files(output_root, files, true)
}

fn write_or_check_files(output_root: &FsPath, files: &[GeneratedFile], check: bool) -> Result<()> {
    let expected = files
        .iter()
        .map(|file| file.relative_path)
        .collect::<BTreeSet<_>>();
    if !check {
        for dir in ["json", "ts"] {
            let root = output_root.join(dir);
            if root.exists() {
                fs::remove_dir_all(&root).with_context(|| format!("remove {}", root.display()))?;
            }
            fs::create_dir_all(&root).with_context(|| format!("create {}", root.display()))?;
        }
    }

    if check {
        let mut actual = BTreeSet::new();
        for dir in ["json", "ts"] {
            let root = output_root.join(dir);
            if !root.exists() {
                bail!("missing generated directory {}", root.display());
            }
            for entry in fs::read_dir(&root).with_context(|| format!("read {}", root.display()))? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    actual.insert(format!("{dir}/{}", entry.file_name().to_string_lossy()));
                }
            }
        }
        if actual != expected.iter().map(|path| path.to_string()).collect() {
            bail!("generated file set is stale");
        }
    }

    for file in files {
        let path = output_root.join(file.relative_path);
        if check {
            let current = fs::read_to_string(&path)
                .with_context(|| format!("read generated file {}", path.display()))?;
            if normalize_json_if_needed(file.relative_path, &current)?
                != normalize_json_if_needed(file.relative_path, &file.contents)?
            {
                bail!("generated file is stale: {}", path.display());
            }
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("create {}", parent.display()))?;
            }
            fs::write(&path, &file.contents)
                .with_context(|| format!("write {}", path.display()))?;
        }
    }

    Ok(())
}

fn normalize_json_if_needed(relative_path: &str, contents: &str) -> Result<String> {
    if !relative_path.ends_with(".json") {
        return Ok(contents.to_owned());
    }

    let value: Value = serde_json::from_str(contents)
        .with_context(|| format!("parse JSON artifact {relative_path}"))?;
    serde_json::to_string_pretty(&value)
        .with_context(|| format!("normalize JSON artifact {relative_path}"))
}
