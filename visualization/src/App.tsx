import { OrbitControls } from "@react-three/drei";
import { Canvas } from "@react-three/fiber";
import { useEffect, useMemo, useState } from "react";
import { RaidDefenseSimulationClient } from "./simulationClient";
import type {
  CellView,
  EntityView,
  RaidDefenseCommand,
  RaidDefenseEvent,
  SnapshotView,
  TowerArchetype,
} from "./simulationTypes";

const DEFAULT_SEED = 0x5eed;
const TICK_INTERVAL_MS = 100;

function towerName(archetype: TowerArchetype) {
  return archetype === "arrow" ? "Arrow tower" : "Cannon tower";
}

function formatCycleTimer(ticks: number) {
  const totalSeconds = Math.ceil((ticks * TICK_INTERVAL_MS) / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
}

function describeEvent(event: RaidDefenseEvent | null) {
  if (!event) {
    return "Command accepted.";
  }

  switch (event.type) {
    case "tower_construction_started":
      return `${towerName(event.archetype)} construction started at ${event.cell.x}, ${event.cell.z}. Workers must haul ${event.wood_required} wood to the site.`;
    case "sawmill_built":
      return `Sawmill built at ${event.cell.x}, ${event.cell.z}. It harvests nearby forest and workers haul the wood into settlement storage.`;
    case "storage_house_built":
      return `Storage house built at ${event.cell.x}, ${event.cell.z} with capacity for ${event.wood_capacity} wood.`;
    case "house_built":
      return `House built at ${event.cell.x}, ${event.cell.z}. ${event.people_added} people arrived; capacity is now ${event.population_capacity}.`;
    case "tower_upgraded":
      return `${towerName(event.archetype)} upgraded to level ${event.level} for ${event.wood_cost} wood.`;
    case "wave_started":
      return `Wave ${event.wave} started. Raiders are targeting the nearest storage with wood.`;
    case "tick_advanced": {
      const consequences = [
        event.wood_delivered > 0 ? `${event.wood_delivered} wood delivered` : null,
        event.towers_completed > 0
          ? `${event.towers_completed} tower${event.towers_completed === 1 ? "" : "s"} completed`
          : null,
        event.wood_stolen > 0 ? `${event.wood_stolen} wood stolen` : null,
        event.kills > 0 ? `${event.kills} kill${event.kills === 1 ? "" : "s"}` : null,
        event.town_damage > 0 ? `${event.town_damage} Town Hall damage` : null,
        event.completed_wave !== null ? `wave ${event.completed_wave} completed` : null,
      ].filter(Boolean);
      return consequences.length > 0
        ? `Tick ${event.tick}: ${consequences.join(" · ")}.`
        : `Tick ${event.tick}.`;
    }
  }
}

function describeError(code: string) {
  const labels: Record<string, string> = {
    out_of_bounds: "That cell is outside the build grid.",
    cell_occupied: "That cell is occupied by a resource, building, or moving unit.",
    protected_cell: "That cell is reserved for the Town Hall or an edge spawn gate.",
    path_blocked: "That building would seal a required raider or worker route.",
    insufficient_wood: "Settlement storage does not contain enough wood for that action.",
    no_forest_in_range: "A sawmill needs a harvestable forest within its working radius.",
    house_locked: "Houses unlock after you complete the first 10 waves.",
    no_tower: "There is no completed tower on the selected cell to upgrade.",
    max_tower_level: "That tower is already at the maximum level.",
    raiders_still_active: "Finish the current raid before starting another one.",
    game_over: "The Town Hall has fallen.",
    invalid_command_json: "The browser produced an invalid command envelope.",
  };
  return labels[code] ?? `Command rejected: ${code}.`;
}

function worldX(value: number, width: number) {
  return value - width / 2;
}

function worldZ(value: number, height: number) {
  return value - height / 2;
}

function entityPosition(entity: EntityView, snapshot: SnapshotView): [number, number] {
  return [
    worldX(entity.x_milli / 1000, snapshot.grid_width),
    worldZ(entity.z_milli / 1000, snapshot.grid_height),
  ];
}

function isTownCell(cell: CellView, snapshot: SnapshotView) {
  const centerX = Math.floor(snapshot.grid_width / 2);
  const centerZ = Math.floor(snapshot.grid_height / 2);
  return Math.abs(cell.x - centerX) <= 1 && Math.abs(cell.z - centerZ) <= 1;
}

function isGateCell(cell: CellView, snapshot: SnapshotView) {
  const middleX = Math.floor(snapshot.grid_width / 2);
  const middleZ = Math.floor(snapshot.grid_height / 2);
  return (
    (cell.x === middleX && (cell.z === 0 || cell.z === snapshot.grid_height - 1)) ||
    (cell.z === middleZ && (cell.x === 0 || cell.x === snapshot.grid_width - 1))
  );
}

function clampCellCoordinate(value: number, size: number) {
  return Math.max(0, Math.min(size - 1, value));
}

function MobileCellNudge({
  selectedCell,
  snapshot,
  onSelectCell,
}: {
  selectedCell: CellView;
  snapshot: SnapshotView;
  onSelectCell: (cell: CellView) => void;
}) {
  function move(dx: number, dz: number) {
    onSelectCell({
      x: clampCellCoordinate(selectedCell.x + dx, snapshot.grid_width),
      z: clampCellCoordinate(selectedCell.z + dz, snapshot.grid_height),
    });
  }

  return (
    <div className="mobile-cell-nudge" aria-label="Move selected grid cell">
      <button
        type="button"
        aria-label="Move selected cell west"
        data-testid="cell-west"
        disabled={selectedCell.x <= 0}
        onClick={() => move(-1, 0)}
      >
        ←
      </button>
      <button
        type="button"
        aria-label="Move selected cell north"
        data-testid="cell-north"
        disabled={selectedCell.z <= 0}
        onClick={() => move(0, -1)}
      >
        ↑
      </button>
      <output data-testid="mobile-selected-cell" aria-label="Selected grid cell">
        {selectedCell.x},{selectedCell.z}
      </output>
      <button
        type="button"
        aria-label="Move selected cell south"
        data-testid="cell-south"
        disabled={selectedCell.z >= snapshot.grid_height - 1}
        onClick={() => move(0, 1)}
      >
        ↓
      </button>
      <button
        type="button"
        aria-label="Move selected cell east"
        data-testid="cell-east"
        disabled={selectedCell.x >= snapshot.grid_width - 1}
        onClick={() => move(1, 0)}
      >
        →
      </button>
    </div>
  );
}

function GroundTile({
  cell,
  snapshot,
  selected,
  occupied,
  onSelectCell,
}: {
  cell: CellView;
  snapshot: SnapshotView;
  selected: boolean;
  occupied: boolean;
  onSelectCell: (cell: CellView) => void;
}) {
  const protectedCell = isTownCell(cell, snapshot) || isGateCell(cell, snapshot);
  const base = protectedCell ? "#384238" : occupied ? "#3f372e" : "#263129";
  return (
    <mesh
      position={[
        worldX(cell.x + 0.5, snapshot.grid_width),
        -0.08,
        worldZ(cell.z + 0.5, snapshot.grid_height),
      ]}
      onClick={(event) => {
        event.stopPropagation();
        if (event.delta > 6) return;
        onSelectCell(cell);
      }}
      receiveShadow
    >
      <boxGeometry args={[0.94, 0.12, 0.94]} />
      <meshStandardMaterial color={selected ? "#d1af63" : base} roughness={0.92} />
    </mesh>
  );
}

function TownHallModel({ entity, snapshot }: { entity: EntityView; snapshot: SnapshotView }) {
  const [x, z] = entityPosition(entity, snapshot);
  return (
    <group position={[x, 0, z]}>
      <mesh position={[0, 0.38, 0]} castShadow receiveShadow>
        <boxGeometry args={[2.7, 0.72, 2.7]} />
        <meshStandardMaterial color="#a78b5a" roughness={0.82} />
      </mesh>
      <mesh position={[0, 1.08, 0]} castShadow>
        <boxGeometry args={[1.65, 0.75, 1.65]} />
        <meshStandardMaterial color="#d1c29b" roughness={0.76} />
      </mesh>
      <mesh position={[0, 1.72, 0]} rotation={[0, Math.PI / 4, 0]} castShadow>
        <coneGeometry args={[1.35, 1.05, 4]} />
        <meshStandardMaterial color="#725246" roughness={0.8} />
      </mesh>
    </group>
  );
}

function ForestModel({ entity, snapshot }: { entity: EntityView; snapshot: SnapshotView }) {
  const [x, z] = entityPosition(entity, snapshot);
  const ratio = entity.wood_capacity > 0 ? entity.stored_wood / entity.wood_capacity : 0;
  const scale = 0.72 + ratio * 0.28;
  return (
    <group position={[x, 0, z]} scale={scale}>
      <mesh position={[0, 0.32, 0]} castShadow>
        <cylinderGeometry args={[0.1, 0.14, 0.64, 7]} />
        <meshStandardMaterial color="#6f4b2f" roughness={0.95} />
      </mesh>
      <mesh position={[0, 0.86, 0]} castShadow>
        <coneGeometry args={[0.48, 0.95, 7]} />
        <meshStandardMaterial color="#365a35" roughness={0.92} />
      </mesh>
    </group>
  );
}

function TowerModel({ entity, snapshot }: { entity: EntityView; snapshot: SnapshotView }) {
  const [x, z] = entityPosition(entity, snapshot);
  if (entity.tower_level === 0) {
    const ratio = entity.wood_capacity > 0 ? entity.stored_wood / entity.wood_capacity : 0;
    return (
      <group position={[x, 0, z]}>
        <mesh position={[0, 0.14, 0]} castShadow>
          <boxGeometry args={[0.74, 0.22, 0.74]} />
          <meshStandardMaterial color="#806b4b" roughness={0.96} />
        </mesh>
        <mesh position={[-0.26, 0.52, -0.26]} castShadow>
          <boxGeometry args={[0.08, 0.8, 0.08]} />
          <meshStandardMaterial color="#a27a45" roughness={0.92} />
        </mesh>
        <mesh position={[0.26, 0.52, 0.26]} castShadow>
          <boxGeometry args={[0.08, 0.8, 0.08]} />
          <meshStandardMaterial color="#a27a45" roughness={0.92} />
        </mesh>
        <mesh position={[0, 0.38, 0]} castShadow scale={[Math.max(0.1, ratio), 1, 1]}>
          <boxGeometry args={[0.58, 0.12, 0.18]} />
          <meshStandardMaterial color="#c69b61" roughness={0.9} />
        </mesh>
      </group>
    );
  }

  const archetype = entity.tower_archetype ?? "arrow";
  const levelScale = 1 + Math.max(0, entity.tower_level - 1) * 0.1;
  return (
    <group position={[x, 0, z]} scale={levelScale}>
      <mesh position={[0, 0.28, 0]} castShadow receiveShadow>
        <cylinderGeometry args={[0.42, 0.48, 0.54, 8]} />
        <meshStandardMaterial color={archetype === "cannon" ? "#6d665e" : "#81705b"} roughness={0.86} />
      </mesh>
      <mesh position={[0, 0.82, 0]} castShadow>
        <boxGeometry args={[0.56, 0.58, 0.56]} />
        <meshStandardMaterial color={archetype === "cannon" ? "#8b806f" : "#b6a384"} roughness={0.78} />
      </mesh>
      {archetype === "cannon" ? (
        <mesh position={[0, 1.03, -0.32]} rotation={[Math.PI / 2, 0, 0]} castShadow>
          <cylinderGeometry args={[0.1, 0.14, 0.7, 10]} />
          <meshStandardMaterial color="#3d4140" metalness={0.35} roughness={0.58} />
        </mesh>
      ) : (
        <mesh position={[0, 1.3, 0]} rotation={[0, Math.PI / 4, 0]} castShadow>
          <coneGeometry args={[0.44, 0.46, 4]} />
          <meshStandardMaterial color="#5f493f" roughness={0.78} />
        </mesh>
      )}
    </group>
  );
}

function SawmillModel({ entity, snapshot }: { entity: EntityView; snapshot: SnapshotView }) {
  const [x, z] = entityPosition(entity, snapshot);
  return (
    <group position={[x, 0, z]}>
      <mesh position={[0, 0.3, 0]} castShadow>
        <boxGeometry args={[0.8, 0.54, 0.76]} />
        <meshStandardMaterial color="#7c5a38" roughness={0.9} />
      </mesh>
      <mesh position={[0.43, 0.42, 0]} rotation={[Math.PI / 2, 0, 0]} castShadow>
        <cylinderGeometry args={[0.24, 0.24, 0.08, 16]} />
        <meshStandardMaterial color="#aaa79a" metalness={0.35} roughness={0.6} />
      </mesh>
    </group>
  );
}

function StorageHouseModel({ entity, snapshot }: { entity: EntityView; snapshot: SnapshotView }) {
  const [x, z] = entityPosition(entity, snapshot);
  const ratio = entity.wood_capacity > 0 ? entity.stored_wood / entity.wood_capacity : 0;
  return (
    <group position={[x, 0, z]}>
      <mesh position={[0, 0.34, 0]} castShadow receiveShadow>
        <boxGeometry args={[0.9, 0.64, 0.84]} />
        <meshStandardMaterial color="#8a7353" roughness={0.9} />
      </mesh>
      <mesh position={[0, 0.82, 0]} rotation={[0, Math.PI / 4, 0]} castShadow>
        <coneGeometry args={[0.68, 0.5, 4]} />
        <meshStandardMaterial color="#514137" roughness={0.86} />
      </mesh>
      {ratio > 0 && (
        <mesh position={[0, 0.2, 0.46]} castShadow scale={[0.5 + ratio * 0.5, 1, 1]}>
          <boxGeometry args={[0.64, 0.18, 0.18]} />
          <meshStandardMaterial color="#9b6834" roughness={0.95} />
        </mesh>
      )}
    </group>
  );
}

function HouseModel({ entity, snapshot }: { entity: EntityView; snapshot: SnapshotView }) {
  const [x, z] = entityPosition(entity, snapshot);
  return (
    <group position={[x, 0, z]}>
      <mesh position={[0, 0.32, 0]} castShadow receiveShadow>
        <boxGeometry args={[0.82, 0.58, 0.72]} />
        <meshStandardMaterial color="#c0a27a" roughness={0.88} />
      </mesh>
      <mesh position={[0, 0.78, 0]} rotation={[0, Math.PI / 4, 0]} castShadow>
        <coneGeometry args={[0.62, 0.52, 4]} />
        <meshStandardMaterial color="#68483a" roughness={0.84} />
      </mesh>
    </group>
  );
}

function PersonModel({ entity, snapshot }: { entity: EntityView; snapshot: SnapshotView }) {
  const [x, z] = entityPosition(entity, snapshot);
  return (
    <group position={[x, 0, z]}>
      <mesh position={[0, 0.31, 0]} castShadow>
        <capsuleGeometry args={[0.12, 0.27, 3, 6]} />
        <meshStandardMaterial color="#4f6b55" roughness={0.8} />
      </mesh>
      <mesh position={[0, 0.63, 0]} castShadow>
        <sphereGeometry args={[0.12, 8, 6]} />
        <meshStandardMaterial color="#bd8f6d" roughness={0.82} />
      </mesh>
      {entity.cargo_wood > 0 && (
        <mesh position={[0.18, 0.32, 0]} castShadow>
          <boxGeometry args={[0.2, 0.24, 0.24]} />
          <meshStandardMaterial color="#916132" roughness={0.95} />
        </mesh>
      )}
    </group>
  );
}

function RaiderModel({ entity, snapshot }: { entity: EntityView; snapshot: SnapshotView }) {
  const [x, z] = entityPosition(entity, snapshot);
  const healthRatio = entity.max_health > 0 ? entity.health / entity.max_health : 0;
  return (
    <group position={[x, 0, z]}>
      <mesh position={[0, 0.38, 0]} castShadow>
        <capsuleGeometry args={[0.22, 0.4, 4, 8]} />
        <meshStandardMaterial color="#7a3030" roughness={0.72} />
      </mesh>
      <mesh position={[0, 0.82, 0]} castShadow>
        <sphereGeometry args={[0.2, 10, 8]} />
        <meshStandardMaterial color="#b78463" roughness={0.82} />
      </mesh>
      <mesh position={[0, 1.15, 0]}>
        <boxGeometry args={[0.58, 0.055, 0.07]} />
        <meshBasicMaterial color="#171b18" />
      </mesh>
      <mesh position={[-0.29 + 0.58 * healthRatio * 0.5, 1.15, -0.04]}>
        <boxGeometry args={[0.58 * healthRatio, 0.04, 0.04]} />
        <meshBasicMaterial color="#a9b75e" />
      </mesh>
    </group>
  );
}

function ProjectileModel({ entity, snapshot }: { entity: EntityView; snapshot: SnapshotView }) {
  const [x, z] = entityPosition(entity, snapshot);
  const cannon = entity.tower_archetype === "cannon";
  return (
    <mesh position={[x, cannon ? 0.72 : 0.9, z]} castShadow>
      <sphereGeometry args={[cannon ? 0.14 : 0.07, 10, 8]} />
      <meshStandardMaterial color={cannon ? "#d58a49" : "#d8c485"} roughness={0.45} />
    </mesh>
  );
}

function GateMarker({ cell, snapshot }: { cell: CellView; snapshot: SnapshotView }) {
  return (
    <group
      position={[
        worldX(cell.x + 0.5, snapshot.grid_width),
        0,
        worldZ(cell.z + 0.5, snapshot.grid_height),
      ]}
    >
      <mesh position={[-0.3, 0.38, 0]} castShadow>
        <boxGeometry args={[0.16, 0.76, 0.16]} />
        <meshStandardMaterial color="#655d50" />
      </mesh>
      <mesh position={[0.3, 0.38, 0]} castShadow>
        <boxGeometry args={[0.16, 0.76, 0.16]} />
        <meshStandardMaterial color="#655d50" />
      </mesh>
      <mesh position={[0, 0.72, 0]} castShadow>
        <boxGeometry args={[0.76, 0.14, 0.16]} />
        <meshStandardMaterial color="#655d50" />
      </mesh>
    </group>
  );
}

function GameWorld({
  snapshot,
  selectedCell,
  onSelectCell,
}: {
  snapshot: SnapshotView;
  selectedCell: CellView;
  onSelectCell: (cell: CellView) => void;
}) {
  const cells = useMemo(() => {
    const result: CellView[] = [];
    for (let z = 0; z < snapshot.grid_height; z += 1) {
      for (let x = 0; x < snapshot.grid_width; x += 1) {
        result.push({ x, z });
      }
    }
    return result;
  }, [snapshot.grid_height, snapshot.grid_width]);
  const occupied = useMemo(
    () =>
      new Set(
        snapshot.entities
          .filter((entity) =>
            ["tower", "sawmill", "storage_house", "house", "forest"].includes(entity.kind),
          )
          .map((entity) => `${entity.cell.x}:${entity.cell.z}`),
      ),
    [snapshot.entities],
  );
  const gates = [
    { x: Math.floor(snapshot.grid_width / 2), z: 0 },
    { x: snapshot.grid_width - 1, z: Math.floor(snapshot.grid_height / 2) },
    { x: Math.floor(snapshot.grid_width / 2), z: snapshot.grid_height - 1 },
    { x: 0, z: Math.floor(snapshot.grid_height / 2) },
  ];

  return (
    <Canvas
      camera={{ position: [15, 18, 19], fov: 42, near: 0.1, far: 100 }}
      shadows
      dpr={[1, 1.6]}
      style={{ touchAction: "none" }}
    >
      <color attach="background" args={["#111712"]} />
      <fog attach="fog" args={["#111712", 26, 54]} />
      <ambientLight intensity={1.3} />
      <directionalLight position={[9, 16, 10]} intensity={2.1} castShadow />
      {cells.map((cell) => (
        <GroundTile
          key={`${cell.x}:${cell.z}`}
          cell={cell}
          snapshot={snapshot}
          selected={cell.x === selectedCell.x && cell.z === selectedCell.z}
          occupied={occupied.has(`${cell.x}:${cell.z}`)}
          onSelectCell={onSelectCell}
        />
      ))}
      {gates.map((cell) => (
        <GateMarker key={`gate-${cell.x}:${cell.z}`} cell={cell} snapshot={snapshot} />
      ))}
      {snapshot.entities.map((entity) => {
        switch (entity.kind) {
          case "town_hall":
            return <TownHallModel key={entity.id} entity={entity} snapshot={snapshot} />;
          case "tower":
            return <TowerModel key={entity.id} entity={entity} snapshot={snapshot} />;
          case "sawmill":
            return <SawmillModel key={entity.id} entity={entity} snapshot={snapshot} />;
          case "storage_house":
            return <StorageHouseModel key={entity.id} entity={entity} snapshot={snapshot} />;
          case "house":
            return <HouseModel key={entity.id} entity={entity} snapshot={snapshot} />;
          case "forest":
            return <ForestModel key={entity.id} entity={entity} snapshot={snapshot} />;
          case "person":
            return <PersonModel key={entity.id} entity={entity} snapshot={snapshot} />;
          case "raider":
            return <RaiderModel key={entity.id} entity={entity} snapshot={snapshot} />;
          case "projectile":
            return <ProjectileModel key={entity.id} entity={entity} snapshot={snapshot} />;
        }
      })}
      <OrbitControls makeDefault target={[0, 0.4, 0]} minDistance={9} maxDistance={42} maxPolarAngle={Math.PI * 0.47} enableDamping />
    </Canvas>
  );
}

function App() {
  const [client, setClient] = useState<RaidDefenseSimulationClient | null>(null);
  const [snapshot, setSnapshot] = useState<SnapshotView | null>(null);
  const [selectedCell, setSelectedCell] = useState<CellView>({ x: 2, z: 2 });
  const [feedback, setFeedback] = useState("Initializing deterministic defense grid…");

  useEffect(() => {
    let cancelled = false;
    void RaidDefenseSimulationClient.create(DEFAULT_SEED)
      .then((simulation) => {
        if (cancelled) return;
        const initial = simulation.snapshot();
        setClient(simulation);
        setSnapshot(initial);
        setFeedback(
          "Forests are finite. Sawmills harvest nearby trees, workers move wood between stores, and tower sites need physical deliveries.",
        );
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setFeedback(error instanceof Error ? error.message : "Failed to initialize the simulation.");
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const townHealth = snapshot?.town_health;

  function issue(command: RaidDefenseCommand) {
    if (!client) return;
    const response = client.dispatch(command);
    setSnapshot(response.snapshot);
    setFeedback(response.ok ? describeEvent(response.event) : describeError(response.error?.code ?? "unknown_error"));
  }

  useEffect(() => {
    if (!client || townHealth === undefined || townHealth === 0) return;
    const timer = window.setInterval(() => {
      const response = client.dispatch({ type: "advance_tick" });
      setSnapshot(response.snapshot);
      if (!response.ok) {
        setFeedback(describeError(response.error?.code ?? "unknown_error"));
        return;
      }
      if (
        response.event?.type === "tick_advanced" &&
        (response.event.wood_delivered > 0 ||
          response.event.towers_completed > 0 ||
          response.event.wood_stolen > 0 ||
          response.event.kills > 0 ||
          response.event.town_damage > 0 ||
          response.event.completed_wave !== null)
      ) {
        setFeedback(describeEvent(response.event));
      }
    }, TICK_INTERVAL_MS);
    return () => window.clearInterval(timer);
  }, [client, townHealth]);

  if (!snapshot) {
    return (
      <main className="loading-shell" data-testid="loading-state">
        <h1>Raid Defense</h1>
        <p>{feedback}</p>
      </main>
    );
  }

  const towerEntities = snapshot.entities.filter((entity) => entity.kind === "tower");
  const towers = towerEntities.filter((entity) => entity.tower_level > 0);
  const constructionSites = towerEntities.filter((entity) => entity.tower_level === 0);
  const sawmills = snapshot.entities.filter((entity) => entity.kind === "sawmill");
  const storageHouses = snapshot.entities.filter((entity) => entity.kind === "storage_house");
  const forests = snapshot.entities.filter((entity) => entity.kind === "forest");
  const houses = snapshot.entities.filter((entity) => entity.kind === "house");
  const people = snapshot.entities.filter((entity) => entity.kind === "person");
  const raiderCount = snapshot.entities.filter((entity) => entity.kind === "raider").length;
  const projectileCount = snapshot.entities.filter((entity) => entity.kind === "projectile").length;
  const selectedTower = towerEntities.find(
    (entity) => entity.cell.x === selectedCell.x && entity.cell.z === selectedCell.z,
  );
  const selectedSawmill = sawmills.find(
    (entity) => entity.cell.x === selectedCell.x && entity.cell.z === selectedCell.z,
  );
  const selectedStorage = storageHouses.find(
    (entity) => entity.cell.x === selectedCell.x && entity.cell.z === selectedCell.z,
  );
  const selectedForest = forests.find(
    (entity) => entity.cell.x === selectedCell.x && entity.cell.z === selectedCell.z,
  );
  const selectedHouse = houses.find(
    (entity) => entity.cell.x === selectedCell.x && entity.cell.z === selectedCell.z,
  );
  const townPercent = Math.max(0, Math.min(100, (snapshot.town_health / snapshot.town_max_health) * 100));
  const woodPercent = Math.max(0, Math.min(100, (snapshot.wood / snapshot.wood_capacity) * 100));
  const canStartWave = !snapshot.is_night && snapshot.town_health > 0;
  const canUpgrade =
    selectedTower !== undefined &&
    selectedTower.tower_level > 0 &&
    selectedTower.upgrade_cost !== null &&
    snapshot.wood >= selectedTower.upgrade_cost &&
    snapshot.town_health > 0;
  const canBuildHouse =
    snapshot.houses_unlocked && snapshot.town_health > 0 && snapshot.wood >= snapshot.house_cost;
  const carryingPeople = people.filter((person) => person.cargo_wood > 0).length;

  const selectedDescription = selectedTower?.tower_archetype
    ? selectedTower.tower_level === 0
      ? `${towerName(selectedTower.tower_archetype)} construction · ${selectedTower.stored_wood}/${selectedTower.wood_capacity} wood delivered`
      : `${towerName(selectedTower.tower_archetype)} · L${selectedTower.tower_level} · ${selectedTower.attack_damage} dmg`
    : selectedSawmill
      ? `Sawmill · local wood ${selectedSawmill.stored_wood}/${selectedSawmill.wood_capacity}`
      : selectedStorage
        ? `Storage house · ${selectedStorage.stored_wood}/${selectedStorage.wood_capacity} wood`
        : selectedForest
          ? `Forest · ${selectedForest.stored_wood}/${selectedForest.wood_capacity} wood remaining`
          : selectedHouse
            ? `House · +${selectedHouse.housing_capacity} population capacity`
            : "Empty build cell";

  return (
    <main className="game-shell" data-testid="raid-defense-game">
      <header className="top-bar">
        <div className="identity">
          <span className="eyebrow">Economy-defense prototype</span>
          <h1>Raid Defense</h1>
        </div>
        <dl className="status-strip" aria-label="Settlement status">
          <div><dt>Wood</dt><dd data-testid="wood-value">{snapshot.wood}</dd></div>
          <div><dt>People</dt><dd data-testid="people-value">{snapshot.people}/{snapshot.population_capacity}</dd></div>
          <div><dt>Hauling</dt><dd data-testid="hauling-value">{carryingPeople}</dd></div>
          <div><dt>Cycle</dt><dd data-testid="cycle-timer">{snapshot.is_night ? `Night · Wave ${snapshot.wave}` : `Day · ${formatCycleTimer(snapshot.day_ticks_remaining)}`}</dd></div>
          <div><dt>Wave</dt><dd data-testid="wave-value">{snapshot.wave}</dd></div>
          <div><dt>Cleared</dt><dd data-testid="completed-waves-value">{snapshot.completed_waves}</dd></div>
          <div><dt>Raiders</dt><dd data-testid="raider-count">{raiderCount}</dd></div>
          <div><dt>Forests</dt><dd data-testid="forest-count">{forests.length}</dd></div>
          <div><dt>Stores</dt><dd data-testid="storage-count">{storageHouses.length}</dd></div>
          <div><dt>Sawmills</dt><dd data-testid="sawmill-count">{sawmills.length}</dd></div>
          <div><dt>Building</dt><dd data-testid="construction-count">{constructionSites.length}</dd></div>
          <div><dt>Towers</dt><dd data-testid="tower-count">{towers.length}</dd></div>
        </dl>
        <button className="wave-button" type="button" disabled={!canStartWave} onClick={() => issue({ type: "start_wave" })} data-testid="start-wave">
          Start raid
        </button>
      </header>

      <section className="battlefield" data-testid="world-3d" aria-label="3D defense grid">
        <GameWorld snapshot={snapshot} selectedCell={selectedCell} onSelectCell={setSelectedCell} />
        <div className="town-health" aria-label="Settlement status">
          <span>Town Hall {snapshot.town_health}/{snapshot.town_max_health}</span>
          <span className="health-track" aria-hidden="true"><span style={{ width: `${townPercent}%` }} /></span>
          <span>Settlement wood {snapshot.wood}/{snapshot.wood_capacity}</span>
          <span className="wood-track" aria-hidden="true"><span style={{ width: `${woodPercent}%` }} /></span>
          <span data-testid="house-lock-state">
            {snapshot.houses_unlocked
              ? "Houses unlocked"
              : `Houses unlock after ${snapshot.house_unlock_completed_waves} completed waves (${snapshot.completed_waves}/${snapshot.house_unlock_completed_waves})`}
          </span>
        </div>
        <p className="mobile-world-hint" data-testid="mobile-world-hint">
          Tap a tile to select · drag to orbit · pinch to zoom
        </p>
      </section>

      <section className="build-bar" aria-label="Build controls" data-testid="mobile-command-dock">
        <div className="cell-controls">
          <span className="eyebrow">Selected grid cell</span>
          <div className="coordinate-inputs">
            <label>
              X
              <input
                data-testid="cell-x"
                type="number"
                min={0}
                max={snapshot.grid_width - 1}
                value={selectedCell.x}
                onChange={(event) =>
                  setSelectedCell((current) => ({
                    ...current,
                    x: clampCellCoordinate(Number(event.target.value), snapshot.grid_width),
                  }))
                }
              />
            </label>
            <label>
              Z
              <input
                data-testid="cell-z"
                type="number"
                min={0}
                max={snapshot.grid_height - 1}
                value={selectedCell.z}
                onChange={(event) =>
                  setSelectedCell((current) => ({
                    ...current,
                    z: clampCellCoordinate(Number(event.target.value), snapshot.grid_height),
                  }))
                }
              />
            </label>
          </div>
          <MobileCellNudge
            selectedCell={selectedCell}
            snapshot={snapshot}
            onSelectCell={setSelectedCell}
          />
          <span className="selected-tower" data-testid="selected-building">{selectedDescription}</span>
        </div>
        <div className="tower-actions">
          <button className="build-button economy-button" type="button" disabled={snapshot.town_health === 0 || snapshot.wood < snapshot.sawmill_cost} onClick={() => issue({ type: "place_sawmill", x: selectedCell.x, z: selectedCell.z })} data-testid="build-sawmill">
            Sawmill · {snapshot.sawmill_cost}w
          </button>
          <button className="build-button economy-button" type="button" disabled={snapshot.town_health === 0 || snapshot.wood < snapshot.storage_house_cost} onClick={() => issue({ type: "place_storage_house", x: selectedCell.x, z: selectedCell.z })} data-testid="build-storage-house">
            Storage · {snapshot.storage_house_cost}w
          </button>
          <button className="build-button economy-button" type="button" disabled={!canBuildHouse} onClick={() => issue({ type: "place_house", x: selectedCell.x, z: selectedCell.z })} data-testid="build-house">
            {snapshot.houses_unlocked ? `House · ${snapshot.house_cost}w` : `House · unlocks after ${snapshot.house_unlock_completed_waves} waves`}
          </button>
          <button className="build-button" type="button" disabled={snapshot.town_health === 0 || snapshot.wood < snapshot.arrow_tower_cost} onClick={() => issue({ type: "place_tower", x: selectedCell.x, z: selectedCell.z, archetype: "arrow" })} data-testid="build-arrow-tower">
            Arrow · {snapshot.arrow_tower_cost}w
          </button>
          <button className="build-button" type="button" disabled={snapshot.town_health === 0 || snapshot.wood < snapshot.cannon_tower_cost} onClick={() => issue({ type: "place_tower", x: selectedCell.x, z: selectedCell.z, archetype: "cannon" })} data-testid="build-cannon-tower">
            Cannon · {snapshot.cannon_tower_cost}w
          </button>
          <button className="build-button upgrade-button" type="button" disabled={!canUpgrade} onClick={() => issue({ type: "upgrade_tower", x: selectedCell.x, z: selectedCell.z })} data-testid="upgrade-tower">
            {selectedTower?.tower_level === 0
              ? "Awaiting materials"
              : selectedTower?.upgrade_cost === null
                ? "Max level"
                : `Upgrade${selectedTower?.upgrade_cost ? ` · ${selectedTower.upgrade_cost}w` : ""}`}
          </button>
        </div>
        <p className="feedback-line" role="status" aria-live="polite" data-testid="event-feedback">{feedback}</p>
      </section>

      <footer className="evidence-line">
        <span>Seed {snapshot.seed}</span>
        <span>Map {snapshot.grid_width}×{snapshot.grid_height}</span>
        <span>Contract v{snapshot.contract_version}</span>
        <span data-testid="projectile-count">Projectiles {projectileCount}</span>
        <span data-testid="checksum">Checksum {snapshot.checksum}</span>
      </footer>
    </main>
  );
}

export default App;
