import { startTransition, useEffect, useRef, useState } from "react";
import { BattlefieldScene } from "./components/BattlefieldScene";
import {
  parseSimulationFileText,
  sanitizeSimulationFileName,
  serializeSimulationFile,
  simulationFileFromSaveState,
  toImportedSimulationSaveState,
  type SimulationJsonFile,
} from "./simulationSaveFormat";
import { RaidDefenseSimulationClient } from "./simulationClient";
import {
  provincesFromView,
  rivalsFromView,
  type RuntimeRaidDefenseCommandResponse,
  type RuntimeMapLocation,
  type RuntimeRaidDefenseView,
  type SimulationSaveState,
} from "./simulationTypes";

declare global {
  interface Window {
    __RAID_DEFENSE_E2E__?: {
      exportCurrentSimulation: () => SimulationJsonFile | null;
      currentView: () => RuntimeRaidDefenseView | null;
      applyCommand: (command: unknown) => RuntimeRaidDefenseCommandResponse | null;
      advanceSimulation: (deltaSeconds: number) => RuntimeRaidDefenseCommandResponse | null;
      loadSimulationFile: (input: SimulationJsonFile | string) => Promise<{
        name: string;
        now_seconds: number;
        seed: string;
      }>;
    };
  }
}

const numberFormatter = new Intl.NumberFormat("en-US");
const dateFormatter = new Intl.DateTimeFormat("en-US", {
  dateStyle: "medium",
  timeStyle: "short",
});

const simulationSaveStorageKey = "raid-defense.simulation.saves.v1";
const mainCampaignStorageKey = "raid-defense.main-campaign.v1";
const appSettingsStorageKey = "raid-defense.settings.v1";
const defaultSeed = 2106659303718057601n;

const severityStyles: Record<string, string> = {
  critical: "border-red-400/50 bg-red-500/12 text-red-100",
  warning: "border-amber-300/50 bg-amber-400/12 text-amber-50",
  info: "border-sky-300/40 bg-sky-400/10 text-sky-50",
};

const resourceAccent: Record<string, string> = {
  crowns: "from-amber-200/35 to-amber-600/20",
  food: "from-lime-200/30 to-lime-500/18",
  influence: "from-cyan-200/30 to-cyan-500/18",
  legion_strength: "from-red-200/30 to-red-500/18",
  stability: "from-orange-200/30 to-orange-500/18",
  wood: "from-orange-200/25 to-yellow-700/18",
  stone: "from-slate-200/20 to-slate-500/16",
  iron: "from-zinc-100/18 to-zinc-500/18",
};

type CostPart = {
  resource: string;
  amount: number;
};

const resourceNames: Record<string, string> = {
  crowns: "Crowns",
  food: "Food",
  wood: "Wood",
  stone: "Stone",
  iron: "Iron",
  influence: "Influence",
  legion_strength: "Legion Strength",
  stability: "Stability",
  intelligence: "Intelligence",
  trade_goods: "Trade Goods",
  citizens: "Citizens",
};

const buildPalette = [
  { kind: "farm", label: "Farm", note: "Converts frontier land into a stable food base." },
  {
    kind: "storage_house",
    label: "Storage House",
    note: "Buffers supplies so the frontier can absorb shocks.",
  },
  { kind: "tower", label: "Tower", note: "Adds local security and extends your line of defense." },
  {
    kind: "frontier_fort",
    label: "Frontier Fort",
    note: "Anchors expansion and can be turned into a province outpost.",
  },
  { kind: "barracks", label: "Barracks", note: "Supports military staffing and local order." },
  {
    kind: "senate_hall",
    label: "Senate Hall",
    note: "Strengthens influence and unlock pressure for civic growth.",
  },
] as const;

// Mirrors src/catalog.rs until construction costs are exposed directly in the view contract.
const buildingCosts: Record<string, Record<number, CostPart[]>> = {
  farm: {
    1: [{ resource: "wood", amount: 50 }],
  },
  storage_house: {
    1: [
      { resource: "wood", amount: 25 },
      { resource: "stone", amount: 20 },
    ],
    2: [
      { resource: "wood", amount: 25 },
      { resource: "stone", amount: 20 },
    ],
  },
  tower: {
    1: [
      { resource: "wood", amount: 50 },
      { resource: "stone", amount: 50 },
    ],
  },
  frontier_fort: {
    1: [
      { resource: "crowns", amount: 86 },
      { resource: "stone", amount: 34 },
      { resource: "wood", amount: 20 },
    ],
    2: [
      { resource: "crowns", amount: 100 },
      { resource: "stone", amount: 42 },
    ],
  },
  barracks: {
    1: [
      { resource: "crowns", amount: 48 },
      { resource: "wood", amount: 18 },
      { resource: "stone", amount: 10 },
    ],
    2: [
      { resource: "crowns", amount: 48 },
      { resource: "wood", amount: 18 },
      { resource: "stone", amount: 10 },
    ],
  },
  senate_hall: {
    1: [
      { resource: "crowns", amount: 70 },
      { resource: "stone", amount: 28 },
    ],
    2: [
      { resource: "crowns", amount: 70 },
      { resource: "stone", amount: 28 },
    ],
  },
};

const resourcePacks = [
  { resource: "wood", amount: 100, label: "+100 Wood" },
  { resource: "stone", amount: 100, label: "+100 Stone" },
  { resource: "food", amount: 100, label: "+100 Food" },
  { resource: "crowns", amount: 120, label: "+120 Crowns" },
  { resource: "influence", amount: 40, label: "+40 Influence" },
  { resource: "iron", amount: 40, label: "+40 Iron" },
  { resource: "legion_strength", amount: 18, label: "+18 Legions" },
  { resource: "stability", amount: 24, label: "+24 Stability" },
] as const;

const timeSteps = [
  { seconds: 30, label: "+30s" },
  { seconds: 300, label: "+5m" },
  { seconds: 1800, label: "+30m" },
] as const;

const recruitableUnits = [
  { kind: "worker", label: "Recruit Worker" },
  { kind: "prefect", label: "Recruit Prefect" },
  { kind: "legate", label: "Recruit Legate" },
] as const;

const unitIntel: Record<string, string> = {
  worker: "Workers keep construction, hauling, and food consumption moving through the settlement core.",
  prefect: "Prefects anchor civic production and improve how well frontier administration keeps pace.",
  legate: "Legates staff military buildings and turn iron and food into usable legion strength.",
  envoy: "Envoys push diplomacy and expansion once the campaign reaches its second rank.",
  basic_raider: "Raiders probe for weak storage, break blockers, loot what they can carry, and then retreat.",
};

type Screen = "home" | "main" | "simulations" | "settings" | "wiki";
type GameMode = "main" | "simulation";

type CampaignSaveState = {
  seed: string;
  updated_at: string;
  now_seconds: number;
  snapshot_json: string;
};

type AppSettings = {
  defaultMainSeed: string;
  defaultSimulationSeed: string;
  showAmbientOverlay: boolean;
  showNotifications: boolean;
};

const defaultSettings: AppSettings = {
  defaultMainSeed: defaultSeed.toString(),
  defaultSimulationSeed: defaultSeed.toString(),
  showAmbientOverlay: true,
  showNotifications: true,
};

