import { OrbitControls, Line } from "@react-three/drei";
import { Canvas, type ThreeEvent, useFrame } from "@react-three/fiber";
import { useRef } from "react";
import { CatmullRomCurve3, Color, MathUtils, Vector3, type Mesh } from "three";
import type {
  RuntimeBuildingView,
  RuntimeEntityView,
  RuntimeLogisticsRouteView,
  RuntimeMapLocation,
  RuntimeProvinceView,
  RuntimeRaidDefenseView,
  RuntimeRivalView,
} from "../simulationTypes";

type BattlefieldSceneProps = {
  view: RuntimeRaidDefenseView;
  provinces: RuntimeProvinceView[];
  rivals: RuntimeRivalView[];
  paused: boolean;
  placementKind: string | null;
  selectedBuildingId: number | null;
  selectedProvinceId: number | null;
  selectedTile: RuntimeMapLocation | null;
  onSelectBuilding: (buildingId: number) => void;
  onSelectProvince: (provinceId: number) => void;
  onSelectTile: (location: RuntimeMapLocation) => void;
};

const mapSize = 30;
const tileSize = 0.78;
const mapCenter = (mapSize - 1) / 2;
const mapWorldSize = mapSize * tileSize;

const toScenePoint = (location: RuntimeMapLocation) =>
  new Vector3((location.x - mapCenter) * tileSize, 0, (location.y - mapCenter) * tileSize);

const toMapLocation = (point: Vector3): RuntimeMapLocation => ({
  x: Math.max(0, Math.min(mapSize - 1, Math.round(point.x / tileSize + mapCenter))),
  y: Math.max(0, Math.min(mapSize - 1, Math.round(point.z / tileSize + mapCenter))),
  elevation: 0,
});

export function BattlefieldScene({
  view,
  provinces,
  rivals,
  paused,
  placementKind,
  selectedBuildingId,
  selectedProvinceId,
  selectedTile,
  onSelectBuilding,
  onSelectProvince,
  onSelectTile,
}: BattlefieldSceneProps) {
  const selectedBuildingRoutes =
    view.buildings.find((building) => building.id === selectedBuildingId)?.logistics?.routes ?? [];

  return (
    <Canvas
      camera={{ position: [5.5, 9.5, 12.5], fov: 42 }}
      className="absolute inset-0"
      gl={{ antialias: true }}
    >
      <color args={["#120d0b"]} attach="background" />
      <fog args={["#120d0b", 13, 30]} attach="fog" />
      <ambientLight intensity={0.7} />
      <hemisphereLight args={["#ffd9b5", "#29160f", 0.75]} />
      <directionalLight color="#ffddb1" intensity={1.2} position={[8, 12, 6]} />
      <Ground onSelectTile={onSelectTile} />

      {view.paths.map((path) => (
        <Line
          color="#f3c789"
          key={path.id}
          opacity={0.8}
          points={path.waypoints.map((waypoint) => toScenePoint(waypoint).setY(0.08))}
          transparent
        />
      ))}

      {selectedBuildingRoutes.map((route, index) => (
        <LogisticsRoute
          key={`${route.source_building_id}-${route.target_building_id}-${route.started_at_seconds}-${index}`}
          nowSeconds={view.now_seconds}
          paused={paused}
          route={route}
        />
      ))}

      {view.areas.map((area) =>
        area.tiles.map((tile, index) => (
          <mesh
            key={`${area.id}-${index}`}
            position={toScenePoint(tile).setY(0.015)}
            rotation={[-Math.PI / 2, 0, 0]}
          >
            <planeGeometry args={[tileSize * 0.72, tileSize * 0.72]} />
            <meshBasicMaterial
              color={area.kind === "heartland" ? "#5c4d2a" : "#583420"}
              opacity={0.48}
              transparent
            />
          </mesh>
        )),
      )}

      {view.buildings.map((building) => (
        <BuildingMarker
          building={building}
          key={building.id}
          selected={building.id === selectedBuildingId}
          onSelectBuilding={onSelectBuilding}
        />
      ))}

      {view.entities
        .filter((entity) => entity.kind === "basic_raider")
        .map((entity) => (
          <RaiderMarker entity={entity} key={entity.id} paused={paused} />
        ))}

      {provinces.map((province) => (
        <ProvinceMarker
          key={province.id}
          onSelectProvince={onSelectProvince}
          paused={paused}
          province={province}
          selected={province.id === selectedProvinceId}
        />
      ))}

      {rivals.map((rival, index) => {
        const target =
          provinces.find((province) => province.id === selectedProvinceId) ??
          provinces[Math.min(index, provinces.length - 1)];
        if (!target) {
          return null;
        }

        return (
          <RaidLane
            key={rival.id}
            origin={rival.location}
            paused={paused}
            target={target.location}
            intensity={rival.stats.aggression ?? 55}
          />
        );
      })}

      {(selectedTile || placementKind) && (
        <TileMarker
          armed={Boolean(placementKind)}
          location={selectedTile ?? { x: 15, y: 15, elevation: 0 }}
        />
      )}

      <OrbitControls
        enablePan={false}
        maxDistance={18}
        maxPolarAngle={Math.PI / 2.05}
        minDistance={8}
      />
    </Canvas>
  );
}

