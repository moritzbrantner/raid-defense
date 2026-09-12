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
} from "./simulationTypes";

const DEFAULT_SEED = 0x5eed;
const TICK_INTERVAL_MS = 100;

function describeEvent(event: RaidDefenseEvent | null) {
  if (!event) {
    return "Command accepted.";
  }

  switch (event.type) {
    case "tower_built":
      return `Guard tower built at ${event.cell.x}, ${event.cell.z}.`;
    case "wave_started":
      return `Wave ${event.wave} started from all four edges with ${event.raiders} raiders.`;
    case "tick_advanced": {
      const consequences = [
        event.shots > 0 ? `${event.shots} shot${event.shots === 1 ? "" : "s"}` : null,
        event.kills > 0 ? `${event.kills} kill${event.kills === 1 ? "" : "s"}` : null,
        event.town_damage > 0 ? `${event.town_damage} town damage` : null,
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
    cell_occupied: "That cell is occupied by a building or a moving raider.",
    protected_cell: "That cell is reserved for the town or an edge spawn gate.",
    path_blocked: "That tower would seal a route from an edge to the town.",
    insufficient_gold: "You do not have enough gold for that tower.",
    raiders_still_active: "Finish the current wave before starting another one.",
    game_over: "The town has fallen.",
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

type WorldProps = {
  snapshot: SnapshotView;
  selectedCell: CellView;
  onSelectCell: (cell: CellView) => void;
};

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
  const color = selected ? "#d1af63" : base;

  return (
    <mesh
      position={[
        worldX(cell.x + 0.5, snapshot.grid_width),
        -0.08,
        worldZ(cell.z + 0.5, snapshot.grid_height),
      ]}
      onClick={(event) => {
        event.stopPropagation();
        onSelectCell(cell);
      }}
      receiveShadow
    >
      <boxGeometry args={[0.94, 0.12, 0.94]} />
      <meshStandardMaterial color={color} roughness={0.92} />
    </mesh>
  );
}

function TownModel({ entity, snapshot }: { entity: EntityView; snapshot: SnapshotView }) {
  const x = worldX(entity.x_milli / 1000, snapshot.grid_width);
  const z = worldZ(entity.z_milli / 1000, snapshot.grid_height);

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

function TowerModel({ entity, snapshot }: { entity: EntityView; snapshot: SnapshotView }) {
  const x = worldX(entity.x_milli / 1000, snapshot.grid_width);
  const z = worldZ(entity.z_milli / 1000, snapshot.grid_height);

  return (
    <group position={[x, 0, z]}>
      <mesh position={[0, 0.22, 0]} castShadow receiveShadow>
        <cylinderGeometry args={[0.38, 0.46, 0.44, 8]} />
        <meshStandardMaterial color="#81705b" roughness={0.88} />
      </mesh>
      <mesh position={[0, 0.82, 0]} castShadow>
        <cylinderGeometry args={[0.24, 0.31, 0.92, 8]} />
        <meshStandardMaterial color="#b6a384" roughness={0.76} />
      </mesh>
      <mesh position={[0, 1.32, 0]} rotation={[0, Math.PI / 4, 0]} castShadow>
        <coneGeometry args={[0.46, 0.48, 4]} />
        <meshStandardMaterial color="#5f493f" roughness={0.78} />
      </mesh>
    </group>
  );
}

function RaiderModel({ entity, snapshot }: { entity: EntityView; snapshot: SnapshotView }) {
  const x = worldX(entity.x_milli / 1000, snapshot.grid_width);
  const z = worldZ(entity.z_milli / 1000, snapshot.grid_height);
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

function GameWorld({ snapshot, selectedCell, onSelectCell }: WorldProps) {
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
          .filter((entity) => entity.kind === "tower")
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
      camera={{ position: [11, 13, 15], fov: 42, near: 0.1, far: 80 }}
      shadows
      dpr={[1, 1.75]}
    >
      <color attach="background" args={["#111712"]} />
      <fog attach="fog" args={["#111712", 20, 42]} />
      <ambientLight intensity={1.3} />
      <directionalLight
        position={[7, 14, 8]}
        intensity={2.1}
        castShadow
        shadow-mapSize-width={1024}
        shadow-mapSize-height={1024}
      />

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
          case "town":
            return <TownModel key={entity.id} entity={entity} snapshot={snapshot} />;
          case "tower":
            return <TowerModel key={entity.id} entity={entity} snapshot={snapshot} />;
          case "raider":
            return <RaiderModel key={entity.id} entity={entity} snapshot={snapshot} />;
        }
      })}

      <OrbitControls
        makeDefault
        target={[0, 0.4, 0]}
        minDistance={8}
        maxDistance={32}
        maxPolarAngle={Math.PI * 0.47}
        enableDamping
      />
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
        if (cancelled) {
          return;
        }
        const initial = simulation.snapshot();
        setClient(simulation);
        setSnapshot(initial);
        setFeedback("Build a maze of towers, keep every edge connected, then start the wave.");
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

  const raiderCount = snapshot?.entities.filter((entity) => entity.kind === "raider").length ?? 0;

  function issue(command: RaidDefenseCommand) {
    if (!client) {
      return;
    }
    const response = client.dispatch(command);
    setSnapshot(response.snapshot);
    setFeedback(
      response.ok
        ? describeEvent(response.event)
        : describeError(response.error?.code ?? "unknown_error"),
    );
  }

  useEffect(() => {
    if (!client || raiderCount === 0) {
      return;
    }

    const timer = window.setInterval(() => {
      const response = client.dispatch({ type: "advance_tick" });
      setSnapshot(response.snapshot);
      if (!response.ok) {
        setFeedback(describeError(response.error?.code ?? "unknown_error"));
        return;
      }
      if (
        response.event?.type === "tick_advanced" &&
        (response.event.kills > 0 || response.event.town_damage > 0)
      ) {
        setFeedback(describeEvent(response.event));
      }
    }, TICK_INTERVAL_MS);

    return () => window.clearInterval(timer);
  }, [client, raiderCount]);

  if (!snapshot) {
    return (
      <main className="loading-shell" data-testid="loading-state">
        <h1>Raid Defense</h1>
        <p>{feedback}</p>
      </main>
    );
  }

  const towerCount = snapshot.entities.filter((entity) => entity.kind === "tower").length;
  const townPercent = Math.max(
    0,
    Math.min(100, (snapshot.town_health / snapshot.town_max_health) * 100),
  );
  const canStartWave = raiderCount === 0 && snapshot.town_health > 0;

  return (
    <main className="game-shell" data-testid="raid-defense-game">
      <header className="top-bar">
        <div className="identity">
          <span className="eyebrow">Grid maul prototype</span>
          <h1>Raid Defense</h1>
        </div>

        <dl className="status-strip" aria-label="Battle status">
          <div>
            <dt>Gold</dt>
            <dd data-testid="gold-value">{snapshot.gold}</dd>
          </div>
          <div>
            <dt>Wave</dt>
            <dd data-testid="wave-value">{snapshot.wave}</dd>
          </div>
          <div>
            <dt>Raiders</dt>
            <dd data-testid="raider-count">{raiderCount}</dd>
          </div>
          <div>
            <dt>Towers</dt>
            <dd data-testid="tower-count">{towerCount}</dd>
          </div>
          <div>
            <dt>Tick</dt>
            <dd data-testid="tick-value">{snapshot.tick}</dd>
          </div>
        </dl>

        <button
          className="wave-button"
          type="button"
          disabled={!canStartWave}
          onClick={() => issue({ type: "start_wave" })}
          data-testid="start-wave"
        >
          Start wave
        </button>
      </header>

      <section className="battlefield" data-testid="world-3d" aria-label="3D defense grid">
        <GameWorld
          snapshot={snapshot}
          selectedCell={selectedCell}
          onSelectCell={setSelectedCell}
        />

        <div className="town-health" aria-label="Town health">
          <span>Town {snapshot.town_health}/{snapshot.town_max_health}</span>
          <span className="health-track" aria-hidden="true">
            <span style={{ width: `${townPercent}%` }} />
          </span>
        </div>
      </section>

      <section className="build-bar" aria-label="Build controls">
        <div className="cell-controls">
          <span className="eyebrow">Selected grid cell</span>
          <label>
            X
            <input
              data-testid="cell-x"
              type="number"
              min={0}
              max={snapshot.grid_width - 1}
              value={selectedCell.x}
              onChange={(event) =>
                setSelectedCell((current) => ({ ...current, x: Number(event.target.value) }))
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
                setSelectedCell((current) => ({ ...current, z: Number(event.target.value) }))
              }
            />
          </label>
        </div>

        <button
          className="build-button"
          type="button"
          disabled={snapshot.town_health === 0}
          onClick={() =>
            issue({ type: "place_tower", x: selectedCell.x, z: selectedCell.z })
          }
          data-testid="build-tower"
        >
          Build guard tower · {snapshot.tower_cost}g
        </button>

        <p className="feedback-line" role="status" aria-live="polite" data-testid="event-feedback">
          {feedback}
        </p>
      </section>

      <footer className="evidence-line">
        <span>Seed {snapshot.seed}</span>
        <span>Contract v{snapshot.contract_version}</span>
        <span data-testid="checksum">Checksum {snapshot.checksum}</span>
      </footer>
    </main>
  );
}

export default App;