export default function App() {
  const [screen, setScreen] = useState<Screen>("home");
  const [activeMode, setActiveMode] = useState<GameMode | null>(null);
  const [pendingMode, setPendingMode] = useState<GameMode | null>(null);
  const [client, setClient] = useState<RaidDefenseSimulationClient | null>(null);
  const [view, setView] = useState<RuntimeRaidDefenseView | null>(null);
  const [loading, setLoading] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [seedInput, setSeedInput] = useState(defaultSeed.toString());
  const [saveName, setSaveName] = useState("");
  const [saveStates, setSaveStates] = useState<SimulationSaveState[]>(() => loadSaveStates());
  const [mainCampaignSave, setMainCampaignSave] = useState<CampaignSaveState | null>(() =>
    loadMainCampaignSave(),
  );
  const [settings, setSettings] = useState<AppSettings>(() => loadAppSettings());
  const [selectedProvinceId, setSelectedProvinceId] = useState<number | null>(null);
  const [selectedBuildingId, setSelectedBuildingId] = useState<number | null>(null);
  const [selectedTile, setSelectedTile] = useState<RuntimeMapLocation | null>(null);
  const [placementKind, setPlacementKind] = useState<string | null>(null);
  const simulationImportInputRef = useRef<HTMLInputElement | null>(null);
  const clientRef = useRef<RaidDefenseSimulationClient | null>(null);
  const viewRef = useRef<RuntimeRaidDefenseView | null>(null);
  const activeModeRef = useRef<GameMode | null>(null);

  const provinces = view ? provincesFromView(view) : [];
  const rivals = view ? rivalsFromView(view) : [];
  const trackedBuildings = view?.buildings.filter((building) => hasTrackedBuildCost(building.kind)) ?? [];
  const currentBuildCost = sumCosts(
    trackedBuildings.map((building) => constructionCostThroughLevel(building.kind, building.level)),
  );

  useEffect(() => {
    window.localStorage.setItem(simulationSaveStorageKey, JSON.stringify(saveStates));
  }, [saveStates]);

  useEffect(() => {
    window.localStorage.setItem(appSettingsStorageKey, JSON.stringify(settings));
  }, [settings]);

  useEffect(() => {
    if (!mainCampaignSave) {
      window.localStorage.removeItem(mainCampaignStorageKey);
      return;
    }

    window.localStorage.setItem(mainCampaignStorageKey, JSON.stringify(mainCampaignSave));
  }, [mainCampaignSave]);

  useEffect(() => {
    if (!view) {
      return;
    }

    startTransition(() => {
      setSelectedProvinceId((current) =>
        provinces.some((province) => province.id === current) ? current : (provinces[0]?.id ?? null),
      );
      setSelectedBuildingId((current) =>
        view.buildings.some((building) => building.id === current)
          ? current
          : (view.buildings[0]?.id ?? null),
      );
    });
  }, [provinces, view]);

  useEffect(() => {
    clientRef.current = client;
    viewRef.current = view;
    activeModeRef.current = activeMode;
  }, [activeMode, client, view]);

  useEffect(() => {
    if (typeof window === "undefined") {
      return;
    }

    window.__RAID_DEFENSE_E2E__ = {
      exportCurrentSimulation: () => {
        if (activeModeRef.current !== "simulation") {
          return null;
        }

        const bridgeClient = clientRef.current;
        const bridgeView = viewRef.current;
        if (!bridgeClient || !bridgeView) {
          return null;
        }

        return simulationFileFromSaveState({
          name: "E2E Snapshot",
          seed: bridgeClient.seed.toString(),
          now_seconds: bridgeView.now_seconds,
          snapshot_json: bridgeClient.saveSnapshot(),
        });
      },
      currentView: () => {
        if (activeModeRef.current !== "simulation") {
          return null;
        }

        return viewRef.current;
      },
      applyCommand: (command) => {
        const bridgeClient = clientRef.current;
        if (activeModeRef.current !== "simulation" || !bridgeClient) {
          return null;
        }

        const response = bridgeClient.apply(command);
        if (response.accepted) {
          viewRef.current = response.view;
          startTransition(() => {
            setView(response.view);
          });
        }
        return response;
      },
      advanceSimulation: (deltaSeconds) => {
        const bridgeClient = clientRef.current;
        if (activeModeRef.current !== "simulation" || !bridgeClient) {
          return null;
        }

        const response = bridgeClient.advance(deltaSeconds);
        if (response.accepted) {
          viewRef.current = response.view;
          startTransition(() => {
            setView(response.view);
          });
        }
        return response;
      },
      loadSimulationFile: async (input) => {
        const imported =
          typeof input === "string"
            ? parseSimulationFileText(input)
            : parseSimulationFileText(JSON.stringify(input));
        const saveState = toImportedSimulationSaveState(imported, createSaveId);
        rememberSimulationSaveState(saveState);
        await bootMode({
          mode: "simulation",
          seed: parseSeedOrDefault(imported.seed, defaultSeed),
          snapshotJson: imported.snapshot_json,
          successNotice: `Loaded ${imported.name}.`,
        });
        return {
          name: imported.name,
          now_seconds: imported.now_seconds,
          seed: imported.seed,
        };
      },
    };

    return () => {
      delete window.__RAID_DEFENSE_E2E__;
    };
  }, [settings.showNotifications]);

  useEffect(() => {
    if (activeMode !== "main" || !client || !view) {
      return;
    }

    const snapshot_json = client.saveSnapshot();
    setMainCampaignSave((current) => {
      if (
        current?.seed === client.seed.toString() &&
        current?.now_seconds === view.now_seconds &&
        current.snapshot_json === snapshot_json
      ) {
        return current;
      }

      return {
        seed: client.seed.toString(),
        updated_at: new Date().toISOString(),
        now_seconds: view.now_seconds,
        snapshot_json,
      };
    });
  }, [activeMode, client, view]);

  const selectedProvince =
    provinces.find((province) => province.id === selectedProvinceId) ?? provinces[0] ?? null;
  const selectedBuilding =
    view?.buildings.find((building) => building.id === selectedBuildingId) ?? null;
  const claimableProvince =
    selectedBuilding &&
    (selectedBuilding.kind === "frontier_fort" || selectedBuilding.kind === "embassy") &&
    !provinces.some((province) => province.stats.outpost_id === selectedBuilding.id);
  const castle = view?.buildings.find((building) => building.kind === "castle") ?? null;

  function createSaveId() {
    return typeof crypto !== "undefined" && "randomUUID" in crypto
      ? crypto.randomUUID()
      : `${Date.now()}`;
  }

  function buildSimulationSaveState(name: string): SimulationSaveState | null {
    if (!client || !view) {
      return null;
    }

    return {
      id: createSaveId(),
      name,
      seed: client.seed.toString(),
      created_at: new Date().toISOString(),
      now_seconds: view.now_seconds,
      snapshot_json: client.saveSnapshot(),
    };
  }

  function rememberSimulationSaveState(nextSave: SimulationSaveState) {
    startTransition(() => {
      setSaveStates((current) => {
        const duplicateIndex = current.findIndex(
          (saveState) =>
            saveState.seed === nextSave.seed &&
            saveState.now_seconds === nextSave.now_seconds &&
            saveState.snapshot_json === nextSave.snapshot_json,
        );
        if (duplicateIndex === 0) {
          return current;
        }
        if (duplicateIndex > 0) {
          const existing = current[duplicateIndex];
          return [
            {
              ...existing,
              name: nextSave.name,
              created_at: nextSave.created_at,
            },
            ...current.filter((_, index) => index !== duplicateIndex),
          ];
        }
        return [nextSave, ...current];
      });
    });
  }

  async function bootMode(options: {
    mode: GameMode;
    seed: bigint;
    snapshotJson?: string;
    successNotice: string;
  }) {
    setLoading(true);
    setBusy(true);
    setPendingMode(options.mode);
    setError(null);

    try {
      const nextClient = await RaidDefenseSimulationClient.create(options.seed);
      let nextView = nextClient.view();

      if (options.snapshotJson) {
        const response = nextClient.loadSnapshot(options.snapshotJson);
        if (!response.accepted) {
          throw new Error(response.error ?? "Could not load the saved state.");
        }
        nextView = response.view;
      }

      clientRef.current = nextClient;
      viewRef.current = nextView;
      activeModeRef.current = options.mode;
      startTransition(() => {
        setClient(nextClient);
        setView(nextView);
        setActiveMode(options.mode);
        setScreen(options.mode === "main" ? "main" : "simulations");
        setSeedInput(options.seed.toString());
        setSelectedTile(null);
        setPlacementKind(null);
      });
      setNotice(settings.showNotifications ? options.successNotice : null);
    } catch (nextError) {
      setScreen("home");
      setError(getErrorMessage(nextError));
    } finally {
      setBusy(false);
      setLoading(false);
      setPendingMode(null);
    }
  }

  async function openMainGame() {
    if (activeMode === "main" && client && view) {
      setScreen("main");
      return;
    }

    if (mainCampaignSave) {
      const seed = parseSeedOrDefault(mainCampaignSave.seed, defaultSeed);
      await bootMode({
        mode: "main",
        seed,
        snapshotJson: mainCampaignSave.snapshot_json,
        successNotice: "Main campaign continued from the latest autosave.",
      });
      return;
    }

    await bootMode({
      mode: "main",
      seed: parseSeedOrDefault(settings.defaultMainSeed, defaultSeed),
      successNotice: "Main campaign started.",
    });
  }

  async function startFreshMainGame() {
    setMainCampaignSave(null);
    await bootMode({
      mode: "main",
      seed: parseSeedOrDefault(settings.defaultMainSeed, defaultSeed),
      successNotice: "Fresh main campaign started.",
    });
  }

  async function openSimulations() {
    if (activeMode === "simulation" && client && view) {
      setScreen("simulations");
      return;
    }

    await bootMode({
      mode: "simulation",
      seed: parseSeedOrDefault(settings.defaultSimulationSeed, defaultSeed),
      successNotice: "Simulation lab opened.",
    });
  }

  async function applyResponse(
    action: () => RuntimeRaidDefenseCommandResponse | Promise<RuntimeRaidDefenseCommandResponse>,
    successNotice: string,
  ) {
    setBusy(true);
    setError(null);

    try {
      const response = await action();
      if (!response.accepted) {
        throw new Error(response.error ?? "Command rejected.");
      }
      viewRef.current = response.view;
      startTransition(() => {
        setView(response.view);
      });
      setNotice(settings.showNotifications ? successNotice : null);
    } catch (nextError) {
      setError(getErrorMessage(nextError));
    } finally {
      setBusy(false);
    }
  }

  async function resetSimulation() {
    const normalizedSeed = seedInput.trim();
    if (!normalizedSeed) {
      setError("Enter a valid non-negative seed.");
      return;
    }

    try {
      const seed = BigInt(normalizedSeed);
      if (seed < 0n) {
        throw new Error("negative");
      }

      await bootMode({
        mode: "simulation",
        seed,
        successNotice: `Simulation ready on seed ${seed.toString()}.`,
      });
    } catch {
      setError("Enter a valid non-negative seed.");
    }
  }

  function downloadSimulationJson(saveState: SimulationSaveState) {
    const json = serializeSimulationFile(saveState);
    const fileName = `${sanitizeSimulationFileName(saveState.name)}.json`;
    const blob = new Blob([json], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = fileName;
    link.click();
    URL.revokeObjectURL(url);
    setNotice(settings.showNotifications ? `Exported ${saveState.name}.` : null);
  }

  async function importSimulationFile(file: File) {
    setError(null);
    const imported = parseSimulationFileText(await file.text());
    const saveState = toImportedSimulationSaveState(imported, createSaveId);
    rememberSimulationSaveState(saveState);
    await bootMode({
      mode: "simulation",
      seed: parseSeedOrDefault(imported.seed, defaultSeed),
      snapshotJson: imported.snapshot_json,
      successNotice: `Loaded ${imported.name}.`,
    });
  }

  function saveCurrentState() {
    const name = saveName.trim() || `Snapshot ${saveStates.length + 1}`;
    const nextSave = buildSimulationSaveState(name);
    if (!nextSave) {
      return;
    }

    rememberSimulationSaveState(nextSave);
    startTransition(() => {
      setSaveName("");
    });
    setNotice(settings.showNotifications ? `Saved ${name}.` : null);
  }

  async function loadSaveState(saveState: SimulationSaveState) {
    await bootMode({
      mode: "simulation",
      seed: parseSeedOrDefault(saveState.seed, defaultSeed),
      snapshotJson: saveState.snapshot_json,
      successNotice: `Loaded ${saveState.name}.`,
    });
  }

  function deleteSaveState(saveId: string) {
    startTransition(() => {
      setSaveStates((current) => current.filter((saveState) => saveState.id !== saveId));
    });
  }

  function clearSimulationSaves() {
    setSaveStates([]);
    setNotice(settings.showNotifications ? "Simulation snapshots cleared." : null);
  }

  function clearMainCampaign() {
    setMainCampaignSave(null);
    if (activeMode === "main") {
      activeModeRef.current = null;
      clientRef.current = null;
      viewRef.current = null;
      setActiveMode(null);
      setClient(null);
      setView(null);
      setScreen("home");
    }
    setNotice(settings.showNotifications ? "Main campaign autosave cleared." : null);
  }

  function selectProvince(provinceId: number) {
    startTransition(() => {
      setSelectedProvinceId(provinceId);
    });
  }

  function selectBuilding(buildingId: number) {
    startTransition(() => {
      setSelectedBuildingId(buildingId);
    });
  }

  function selectTile(location: RuntimeMapLocation) {
    startTransition(() => {
      setSelectedTile(location);
    });

    if (!placementKind || !client) {
      return;
    }

    const buildLabel =
      buildPalette.find((building) => building.kind === placementKind)?.label ?? placementKind;
    void applyResponse(
      () =>
        client.apply({
          ConstructBuilding: {
            kind: placementKind,
            location,
          },
        }),
      `${buildLabel} queued at ${location.x}, ${location.y}.`,
    );
  }

  function sendRaider(location: RuntimeMapLocation) {
    if (!client) {
      return;
    }

    void applyResponse(
      () =>
        client.apply({
          SpawnEntity: {
            blueprint: { Unit: "basic_raider" },
            name: null,
            location,
          },
        }),
      `Raider sent from ${location.x}, ${location.y}.`,
    );
  }

  function updateSetting<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
    setSettings((current) => ({
      ...current,
      [key]: value,
    }));
  }

  if (loading) {
    return <LoadingScreen error={error} label={pendingMode === "main" ? "Campaign" : "Simulation"} />;
  }

  if (screen === "home") {
    return (
      <HomeScreen
        activeMode={activeMode}
        error={error}
        mainCampaignSave={mainCampaignSave}
        notice={notice}
        onOpenSettings={() => setScreen("settings")}
        onOpenSimulations={() => {
          void openSimulations();
        }}
        onOpenWiki={() => setScreen("wiki")}
        onStartFreshMainGame={() => {
          void startFreshMainGame();
        }}
        onStartMainGame={() => {
          void openMainGame();
        }}
        saveStates={saveStates}
      />
    );
  }

  if (screen === "settings") {
    return (
      <ShellFrame subtitle="Adjust campaign defaults, interface polish, and stored progress.">
        <section className="grid gap-4 lg:grid-cols-[1.15fr_0.85fr]">
          <div className="rounded-[2rem] border border-white/10 bg-black/30 p-6 backdrop-blur-xl">
            <p className="text-[0.72rem] tracking-[0.3em] text-orange-200/75 uppercase">Settings</p>
            <h1
              className="mt-3 text-4xl font-black tracking-[0.06em] text-stone-50 uppercase"
              style={{ fontFamily: '"Syne", sans-serif' }}
            >
              Control Room
            </h1>
            <div className="mt-6 grid gap-5">
              <SettingInput
                description="Used when you start a fresh main campaign."
                label="Default Main Game Seed"
                onChange={(value) => updateSetting("defaultMainSeed", value)}
                value={settings.defaultMainSeed}
              />
              <SettingInput
                description="Used when you open the simulation lab without loading a snapshot."
                label="Default Simulation Seed"
                onChange={(value) => updateSetting("defaultSimulationSeed", value)}
                value={settings.defaultSimulationSeed}
              />
              <ToggleCard
                checked={settings.showAmbientOverlay}
                description="Keeps the atmospheric glow and vignette above the battlefield."
                label="Ambient Overlay"
                onToggle={() => updateSetting("showAmbientOverlay", !settings.showAmbientOverlay)}
              />
              <ToggleCard
                checked={settings.showNotifications}
                description="Shows success notices after accepted actions and save operations."
                label="Status Notifications"
                onToggle={() => updateSetting("showNotifications", !settings.showNotifications)}
              />
            </div>
          </div>

          <div className="grid gap-4">
            <section className="rounded-[2rem] border border-white/10 bg-black/30 p-6 backdrop-blur-xl">
              <p className="text-[0.72rem] tracking-[0.3em] text-orange-200/75 uppercase">Storage</p>
              <div className="mt-5 grid gap-3">
                <InfoRow
                  label="Main Campaign"
                  value={
                    mainCampaignSave
                      ? `Autosaved at ${formatDateTime(mainCampaignSave.updated_at)}`
                      : "No autosave present"
                  }
                />
                <InfoRow label="Simulation Snapshots" value={`${saveStates.length} saved`} />
              </div>
              <div className="mt-5 grid gap-3 sm:grid-cols-2">
                <button
                  className="rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-sm text-stone-100 transition hover:border-white/20 hover:bg-white/8"
                  onClick={clearMainCampaign}
                  type="button"
                >
                  Clear Main Campaign
                </button>
                <button
                  className="rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-sm text-stone-100 transition hover:border-white/20 hover:bg-white/8"
                  onClick={clearSimulationSaves}
                  type="button"
                >
                  Clear Simulation Saves
                </button>
              </div>
            </section>

            <section className="rounded-[2rem] border border-white/10 bg-black/30 p-6 backdrop-blur-xl">
              <p className="text-[0.72rem] tracking-[0.3em] text-orange-200/75 uppercase">Navigation</p>
              <div className="mt-5 grid gap-3 sm:grid-cols-2">
                <button
                  className="rounded-2xl border border-orange-200/20 bg-orange-300/12 px-4 py-3 text-sm font-semibold tracking-[0.14em] text-orange-50 uppercase transition hover:bg-orange-300/18"
                  onClick={() => setScreen("home")}
                  type="button"
                >
                  Back Home
                </button>
                <button
                  className="rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-sm text-stone-100 transition hover:border-white/20 hover:bg-white/8"
                  onClick={() => setScreen("wiki")}
                  type="button"
                >
                  Open Wiki
                </button>
              </div>
            </section>
          </div>
        </section>
      </ShellFrame>
    );
  }

  if (screen === "wiki") {
    const encounteredUnits = view?.encountered_units ?? [];
    const encounteredAttackWaves = view?.encountered_attack_waves ?? [];

    return (
      <ShellFrame subtitle="The field manual only records units and raid patterns the current frontier has already encountered.">
        <section className="rounded-[2rem] border border-white/10 bg-black/30 p-6 backdrop-blur-xl">
          <p className="text-[0.72rem] tracking-[0.3em] text-orange-200/75 uppercase">Wiki</p>
          <h1
            className="mt-3 text-4xl font-black tracking-[0.06em] text-stone-50 uppercase"
            style={{ fontFamily: '"Syne", sans-serif' }}
          >
            Field Manual
          </h1>
          <p className="mt-4 max-w-3xl text-sm leading-6 text-stone-300">
            Entries unlock from live campaign state. Recruiting a unit logs it here, and every
            raider incursion adds a wave record with its entry point and composition.
          </p>
        </section>

        {!view ? (
          <section className="mt-4">
            <WikiPanel
              description="The wiki is driven by the active frontier state, not by the shell alone."
              title="No Active Frontier"
            >
              <p className="text-sm leading-6 text-stone-300">
                Open a campaign or simulation first. Once a run is active, the manual will show
                only the units and attack waves already seen in that frontier.
              </p>
            </WikiPanel>
          </section>
        ) : (
          <section className="mt-4 grid gap-4 lg:grid-cols-[1.1fr_0.9fr]">
            <WikiPanel description="Only units already fielded or encountered are recorded here." title="Units">
              {encounteredUnits.length === 0 ? (
                <WikiEmptyState text="No units have been logged yet." />
              ) : (
                <div className="grid gap-3 sm:grid-cols-2">
                  {encounteredUnits.map((unit) => (
                    <article className="rounded-[1.4rem] border border-white/10 bg-white/5 p-4" key={unit.kind}>
                      <div className="flex items-start justify-between gap-3">
                        <p className="text-sm font-semibold tracking-[0.14em] text-stone-50 uppercase">
                          {unit.label}
                        </p>
                        <span className="rounded-full border border-white/10 bg-white/5 px-3 py-1 text-[0.68rem] tracking-[0.18em] text-orange-100/75 uppercase">
                          {unit.current_count > 0 ? `${formatValue(unit.current_count)} active` : "seen"}
                        </span>
                      </div>
                      <p className="mt-2 text-[0.72rem] tracking-[0.18em] text-orange-100/70 uppercase">
                        First encountered at {formatTick(unit.encountered_at_seconds)}
                      </p>
                      <p className="mt-3 text-sm leading-6 text-stone-300">
                        {unitIntel[unit.kind] ??
                          `${unit.label} is now part of the current frontier record.`}
                      </p>
                    </article>
                  ))}
                </div>
              )}
            </WikiPanel>

            <WikiPanel
              description="Raider incursions are written down when they first breach the frontier."
              title="Attack Waves"
            >
              {encounteredAttackWaves.length === 0 ? (
                <WikiEmptyState text="No attack waves have been recorded yet." />
              ) : (
                <div className="grid gap-3">
                  {encounteredAttackWaves.map((wave) => (
                    <article className="rounded-[1.4rem] border border-white/10 bg-white/5 p-4" key={wave.id}>
                      <div className="flex items-start justify-between gap-3">
                        <p className="text-sm font-semibold tracking-[0.14em] text-stone-50 uppercase">
                          {wave.label}
                        </p>
                        <span className="rounded-full border border-white/10 bg-white/5 px-3 py-1 text-[0.68rem] tracking-[0.18em] text-orange-100/75 uppercase">
                          {formatWaveComposition(wave.units)}
                        </span>
                      </div>
                      <p className="mt-2 text-[0.72rem] tracking-[0.18em] text-orange-100/70 uppercase">
                        First seen at {formatTick(wave.encountered_at_seconds)} from {wave.entry.x},{" "}
                        {wave.entry.y}
                      </p>
                      <p className="mt-3 text-sm leading-6 text-stone-300">
                        {describeAttackWave(wave.units)}
                      </p>
                    </article>
                  ))}
                </div>
              )}
            </WikiPanel>
          </section>
        )}
      </ShellFrame>
    );
  }

  if (!view) {
    return (
      <ShellFrame subtitle="No active engine session is loaded right now.">
        <section className="rounded-[2rem] border border-white/10 bg-black/30 p-6 text-center backdrop-blur-xl">
          <p className="text-sm text-stone-300">{error ?? "Open a game mode from the home screen."}</p>
          <button
            className="mt-5 rounded-2xl border border-orange-200/20 bg-orange-300/12 px-5 py-3 text-sm font-semibold tracking-[0.14em] text-orange-50 uppercase transition hover:bg-orange-300/18"
            onClick={() => setScreen("home")}
            type="button"
          >
            Back Home
          </button>
        </section>
      </ShellFrame>
    );
  }

  const isSimulationMode = screen === "simulations";

  return (
    <div className="relative min-h-screen overflow-hidden bg-[#120d0b] text-stone-100">
      <BattlefieldScene
        onSelectBuilding={selectBuilding}
        onSelectProvince={selectProvince}
        onSelectTile={selectTile}
        placementKind={placementKind}
        provinces={provinces}
        rivals={rivals}
        selectedBuildingId={selectedBuildingId}
        selectedProvinceId={selectedProvinceId}
        selectedTile={selectedTile}
        view={view}
      />

      {settings.showAmbientOverlay && (
        <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_top,rgba(255,183,126,0.12),transparent_32%),linear-gradient(180deg,rgba(14,8,8,0.08),rgba(14,8,8,0.72))]" />
      )}

      <div className="relative z-10 flex min-h-screen flex-col">
        <header className="px-4 pt-4 pb-3 sm:px-6 lg:px-8">
          <div className="grid gap-4 lg:grid-cols-[1.25fr_1fr]">
            <div className="rounded-[2rem] border border-white/10 bg-black/30 px-5 py-5 shadow-[0_24px_80px_rgba(0,0,0,0.35)] backdrop-blur-xl">
              <div className="flex flex-wrap items-start justify-between gap-4">
                <div>
                  <p className="text-[0.72rem] font-semibold tracking-[0.38em] text-orange-200/80 uppercase">
                    Mode
                  </p>
                  <div className="mt-3 inline-flex items-center gap-3 rounded-full border border-orange-200/20 bg-orange-300/10 px-4 py-2">
                    <span className="h-2.5 w-2.5 rounded-full bg-orange-200" />
                    <span className="text-sm font-semibold tracking-[0.22em] text-orange-50 uppercase">
                      {isSimulationMode ? "Simulation Lab" : "Main Campaign"}
                    </span>
                  </div>
                  <h1
                    className="mt-4 text-4xl font-black tracking-[0.06em] text-stone-50 uppercase sm:text-5xl"
                    style={{ fontFamily: '"Syne", sans-serif' }}
                  >
                    Raid Defense
                  </h1>
                  <p className="mt-3 max-w-2xl text-sm text-stone-300 sm:text-base">
                    {isSimulationMode
                      ? "Experiment with layouts, advance time, and branch the run into reusable snapshots."
                      : "Hold the frontier together, grow provinces, and push the main campaign forward one autosaved decision at a time."}
                  </p>
                </div>

                <div className="flex flex-wrap justify-end gap-2">
                  <NavButton label="Home" onClick={() => setScreen("home")} />
                  <NavButton label="Settings" onClick={() => setScreen("settings")} />
                  <NavButton label="Wiki" onClick={() => setScreen("wiki")} />
                </div>
              </div>
            </div>

            <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
              {view.resources.map((resource) => (
                <div
                  className={`rounded-[1.6rem] border border-white/10 bg-gradient-to-br ${resourceAccent[resource.id] ?? "from-white/10 to-white/0"} px-4 py-4 shadow-[0_16px_40px_rgba(0,0,0,0.24)] backdrop-blur-xl`}
                  key={resource.id}
                >
                  <p className="text-[0.68rem] tracking-[0.28em] text-stone-300 uppercase">
                    {resource.label}
                  </p>
                  <p className="mt-3 text-2xl font-semibold text-stone-50">
                    {formatValue(resource.amount)}
                  </p>
                  <p className="mt-2 text-xs text-stone-300/80">
                    {resource.capacity ? `Cap ${formatValue(resource.capacity)}` : "Uncapped"}
                  </p>
                </div>
              ))}
            </div>
          </div>
        </header>

        <main className="grid flex-1 gap-4 px-4 pb-4 sm:px-6 lg:grid-cols-[340px_minmax(0,1fr)_360px] lg:px-8">
          <aside className="flex flex-col gap-4">
            {isSimulationMode ? (
              <>
                <section className="rounded-[1.8rem] border border-white/10 bg-black/30 p-5 backdrop-blur-xl">
                  <div className="flex items-center justify-between gap-4">
                    <div>
                      <p className="text-[0.72rem] tracking-[0.28em] text-orange-200/80 uppercase">
                        Simulation State
                      </p>
                      <p className="mt-2 text-sm text-stone-300">
                        Seeded runs with unlimited snapshots.
                      </p>
                    </div>
                    <button
                      className="rounded-full border border-white/10 bg-white/6 px-4 py-2 text-xs tracking-[0.2em] text-stone-200 uppercase transition hover:border-white/20 hover:bg-white/10 disabled:cursor-not-allowed disabled:opacity-60"
                      disabled={busy}
                      onClick={() => {
                        void resetSimulation();
                      }}
                      type="button"
                    >
                      Reset
                    </button>
                  </div>

                  <div className="mt-4 flex gap-2">
                    <input
                      className="min-w-0 flex-1 rounded-2xl border border-white/10 bg-black/25 px-4 py-3 text-sm text-stone-100 outline-none transition focus:border-orange-200/40"
                      onChange={(event) => setSeedInput(event.target.value)}
                      placeholder="Seed"
                      value={seedInput}
                    />
                    <button
                      className="rounded-2xl border border-orange-200/20 bg-orange-300/12 px-4 py-3 text-xs font-semibold tracking-[0.18em] text-orange-50 uppercase transition hover:bg-orange-300/18 disabled:cursor-not-allowed disabled:opacity-60"
                      disabled={busy}
                      onClick={() => {
                        void resetSimulation();
                      }}
                      type="button"
                    >
                      Apply
                    </button>
                  </div>

                  <div className="mt-5 flex gap-2">
                    <input
                      className="min-w-0 flex-1 rounded-2xl border border-white/10 bg-black/25 px-4 py-3 text-sm text-stone-100 outline-none transition focus:border-orange-200/40"
                      data-testid="simulation-save-name"
                      onChange={(event) => setSaveName(event.target.value)}
                      placeholder="Snapshot name"
                      value={saveName}
                    />
                    <button
                      className="rounded-2xl border border-orange-200/20 bg-orange-300/12 px-4 py-3 text-xs font-semibold tracking-[0.18em] text-orange-50 uppercase transition hover:bg-orange-300/18 disabled:cursor-not-allowed disabled:opacity-60"
                      data-testid="simulation-save-current"
                      disabled={busy}
                      onClick={saveCurrentState}
                      type="button"
                    >
                      Save
                    </button>
                  </div>

                  <div className="mt-3 grid gap-2 sm:grid-cols-2">
                    <button
                      className="rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-xs font-semibold tracking-[0.18em] text-stone-100 uppercase transition hover:border-white/20 hover:bg-white/8 disabled:cursor-not-allowed disabled:opacity-60"
                      data-testid="simulation-export-current"
                      disabled={busy || !client || !view}
                      onClick={() => {
                        const nextSave = buildSimulationSaveState(saveName.trim() || "Simulation Snapshot");
                        if (!nextSave) {
                          return;
                        }
                        downloadSimulationJson(nextSave);
                      }}
                      type="button"
                    >
                      Export JSON
                    </button>
                    <button
                      className="rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-xs font-semibold tracking-[0.18em] text-stone-100 uppercase transition hover:border-white/20 hover:bg-white/8 disabled:cursor-not-allowed disabled:opacity-60"
                      data-testid="simulation-import-button"
                      disabled={busy}
                      onClick={() => simulationImportInputRef.current?.click()}
                      type="button"
                    >
                      Import JSON
                    </button>
                    <input
                      accept="application/json,.json"
                      className="hidden"
                      data-testid="simulation-import-input"
                      onChange={(event) => {
                        const file = event.target.files?.[0];
                        event.target.value = "";
                        if (!file) {
                          return;
                        }
                        void importSimulationFile(file).catch((nextError) => {
                          setError(getErrorMessage(nextError));
                        });
                      }}
                      ref={simulationImportInputRef}
                      type="file"
                    />
                  </div>

                  <div className="mt-4 grid max-h-60 gap-3 overflow-y-auto pr-1">
                    {saveStates.length === 0 && (
                      <div className="rounded-[1.4rem] border border-dashed border-white/10 bg-white/4 px-4 py-4 text-sm text-stone-400">
                        No simulation save states yet.
                      </div>
                    )}

                    {saveStates.map((saveState) => (
                      <article
                        className="rounded-[1.4rem] border border-white/10 bg-white/4 px-4 py-4"
                        key={saveState.id}
                      >
                        <div className="flex items-start justify-between gap-4">
                          <div>
                            <p className="text-sm font-semibold text-stone-100">{saveState.name}</p>
                            <p className="mt-1 text-xs text-stone-400">
                              Seed {saveState.seed} at {formatTick(saveState.now_seconds)}
                            </p>
                          </div>
                          <div className="flex items-center gap-3">
                            <button
                              className="text-xs tracking-[0.18em] text-stone-400 uppercase transition hover:text-stone-200"
                              onClick={() => downloadSimulationJson(saveState)}
                              type="button"
                            >
                              Export
                            </button>
                            <button
                              className="text-xs tracking-[0.18em] text-stone-400 uppercase transition hover:text-red-200"
                              onClick={() => deleteSaveState(saveState.id)}
                              type="button"
                            >
                              Delete
                            </button>
                          </div>
                        </div>

                        <button
                          className="mt-3 w-full rounded-2xl border border-white/10 bg-black/20 px-4 py-3 text-xs font-semibold tracking-[0.18em] text-stone-100 uppercase transition hover:border-white/20 hover:bg-black/30 disabled:cursor-not-allowed disabled:opacity-60"
                          data-testid={`simulation-load-save-${saveState.id}`}
                          disabled={busy}
                          onClick={() => {
                            void loadSaveState(saveState);
                          }}
                          type="button"
                        >
                          Load Save State
                        </button>
                      </article>
                    ))}
                  </div>
                </section>

                <section className="rounded-[1.8rem] border border-white/10 bg-black/32 p-5 backdrop-blur-xl">
                  <p className="text-[0.72rem] tracking-[0.28em] text-orange-200/80 uppercase">
                    Sandbox Tools
                  </p>

                  <div className="mt-4 grid gap-2 sm:grid-cols-2">
                    {resourcePacks.map((pack) => (
                      <button
                        className="rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-left text-sm text-stone-100 transition hover:border-white/20 hover:bg-white/8 disabled:cursor-not-allowed disabled:opacity-60"
                        disabled={busy}
                        key={pack.label}
                        onClick={() => {
                          if (!client) {
                            return;
                          }
                          void applyResponse(
                            () => client.grantResource(pack.resource, pack.amount),
                            `${pack.label} applied.`,
                          );
                        }}
                        type="button"
                      >
                        {pack.label}
                      </button>
                    ))}
                  </div>
                </section>
              </>
            ) : (
              <section className="rounded-[1.8rem] border border-white/10 bg-black/30 p-5 backdrop-blur-xl">
                <p className="text-[0.72rem] tracking-[0.28em] text-orange-200/80 uppercase">
                  Campaign State
                </p>
                <h2
                  className="mt-3 text-3xl font-black tracking-[0.05em] text-stone-50 uppercase"
                  style={{ fontFamily: '"Syne", sans-serif' }}
                >
                  Frontier Autosave
                </h2>
                <div className="mt-5 grid gap-3">
                  <InfoRow label="Campaign Seed" value={client.seed.toString()} />
                  <InfoRow
                    label="Latest Autosave"
                    value={
                      mainCampaignSave
                        ? `${formatDateTime(mainCampaignSave.updated_at)} at ${formatTick(mainCampaignSave.now_seconds)}`
                        : "Autosave pending"
                    }
                  />
                  <InfoRow
                    label="Province Count"
                    value={formatValue(view.summary.province_count)}
                  />
                </div>
                <p className="mt-5 text-sm leading-6 text-stone-300">
                  The campaign is persistent. Return to the home screen at any time and continue from
                  the latest autosave later.
                </p>
                <div className="mt-5 grid gap-3 sm:grid-cols-2">
                  <button
                    className="rounded-2xl border border-orange-200/20 bg-orange-300/12 px-4 py-3 text-sm font-semibold tracking-[0.14em] text-orange-50 uppercase transition hover:bg-orange-300/18"
                    onClick={() => {
                      void startFreshMainGame();
                    }}
                    type="button"
                  >
                    Restart Campaign
                  </button>
                  <button
                    className="rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-sm text-stone-100 transition hover:border-white/20 hover:bg-white/8"
                    onClick={() => setScreen("home")}
                    type="button"
                  >
                    Back Home
                  </button>
                </div>
              </section>
            )}
          </aside>

          <section className="flex items-end">
            <div className="w-full rounded-[2rem] border border-white/10 bg-black/18 p-5 backdrop-blur-sm sm:p-6">
              <div className="max-w-xl rounded-[1.7rem] border border-orange-100/12 bg-black/40 px-5 py-4 shadow-[0_18px_48px_rgba(0,0,0,0.26)]">
                <p className="text-[0.7rem] tracking-[0.28em] text-stone-400 uppercase">
                  Live Situation
                </p>
                <p
                  className="mt-2 text-2xl font-black tracking-[0.06em] text-stone-50 uppercase"
                  style={{ fontFamily: '"Syne", sans-serif' }}
                >
                  {placementKind
                    ? `Placing ${buildPalette.find((building) => building.kind === placementKind)?.label ?? placementKind}`
                    : selectedProvince
                      ? `${selectedProvince.name} holds the frontier line`
                      : "Shape the frontier from the heartland outward"}
                </p>
                <p className="mt-3 text-sm leading-6 text-stone-300">
                  {placementKind
                    ? "Click the terrain to queue construction on that tile. Keep the placement mode armed to sketch alternate layouts quickly."
                    : selectedTile
                      ? isSimulationMode
                        ? `Selected tile ${selectedTile.x}, ${selectedTile.y}. Arm a building to place it here, or dispatch a raider from the command table.`
                        : `Selected tile ${selectedTile.x}, ${selectedTile.y}. Arm a building from the palette and click the map to place it here.`
                      : isSimulationMode
                        ? "Build on the map, advance time, and branch snapshots whenever the run reaches an interesting breakpoint."
                        : "Expand carefully, keep the line supplied, and let the objectives pull the campaign toward the next secure province."}
                </p>
              </div>

              {(notice || error) && (
                <div
                  className={`mt-4 rounded-[1.4rem] border px-4 py-3 text-sm ${
                    error
                      ? "border-red-400/40 bg-red-500/10 text-red-100"
                      : "border-emerald-300/25 bg-emerald-400/10 text-emerald-50"
                  }`}
                  data-testid="status-banner"
                >
                  {error ?? notice}
                </div>
              )}

              <section className="mt-4 rounded-[1.8rem] border border-white/10 bg-black/28 p-5 backdrop-blur-xl">
                <div className="flex items-center justify-between gap-4">
                  <div>
                    <p className="text-[0.72rem] tracking-[0.28em] text-orange-200/80 uppercase">
                      Command Table
                    </p>
                    <p className="mt-2 text-sm text-stone-300">
                      {isSimulationMode
                        ? "Advance the frontier clock, recruit from the capital, and probe the map with raiders."
                        : "Advance the frontier clock and recruit from the capital."}
                    </p>
                  </div>
                  <div className="rounded-3xl border border-orange-200/15 bg-white/5 px-4 py-3">
                    <p className="text-[0.7rem] tracking-[0.28em] text-stone-400 uppercase">
                      Clock
                    </p>
                    <p className="mt-2 text-2xl font-bold text-orange-100">
                      <span data-testid="simulation-clock">{formatTick(view.now_seconds)}</span>
                    </p>
                    <p className="mt-2 text-xs text-stone-400">
                      Rank {view.summary.influence_rank} frontier mandate
                    </p>
                  </div>
                </div>

                <div className="mt-4 grid gap-2 sm:grid-cols-3">
                  {timeSteps.map((step) => (
                    <button
                      className="rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-sm text-stone-100 transition hover:border-white/20 hover:bg-white/8 disabled:cursor-not-allowed disabled:opacity-60"
                      disabled={busy}
                      key={step.label}
                      onClick={() => {
                        if (!client) {
                          return;
                        }
                        void applyResponse(
                          () => client.advance(step.seconds),
                          `${isSimulationMode ? "Simulation" : "Campaign"} advanced by ${step.label}.`,
                        );
                      }}
                      type="button"
                    >
                      {step.label}
                    </button>
                  ))}
                </div>

                <div className="mt-5 grid gap-2 sm:grid-cols-3">
                  {recruitableUnits.map((unit) => (
                    <button
                      className="rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-left text-sm text-stone-100 transition hover:border-white/20 hover:bg-white/8 disabled:cursor-not-allowed disabled:opacity-60"
                      disabled={busy || !castle}
                      key={unit.kind}
                      onClick={() => {
                        if (!client || !castle) {
                          return;
                        }
                        void applyResponse(
                          () =>
                            client.apply({
                              SpawnEntity: {
                                blueprint: { Unit: unit.kind },
                                name: null,
                                location: castle.location,
                              },
                            }),
                          `${unit.label} queued.`,
                        );
                      }}
                      type="button"
                    >
                      {unit.label}
                    </button>
                  ))}
                </div>

                {isSimulationMode && (
                  <div className="mt-5 rounded-[1.5rem] border border-red-300/15 bg-red-500/8 p-4">
                    <div className="flex items-start justify-between gap-4">
                      <div>
                        <p className="text-[0.68rem] tracking-[0.26em] text-red-100/75 uppercase">
                          Raider Probe
                        </p>
                        <p className="mt-2 text-sm text-stone-300">
                          Select any tile on the battlefield, then send a raider into the layout from that point.
                        </p>
                      </div>
                      <button
                        className="rounded-2xl border border-red-300/20 bg-red-400/10 px-4 py-3 text-sm font-semibold tracking-[0.14em] text-red-50 uppercase transition hover:bg-red-400/16 disabled:cursor-not-allowed disabled:opacity-60"
                        disabled={busy || !client || !selectedTile}
                        onClick={() => {
                          if (!selectedTile) {
                            return;
                          }
                          sendRaider(selectedTile);
                        }}
                        type="button"
                      >
                        Send Raider
                      </button>
                    </div>
                    <p className="mt-3 text-xs text-stone-400">
                      {selectedTile
                        ? `Entry point armed at ${selectedTile.x}, ${selectedTile.y}.`
                        : "Select a tile on the battlefield to choose the raider entry point."}
                    </p>
                  </div>
                )}
              </section>

              <section className="mt-4 rounded-[1.8rem] border border-white/10 bg-black/28 p-5 backdrop-blur-xl">
                <div className="flex items-center justify-between gap-4">
                  <div>
                    <p className="text-[0.72rem] tracking-[0.28em] text-orange-200/80 uppercase">
                      Build Palette
                    </p>
                    <p className="mt-2 text-sm text-stone-300">
                      Arm a building, then click the map to place it.
                    </p>
                  </div>
                  <div className="flex items-center gap-3">
                    {isSimulationMode && (
                      <div className="max-w-xs rounded-[1.2rem] border border-orange-200/12 bg-white/5 px-4 py-3 text-right">
                        <p className="text-[0.66rem] tracking-[0.24em] text-stone-400 uppercase">
                          Current Build Cost
                        </p>
                        <p className="mt-2 text-sm leading-6 text-stone-100">
                          {currentBuildCost.length ? formatCostList(currentBuildCost) : "No tracked buildings yet."}
                        </p>
                        <p className="mt-1 text-xs text-stone-400">
                          {trackedBuildings.length} placed structure{trackedBuildings.length === 1 ? "" : "s"}
                        </p>
                      </div>
                    )}
                    <button
                      className="rounded-full border border-white/10 bg-white/6 px-4 py-2 text-xs tracking-[0.2em] text-stone-200 uppercase transition hover:border-white/20 hover:bg-white/10"
                      onClick={() => setPlacementKind(null)}
                      type="button"
                    >
                      Clear
                    </button>
                  </div>
                </div>

                <div className="mt-4 grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
                  {buildPalette.map((building) => {
                    const active = placementKind === building.kind;
                    const placementCost = constructionCostAtLevel(building.kind, 1);
                    return (
                      <button
                        className={`rounded-[1.4rem] border px-4 py-4 text-left transition ${
                          active
                            ? "border-orange-200/45 bg-orange-300/12 text-stone-50"
                            : "border-white/10 bg-white/5 text-stone-200 hover:border-white/20 hover:bg-white/8"
                        }`}
                        key={building.kind}
                        onClick={() => setPlacementKind(building.kind)}
                        type="button"
                      >
                        <p className="text-sm font-semibold tracking-[0.16em] uppercase">
                          {building.label}
                        </p>
                        <p className="mt-2 text-xs text-stone-400">{building.note}</p>
                        <p className="mt-3 text-xs leading-5 text-orange-100/85">
                          Build Cost: {formatCostList(placementCost)}
                        </p>
                      </button>
                    );
                  })}
                </div>
              </section>
            </div>
          </section>

          <aside className="flex flex-col gap-4">
            <section className="rounded-[1.8rem] border border-white/10 bg-black/30 p-5 backdrop-blur-xl">
              <div className="flex items-center justify-between gap-4">
                <div>
                  <p className="text-[0.72rem] tracking-[0.28em] text-orange-200/80 uppercase">
                    Selected Building
                  </p>
                  <h2
                    className="mt-2 text-3xl font-black tracking-[0.05em] text-stone-50 uppercase"
                    style={{ fontFamily: '"Syne", sans-serif' }}
                  >
                    {selectedBuilding?.label ?? "No building"}
                  </h2>
                </div>
                <div className="rounded-full border border-orange-300/20 bg-orange-400/10 px-3 py-2 text-center">
                  <p className="text-[0.65rem] tracking-[0.25em] text-orange-100/80 uppercase">
                    Level
                  </p>
                  <p className="mt-1 text-xl font-semibold text-orange-100">
                    {selectedBuilding?.level ?? 0}
                  </p>
                </div>
              </div>

              <div className="mt-5 grid grid-cols-2 gap-3">
                <MetricCard
                  label="Workers"
                  tone="text-orange-100"
                  value={
                    selectedBuilding
                      ? `${selectedBuilding.assigned_workers}/${selectedBuilding.required_workers}`
                      : "0/0"
                  }
                />
                <MetricCard
                  label="Status"
                  tone="text-stone-100"
                  value={selectedBuilding?.status ?? "Idle"}
                />
                <MetricCard
                  label="Security"
                  tone="text-lime-100"
                  value={formatValue(selectedBuilding?.stats.security ?? 0)}
                />
                <MetricCard
                  label="Supply"
                  tone="text-sky-100"
                  value={formatValue(selectedBuilding?.stats.supply ?? 0)}
                />
              </div>

              <div className="mt-5 grid gap-2">
                <button
                  className="rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-left text-sm text-stone-100 transition hover:border-white/20 hover:bg-white/8 disabled:cursor-not-allowed disabled:opacity-60"
                  disabled={busy || !client || !selectedBuilding}
                  onClick={() => {
                    if (!client || !selectedBuilding) {
                      return;
                    }
                    void applyResponse(
                      () =>
                        client.apply({
                          UpgradeBuilding: {
                            building: selectedBuilding.id,
                          },
                        }),
                      `Upgrade requested for ${selectedBuilding.label}.`,
                    );
                  }}
                  type="button"
                >
                  Upgrade Selected Building
                </button>

                {claimableProvince && (
                  <button
                    className="rounded-2xl border border-orange-200/25 bg-orange-300/12 px-4 py-3 text-left text-sm text-orange-50 transition hover:bg-orange-300/18 disabled:cursor-not-allowed disabled:opacity-60"
                    disabled={busy || !client || !selectedBuilding}
                    onClick={() => {
                      if (!client || !selectedBuilding) {
                        return;
                      }
                      void applyResponse(
                        () =>
                          client.apply({
                            SpawnEntity: {
                              blueprint: { Npc: "province" },
                              name: null,
                              location: selectedBuilding.location,
                            },
                          }),
                        `Province claim started from ${selectedBuilding.label}.`,
                      );
                    }}
                    type="button"
                  >
                    Claim Province At Outpost
                  </button>
                )}
              </div>

              {selectedBuilding?.inventory.length ? (
                <div className="mt-5 rounded-[1.4rem] border border-white/10 bg-white/5 p-4">
                  <p className="text-[0.68rem] tracking-[0.26em] text-stone-400 uppercase">
                    Stored Resources
                  </p>
                  <div className="mt-3 grid gap-2">
                    {selectedBuilding.inventory.map((resource) => (
                      <div className="flex items-center justify-between gap-3 text-sm" key={resource.id}>
                        <span className="text-stone-300">{resource.label}</span>
                        <span className="text-stone-100">{formatValue(resource.amount)}</span>
                      </div>
                    ))}
                  </div>
                </div>
              ) : null}
            </section>

            <section className="rounded-[1.8rem] border border-white/10 bg-black/30 p-5 backdrop-blur-xl">
              <p className="text-[0.72rem] tracking-[0.28em] text-orange-200/80 uppercase">
                Objectives
              </p>
              <div className="mt-4 grid gap-3">
                {view.objectives.length === 0 && (
                  <div className="rounded-[1.4rem] border border-dashed border-white/10 bg-white/4 px-4 py-4 text-sm text-stone-400">
                    No active objectives are exposed right now.
                  </div>
                )}

                {view.objectives.map((objective) => (
                  <article
                    className={`rounded-[1.4rem] border px-4 py-4 ${
                      objective.complete
                        ? "border-emerald-300/25 bg-emerald-400/10"
                        : "border-white/10 bg-white/5"
                    }`}
                    key={objective.id}
                  >
                    <div className="flex items-start justify-between gap-4">
                      <p className="text-sm font-semibold tracking-[0.14em] text-stone-50 uppercase">
                        {objective.label}
                      </p>
                      <p className="text-xs text-stone-400">
                        {formatValue(objective.current)} / {formatValue(objective.target)}
                      </p>
                    </div>
                    <p className="mt-2 text-xs text-stone-400">
                      {objective.complete ? "Objective complete." : "Still in progress."}
                    </p>
                  </article>
                ))}
              </div>
            </section>

            <section className="rounded-[1.8rem] border border-white/10 bg-black/30 p-5 backdrop-blur-xl">
              <p className="text-[0.72rem] tracking-[0.28em] text-orange-200/80 uppercase">
                Tech Queue
              </p>
              <div className="mt-4 grid gap-3">
                {view.available_tech_nodes.length === 0 && (
                  <div className="rounded-[1.4rem] border border-dashed border-white/10 bg-white/4 px-4 py-4 text-sm text-stone-400">
                    No tech nodes are currently available.
                  </div>
                )}

                {view.available_tech_nodes.map((techNode) => (
                  <button
                    className="rounded-[1.4rem] border border-white/10 bg-white/5 px-4 py-3 text-left transition hover:border-white/20 hover:bg-white/8 disabled:cursor-not-allowed disabled:opacity-60"
                    disabled={busy || !client}
                    key={techNode}
                    onClick={() => {
                      if (!client) {
                        return;
                      }
                      void applyResponse(
                        () =>
                          client.apply({
                            UnlockTechNode: {
                              kind: techNode,
                            },
                          }),
                        `${formatKind(techNode)} unlocked.`,
                      );
                    }}
                    type="button"
                  >
                    <p className="text-sm font-semibold tracking-[0.14em] text-stone-50 uppercase">
                      {formatKind(techNode)}
                    </p>
                    <p className="mt-2 text-xs text-stone-400">
                      {isSimulationMode
                        ? "Unlock it, then branch the result into a new snapshot if it opens a useful build path."
                        : "Unlock it to strengthen the campaign and widen the next set of frontier options."}
                    </p>
                  </button>
                ))}
              </div>
            </section>

            <section className="rounded-[1.8rem] border border-white/10 bg-black/30 p-5 backdrop-blur-xl">
              <p className="text-[0.72rem] tracking-[0.28em] text-orange-200/80 uppercase">
                Province Board
              </p>
              <div className="mt-4 grid gap-3">
                {provinces.length === 0 && (
                  <div className="rounded-[1.4rem] border border-dashed border-white/10 bg-white/4 px-4 py-4 text-sm text-stone-400">
                    No provinces claimed yet. Build a `Frontier Fort`, secure it, then claim it into
                    a province.
                  </div>
                )}

                {provinces.map((province) => {
                  const active = province.id === selectedProvince?.id;
                  return (
                    <button
                      className={`rounded-[1.4rem] border px-4 py-3 text-left transition ${
                        active
                          ? "border-orange-200/45 bg-orange-300/10 text-stone-50"
                          : "border-white/10 bg-white/4 text-stone-300 hover:border-white/20 hover:bg-white/7"
                      }`}
                      key={province.id}
                      onClick={() => selectProvince(province.id)}
                      type="button"
                    >
                      <div className="flex items-center justify-between gap-4">
                        <p className="text-sm font-semibold tracking-[0.14em] uppercase">
                          {province.name}
                        </p>
                        <p className="text-xs text-stone-400">
                          Threat {formatValue(province.threat)}
                        </p>
                      </div>
                      <p className="mt-2 text-xs text-stone-400">
                        Control {formatValue(province.control)} / Loyalty{" "}
                        {formatValue(province.loyalty)}
                      </p>
                    </button>
                  );
                })}
              </div>
            </section>

            <section className="rounded-[1.8rem] border border-white/10 bg-black/30 p-5 backdrop-blur-xl">
              <p className="text-[0.72rem] tracking-[0.28em] text-orange-200/80 uppercase">Alerts</p>
              <div className="mt-4 grid gap-3">
                {view.alerts.length === 0 && (
                  <div className="rounded-[1.4rem] border border-dashed border-white/10 bg-white/4 px-4 py-4 text-sm text-stone-400">
                    No active alerts at the moment.
                  </div>
                )}

                {view.alerts.map((alert) => (
                  <article
                    className={`rounded-3xl border px-4 py-3 ${severityStyles[alert.severity] ?? severityStyles.info}`}
                    key={alert.message}
                  >
                    <p className="text-[0.68rem] tracking-[0.26em] uppercase">{alert.severity}</p>
                    <p className="mt-2 text-sm leading-6">{alert.message}</p>
                  </article>
                ))}
              </div>
            </section>
          </aside>
        </main>
      </div>
    </div>
  );
}