function Ground({ onSelectTile }: { onSelectTile: (location: RuntimeMapLocation) => void }) {
  const handlePointerDown = (event: ThreeEvent<PointerEvent>) => {
    event.stopPropagation();
    onSelectTile(toMapLocation(event.point));
  };

  return (
    <group>
      <mesh
        onPointerDown={handlePointerDown}
        position={[0, -0.14, 0]}
        receiveShadow
        rotation={[-Math.PI / 2, 0, 0]}
      >
        <planeGeometry args={[mapWorldSize, mapWorldSize]} />
        <meshStandardMaterial color="#251614" roughness={0.96} />
      </mesh>
      <gridHelper args={[mapWorldSize, mapSize, "#8b6048", "#3c2720"]} position={[0, -0.12, 0]} />
    </group>
  );
}

function LogisticsRoute({
  nowSeconds,
  paused,
  route,
}: {
  nowSeconds: number;
  paused: boolean;
  route: RuntimeLogisticsRouteView;
}) {
  const runnerRef = useRef<Mesh>(null);
  const waypoints = route.waypoints.map((waypoint) => toScenePoint(waypoint).setY(0.16));
  const drawable = waypoints.length >= 2;
  const routePoints = drawable ? waypoints : [new Vector3(), new Vector3()];
  const curve =
    routePoints.length >= 3
      ? new CatmullRomCurve3(routePoints)
      : new CatmullRomCurve3([
          routePoints[0],
          routePoints[0].clone().lerp(routePoints[1], 0.5).setY(0.34),
          routePoints[1],
        ]);
  const points = curve.getPoints(24);
  const durationSeconds = Math.max(1, route.completes_at_seconds - route.started_at_seconds);
  const initialProgress = MathUtils.clamp(
    (nowSeconds - route.started_at_seconds) / durationSeconds,
    0,
    1,
  );

  useFrame(({ clock }) => {
    if (paused || !runnerRef.current) {
      return;
    }

    const progress = (initialProgress + clock.elapsedTime * 0.08) % 1;
    runnerRef.current.position.copy(curve.getPointAt(progress));
  });

  if (!drawable) {
    return null;
  }

  return (
    <group>
      <Line color="#1b6f61" lineWidth={7} opacity={0.22} points={points} transparent />
      <Line color="#9ff5d0" lineWidth={3} opacity={0.92} points={points} transparent />
      <mesh ref={runnerRef}>
        <sphereGeometry args={[0.12, 18, 18]} />
        <meshStandardMaterial color="#d6ffe9" emissive="#59d6b2" emissiveIntensity={1.6} />
      </mesh>
      {waypoints.map((point, index) => (
        <mesh key={index} position={point}>
          <sphereGeometry args={[0.08, 14, 14]} />
          <meshStandardMaterial color="#bdfce2" emissive="#2fbf9a" emissiveIntensity={0.9} />
        </mesh>
      ))}
    </group>
  );
}

function BuildingMarker({
  building,
  selected,
  onSelectBuilding,
}: {
  building: RuntimeBuildingView;
  selected: boolean;
  onSelectBuilding: (buildingId: number) => void;
}) {
  const position = toScenePoint(building.location);
  const fill = building.kind === "castle" ? "#ffd08b" : "#9d6b47";
  const width = 0.28 + building.footprint.width * 0.24;
  const depth = 0.28 + building.footprint.depth * 0.24;

  return (
    <group
      onClick={(event) => {
        event.stopPropagation();
        onSelectBuilding(building.id);
      }}
      position={[position.x, 0, position.z]}
    >
      {selected && (
        <mesh position={[0, 0.02, 0]} rotation={[-Math.PI / 2, 0, 0]}>
          <planeGeometry args={[Math.max(width, depth) * 2.2, Math.max(width, depth) * 2.2]} />
          <meshBasicMaterial color="#ffe1a6" opacity={0.32} transparent />
        </mesh>
      )}
      <mesh position={[0, 0.32 + building.height * 0.06, 0]}>
        <boxGeometry args={[width, 0.64 + building.height * 0.12, depth]} />
        <meshStandardMaterial color={fill} emissive="#3b1f16" emissiveIntensity={0.45} />
      </mesh>
      <mesh position={[0, 0.06, 0]}>
        <boxGeometry args={[width + 0.12, 0.12, depth + 0.12]} />
        <meshStandardMaterial color="#2d1b16" roughness={0.92} />
      </mesh>
    </group>
  );
}