function HomeScreen({
  activeMode,
  error,
  mainCampaignSave,
  notice,
  onOpenSettings,
  onOpenSimulations,
  onOpenWiki,
  onStartFreshMainGame,
  onStartMainGame,
  saveStates,
}: {
  activeMode: GameMode | null;
  error: string | null;
  mainCampaignSave: CampaignSaveState | null;
  notice: string | null;
  onOpenSettings: () => void;
  onOpenSimulations: () => void;
  onOpenWiki: () => void;
  onStartFreshMainGame: () => void;
  onStartMainGame: () => void;
  saveStates: SimulationSaveState[];
}) {
  const hasMainCampaign = Boolean(mainCampaignSave);

  return (
    <ShellFrame subtitle="Choose where to pick up the frontier: the persistent campaign, the simulation lab, the control room, or the field manual.">
      <section className="rounded-[2rem] border border-white/10 bg-black/30 p-6 shadow-[0_24px_80px_rgba(0,0,0,0.32)] backdrop-blur-xl">
        <p className="text-[0.72rem] tracking-[0.3em] text-orange-200/75 uppercase">Home Screen</p>
        <h1
          className="mt-3 text-5xl font-black tracking-[0.06em] text-stone-50 uppercase sm:text-6xl"
          style={{ fontFamily: '"Syne", sans-serif' }}
        >
          Raid Defense
        </h1>
        <p className="mt-4 max-w-3xl text-sm leading-7 text-stone-300 sm:text-base">
          The frontier now opens on a proper home screen. Continue the main campaign from its last
          autosave, dive into simulations for experimentation, or step into settings and the wiki
          before you commit to the next move.
        </p>

        {(notice || error) && (
          <div
            className={`mt-5 rounded-[1.4rem] border px-4 py-3 text-sm ${
              error
                ? "border-red-400/40 bg-red-500/10 text-red-100"
                : "border-emerald-300/25 bg-emerald-400/10 text-emerald-50"
            }`}
          >
            {error ?? notice}
          </div>
        )}
      </section>

      <section className="mt-4 grid gap-4 lg:grid-cols-2 xl:grid-cols-4">
        <HomeActionCard
          description={
            hasMainCampaign
              ? `Continue from ${formatDateTime(mainCampaignSave.updated_at)} at ${formatTick(mainCampaignSave.now_seconds)}.`
              : "Start the persistent frontier campaign from the configured default seed."
          }
          eyebrow="Main Game"
          primaryActionLabel={hasMainCampaign ? "Continue Campaign" : "Start Campaign"}
          onPrimaryAction={onStartMainGame}
          secondaryActionLabel={hasMainCampaign ? "New Campaign" : undefined}
          onSecondaryAction={hasMainCampaign ? onStartFreshMainGame : undefined}
          title={hasMainCampaign ? "Campaign Autosave Ready" : "Fresh Frontier"}
        />

        <HomeActionCard
          description={
            saveStates.length > 0
              ? `${saveStates.length} simulation snapshots are available to load once you open the lab.`
              : "Open the sandbox to branch alternate timelines, reset seeds, and grant debug resources."
          }
          eyebrow="Simulations"
          primaryActionLabel={activeMode === "simulation" ? "Resume Lab" : "Open Lab"}
          onPrimaryAction={onOpenSimulations}
          title="Simulation Lab"
        />

        <HomeActionCard
          description="Tune default seeds, interface atmosphere, notifications, and stored progress."
          eyebrow="Settings"
          primaryActionLabel="Open Settings"
          onPrimaryAction={onOpenSettings}
          title="Control Room"
        />

        <HomeActionCard
          description="Read the quick reference for campaign flow, buildings, resources, and sandbox behavior."
          eyebrow="Wiki"
          primaryActionLabel="Open Wiki"
          onPrimaryAction={onOpenWiki}
          title="Field Manual"
        />
      </section>
    </ShellFrame>
  );
}

function LoadingScreen({ error, label }: { error: string | null; label: string }) {
  return (
    <div className="flex min-h-screen items-center justify-center bg-[#120d0b] px-6 text-center text-stone-100">
      <div>
        <p className="text-[0.72rem] font-semibold tracking-[0.34em] text-orange-200/70 uppercase">
          Loading
        </p>
        <h1
          className="mt-3 text-4xl font-black tracking-[0.06em] text-stone-50 uppercase"
          style={{ fontFamily: '"Syne", sans-serif' }}
        >
          {label}
        </h1>
        <p className="mt-4 max-w-md text-sm leading-6 text-stone-300">
          Compiling the Raid Defense engine for the browser and opening the selected mode.
        </p>
        {error && <p className="mt-4 text-sm text-red-200">{error}</p>}
      </div>
    </div>
  );
}

function ShellFrame({
  children,
  subtitle,
}: {
  children: React.ReactNode;
  subtitle: string;
}) {
  return (
    <div className="min-h-screen bg-[#120d0b] px-4 py-5 text-stone-100 sm:px-6 lg:px-8">
      <div className="mx-auto max-w-[1400px]">
        <div className="rounded-[2rem] border border-white/10 bg-black/20 px-5 py-4 backdrop-blur-xl">
          <p className="text-[0.72rem] tracking-[0.3em] text-orange-200/75 uppercase">Raid Defense</p>
          <p className="mt-2 max-w-3xl text-sm leading-6 text-stone-300">{subtitle}</p>
        </div>
        <div className="mt-4">{children}</div>
      </div>
    </div>
  );
}