function ProvinceMarker({
  province,
  paused,
  selected,
  onSelectProvince,
}: {
  province: RuntimeProvinceView;
  paused: boolean;
  selected: boolean;
  onSelectProvince: (provinceId: number) => void;
}) {
  const pulseRef = useRef<Mesh>(null);
  const position = toScenePoint(province.location);
  const controlTint = MathUtils.clamp(province.control / 100, 0.25, 0.9);
  const baseColor = new Color().setRGB(0.42 + controlTint * 0.3, 0.18 + controlTint * 0.18, 0.1);

  useFrame(({ clock }) => {
    if (paused || !pulseRef.current) {
      return;
    }

    const pulse = 1 + Math.sin(clock.elapsedTime * 2.4) * 0.08;
    pulseRef.current.scale.setScalar(selected ? pulse : 1);
    const material = pulseRef.current.material as { opacity: number };
    material.opacity = selected ? 0.9 : 0.46;
  });

  return (
    <group
      onClick={(event) => {
        event.stopPropagation();
        onSelectProvince(province.id);
      }}
      position={[position.x, 0, position.z]}
    >
      <mesh position={[0, 0.15, 0]}>
        <cylinderGeometry args={[0.8, 1.08, 0.3, 4]} />
        <meshStandardMaterial color={baseColor} roughness={0.78} />
      </mesh>
      <mesh position={[0, 0.02, 0]} rotation={[-Math.PI / 2, 0, 0]} ref={pulseRef}>
        <ringGeometry args={[0.95, 1.15, 4]} />
        <meshBasicMaterial color={selected ? "#ffe1a6" : "#d37b53"} transparent />
      </mesh>
      <mesh position={[0, 0.55, 0]}>
        <sphereGeometry args={[0.14 + province.threat / 420, 24, 24]} />
        <meshStandardMaterial color="#ff8f5b" emissive="#8d2c1d" emissiveIntensity={1.2} />
      </mesh>
    </group>
  );
}

function RaiderMarker({ entity, paused }: { entity: RuntimeEntityView; paused: boolean }) {
  const markerRef = useRef<Mesh>(null);
  const position = toScenePoint(entity.location);

  useFrame(({ clock }) => {
    if (paused || !markerRef.current) {
      return;
    }

    markerRef.current.position.y = 0.42 + Math.sin(clock.elapsedTime * 5 + entity.id) * 0.06;
  });

  return (
    <group position={[position.x, 0, position.z]}>
      <mesh position={[0, 0.03, 0]} rotation={[-Math.PI / 2, 0, 0]}>
        <ringGeometry args={[0.2, 0.34, 4]} />
        <meshBasicMaterial color="#ff9d6b" opacity={0.85} transparent />
      </mesh>
      <mesh ref={markerRef}>
        <coneGeometry args={[0.18, 0.48, 4]} />
        <meshStandardMaterial color="#ffb27f" emissive="#a9341d" emissiveIntensity={1.2} />
      </mesh>
    </group>
  );
}

function TileMarker({
  armed,
  location,
}: {
  armed: boolean;
  location: RuntimeMapLocation;
}) {
  const position = toScenePoint(location);

  return (
    <group position={[position.x, 0, position.z]}>
      <mesh position={[0, 0.03, 0]} rotation={[-Math.PI / 2, 0, 0]}>
        <planeGeometry args={[tileSize * 0.88, tileSize * 0.88]} />
        <meshBasicMaterial color={armed ? "#ffb46e" : "#c0d4ff"} opacity={0.8} transparent />
      </mesh>
    </group>
  );
}

function RaidLane({
  origin,
  paused,
  target,
  intensity,
}: {
  origin: RuntimeMapLocation;
  paused: boolean;
  target: RuntimeMapLocation;
  intensity: number;
}) {
  const lineColor = intensity > 76 ? "#ff784b" : "#eab676";
  const runnerRefs = [useRef<Mesh>(null), useRef<Mesh>(null), useRef<Mesh>(null)];
  const curve = new CatmullRomCurve3([
    toScenePoint(origin).setY(0.12),
    toScenePoint(origin).clone().lerp(toScenePoint(target), 0.35).setY(0.85),
    toScenePoint(target).setY(0.18),
  ]);
  const points = curve.getPoints(32);

  useFrame(({ clock }) => {
    if (paused) {
      return;
    }

    runnerRefs.forEach((runnerRef, index) => {
      if (!runnerRef.current) {
        return;
      }

      const offset = (clock.elapsedTime * (0.06 + intensity / 1600) + index * 0.22) % 1;
      const point = curve.getPointAt(offset);
      runnerRef.current.position.copy(point);
    });
  });

  return (
    <group>
      <mesh position={toScenePoint(origin).setY(0.5)}>
        <coneGeometry args={[0.18, 0.65, 4]} />
        <meshStandardMaterial color="#d0583d" emissive="#7a2317" emissiveIntensity={0.95} />
      </mesh>

      <Line color={lineColor} opacity={0.68} points={points} transparent />

      {runnerRefs.map((runnerRef, index) => (
        <mesh key={index} ref={runnerRef}>
          <sphereGeometry args={[0.11, 18, 18]} />
          <meshStandardMaterial color="#ffd4a3" emissive="#ff8a5b" emissiveIntensity={1.4} />
        </mesh>
      ))}
    </group>
  );
}