function HomeActionCard({
  description,
  eyebrow,
  onPrimaryAction,
  onSecondaryAction,
  primaryActionLabel,
  secondaryActionLabel,
  title,
}: {
  description: string;
  eyebrow: string;
  onPrimaryAction: () => void;
  onSecondaryAction?: () => void;
  primaryActionLabel: string;
  secondaryActionLabel?: string;
  title: string;
}) {
  return (
    <article className="rounded-[2rem] border border-white/10 bg-black/30 p-5 backdrop-blur-xl">
      <p className="text-[0.72rem] tracking-[0.28em] text-orange-200/80 uppercase">{eyebrow}</p>
      <h2
        className="mt-3 text-3xl font-black tracking-[0.05em] text-stone-50 uppercase"
        style={{ fontFamily: '"Syne", sans-serif' }}
      >
        {title}
      </h2>
      <p className="mt-3 text-sm leading-6 text-stone-300">{description}</p>
      <div className="mt-5 grid gap-3">
        <button
          className="rounded-2xl border border-orange-200/20 bg-orange-300/12 px-4 py-3 text-sm font-semibold tracking-[0.14em] text-orange-50 uppercase transition hover:bg-orange-300/18"
          onClick={onPrimaryAction}
          type="button"
        >
          {primaryActionLabel}
        </button>
        {secondaryActionLabel && onSecondaryAction && (
          <button
            className="rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-sm text-stone-100 transition hover:border-white/20 hover:bg-white/8"
            onClick={onSecondaryAction}
            type="button"
          >
            {secondaryActionLabel}
          </button>
        )}
      </div>
    </article>
  );
}

function WikiPanel({
  children,
  description,
  title,
}: {
  children: React.ReactNode;
  description: string;
  title: string;
}) {
  return (
    <section className="rounded-[2rem] border border-white/10 bg-black/30 p-6 backdrop-blur-xl">
      <p className="text-[0.72rem] tracking-[0.28em] text-orange-200/80 uppercase">{title}</p>
      <p className="mt-3 text-sm leading-6 text-stone-300">{description}</p>
      <div className="mt-5">{children}</div>
    </section>
  );
}

function WikiEmptyState({ text }: { text: string }) {
  return (
    <div className="rounded-[1.4rem] border border-dashed border-white/12 bg-white/[0.03] px-4 py-5 text-sm leading-6 text-stone-400">
      {text}
    </div>
  );
}

function NavButton({ label, onClick }: { label: string; onClick: () => void }) {
  return (
    <button
      className="rounded-full border border-white/10 bg-white/6 px-4 py-2 text-xs tracking-[0.2em] text-stone-200 uppercase transition hover:border-white/20 hover:bg-white/10"
      onClick={onClick}
      type="button"
    >
      {label}
    </button>
  );
}

function SettingInput({
  description,
  label,
  onChange,
  value,
}: {
  description: string;
  label: string;
  onChange: (value: string) => void;
  value: string;
}) {
  return (
    <label className="block rounded-[1.6rem] border border-white/10 bg-white/5 p-4">
      <span className="text-[0.68rem] tracking-[0.26em] text-stone-400 uppercase">{label}</span>
      <input
        className="mt-3 w-full rounded-2xl border border-white/10 bg-black/25 px-4 py-3 text-sm text-stone-100 outline-none transition focus:border-orange-200/40"
        onChange={(event) => onChange(event.target.value)}
        value={value}
      />
      <span className="mt-3 block text-sm leading-6 text-stone-300">{description}</span>
    </label>
  );
}

function ToggleCard({
  checked,
  description,
  label,
  onToggle,
}: {
  checked: boolean;
  description: string;
  label: string;
  onToggle: () => void;
}) {
  return (
    <button
      className="flex items-start justify-between gap-4 rounded-[1.6rem] border border-white/10 bg-white/5 p-4 text-left transition hover:border-white/20 hover:bg-white/8"
      onClick={onToggle}
      type="button"
    >
      <div>
        <p className="text-[0.68rem] tracking-[0.26em] text-stone-400 uppercase">{label}</p>
        <p className="mt-3 text-sm leading-6 text-stone-300">{description}</p>
      </div>
      <span
        className={`inline-flex min-w-20 justify-center rounded-full px-3 py-2 text-xs font-semibold tracking-[0.18em] uppercase ${
          checked
            ? "border border-emerald-300/25 bg-emerald-400/12 text-emerald-50"
            : "border border-white/10 bg-black/30 text-stone-300"
        }`}
      >
        {checked ? "Enabled" : "Disabled"}
      </span>
    </button>
  );
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-[1.4rem] border border-white/10 bg-white/5 px-4 py-4">
      <p className="text-[0.66rem] tracking-[0.26em] text-stone-400 uppercase">{label}</p>
      <p className="mt-3 text-sm leading-6 text-stone-100">{value}</p>
    </div>
  );
}

function MetricCard({ label, value, tone }: { label: string; value: string; tone: string }) {
  return (
    <div className="rounded-[1.4rem] border border-white/10 bg-white/5 px-4 py-4">
      <p className="text-[0.66rem] tracking-[0.26em] text-stone-400 uppercase">{label}</p>
      <p className={`mt-3 text-2xl font-semibold ${tone}`}>{value}</p>
    </div>
  );
}

function hasTrackedBuildCost(kind: string) {
  return kind in buildingCosts;
}

function constructionCostAtLevel(kind: string, level: number) {
  return buildingCosts[kind]?.[level] ?? [];
}

function constructionCostThroughLevel(kind: string, level: number) {
  const levelCosts = Array.from({ length: level }, (_, index) => constructionCostAtLevel(kind, index + 1));
  return sumCosts(levelCosts);
}

function sumCosts(costGroups: CostPart[][]) {
  const totals = new Map<string, number>();

  for (const costGroup of costGroups) {
    for (const cost of costGroup) {
      totals.set(cost.resource, (totals.get(cost.resource) ?? 0) + cost.amount);
    }
  }

  return Array.from(totals.entries())
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([resource, amount]) => ({ resource, amount }));
}

function formatCostList(costs: CostPart[]) {
  if (costs.length === 0) {
    return "No cost data";
  }

  return costs
    .map((cost) => `${formatValue(cost.amount)} ${resourceNames[cost.resource] ?? formatKind(cost.resource)}`)
    .join(" • ");
}

function formatValue(value: number) {
  return numberFormatter.format(value);
}

function formatTick(seconds: number) {
  const minutes = Math.floor(seconds / 60);
  const remainder = seconds % 60;
  return `${minutes}:${remainder.toString().padStart(2, "0")}`;
}

function formatWaveComposition(units: { label: string; count: number }[]) {
  return units
    .map((unit) => `${formatValue(unit.count)} ${unit.label}${unit.count === 1 ? "" : "s"}`)
    .join(" • ");
}

function describeAttackWave(units: { label: string; count: number }[]) {
  const totalUnits = units.reduce((sum, unit) => sum + unit.count, 0);
  if (totalUnits <= 1) {
    return "A light scouting incursion testing the nearest storage and weak points in the line.";
  }
  if (totalUnits <= 3) {
    return "A coordinated raid party with enough bodies to pressure one flank before withdrawing.";
  }
  return "A heavy assault wave large enough to punish exposed stores and overstretched defenses.";
}

function formatKind(kind: string) {
  return kind.replaceAll("_", " ");
}

function formatDateTime(value: string) {
  return dateFormatter.format(new Date(value));
}

function parseSeedOrDefault(value: string, fallback: bigint) {
  try {
    const normalized = BigInt(value.trim());
    return normalized >= 0n ? normalized : fallback;
  } catch {
    return fallback;
  }
}

function loadSaveStates() {
  if (typeof window === "undefined") {
    return [];
  }

  try {
    const stored = window.localStorage.getItem(simulationSaveStorageKey);
    if (!stored) {
      return [];
    }
    const parsed = JSON.parse(stored) as SimulationSaveState[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

function loadMainCampaignSave() {
  if (typeof window === "undefined") {
    return null;
  }

  try {
    const stored = window.localStorage.getItem(mainCampaignStorageKey);
    if (!stored) {
      return null;
    }

    const parsed = JSON.parse(stored) as CampaignSaveState;
    return parsed && typeof parsed.snapshot_json === "string" ? parsed : null;
  } catch {
    return null;
  }
}

function loadAppSettings() {
  if (typeof window === "undefined") {
    return defaultSettings;
  }

  try {
    const stored = window.localStorage.getItem(appSettingsStorageKey);
    if (!stored) {
      return defaultSettings;
    }

    const parsed = JSON.parse(stored) as Partial<AppSettings>;
    return {
      defaultMainSeed: parsed.defaultMainSeed ?? defaultSettings.defaultMainSeed,
      defaultSimulationSeed: parsed.defaultSimulationSeed ?? defaultSettings.defaultSimulationSeed,
      showAmbientOverlay: parsed.showAmbientOverlay ?? defaultSettings.showAmbientOverlay,
      showNotifications: parsed.showNotifications ?? defaultSettings.showNotifications,
    };
  } catch {
    return defaultSettings;
  }
}

function getErrorMessage(error: unknown) {
  if (error instanceof Error) {
    return error.message;
  }
  return "Unexpected simulation error.";
}
