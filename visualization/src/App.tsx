import { startTransition, useEffect, useState } from "react";
import { BattlefieldScene } from "./components/BattlefieldScene";
import { RaidDefenseSimulationClient } from "./simulationClient";
import {
  provincesFromView,
  rivalsFromView,
  type RuntimeMapLocation,
  type RuntimeRaidDefenseCommandResponse,
  type RuntimeRaidDefenseView,
  type SimulationSaveState,
} from "./simulationTypes";

const numberFormatter = new Intl.NumberFormat("en-US");
const saveStorageKey = "raid-defense.simulation.saves.v1";
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

const buildPalette = [
  { kind: "farm", label: "Farm" },
  { kind: "storage_house", label: "Storage House" },
  { kind: "tower", label: "Tower" },
  { kind: "frontier_fort", label: "Frontier Fort" },
  { kind: "barracks", label: "Barracks" },
  { kind: "senate_hall", label: "Senate Hall" },
] as const;

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

export default function App() {
  const [client, setClient] = useState<RaidDefenseSimulationClient | null>(null);
  const [view, setView] = useState<RuntimeRaidDefenseView | null>(null);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [seedInput, setSeedInput] = useState(defaultSeed.toString());
  const [saveName, setSaveName] = useState("");
  const [saveStates, setSaveStates] = useState<SimulationSaveState[]>(() => loadSaveStates());
  const [selectedProvinceId, setSelectedProvinceId] = useState<number | null>(null);
  const [selectedBuildingId, setSelectedBuildingId] = useState<number | null>(null);
  const [selectedTile, setSelectedTile] = useState<RuntimeMapLocation | null>(null);
  const [placementKind, setPlacementKind] = useState<string | null>(null);

  useEffect(() => {
    void bootSimulation(defaultSeed);
  }, []);

  useEffect(() => {
    window.localStorage.setItem(saveStorageKey, JSON.stringify(saveStates));
  }, [saveStates]);

  const provinces = view ? provincesFromView(view) : [];
  const rivals = view ? rivalsFromView(view) : [];

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

  const selectedProvince =
    provinces.find((province) => province.id === selectedProvinceId) ?? provinces[0] ?? null;
  const selectedBuilding =
    view?.buildings.find((building) => building.id === selectedBuildingId) ?? null;
  const claimableProvince =
    selectedBuilding &&
    (selectedBuilding.kind === "frontier_fort" || selectedBuilding.kind === "embassy") &&
    !provinces.some((province) => province.stats.outpost_id === selectedBuilding.id);
  const castle = view?.buildings.find((building) => building.kind === "castle") ?? null;

  async function bootSimulation(seed: bigint) {
    setLoading(true);
    setBusy(true);
    setError(null);

    try {
      const nextClient = await RaidDefenseSimulationClient.create(seed);
      const nextView = nextClient.view();
      startTransition(() => {
        setClient(nextClient);
        setView(nextView);
        setSelectedTile(null);
        setPlacementKind(null);
      });
      setNotice(`Simulation ready on seed ${seed.toString()}.`);
    } catch (nextError) {
      setError(getErrorMessage(nextError));
    } finally {
      setBusy(false);
      setLoading(false);
    }
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
      startTransition(() => {
        setView(response.view);
      });
      setNotice(successNotice);
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
      await bootSimulation(seed);
    } catch {
      setError("Enter a valid non-negative seed.");
    }
  }

  function saveCurrentState() {
    if (!client || !view) {
      return;
    }

    const name = saveName.trim() || `Snapshot ${saveStates.length + 1}`;
    const nextSave: SimulationSaveState = {
      id: typeof crypto !== "undefined" && "randomUUID" in crypto ? crypto.randomUUID() : `${Date.now()}`,
      name,
      seed: client.seed.toString(),
      created_at: new Date().toISOString(),
      now_seconds: view.now_seconds,
      snapshot_json: client.saveSnapshot(),
    };

    startTransition(() => {
      setSaveStates((current) => [nextSave, ...current]);
      setSaveName("");
    });
    setNotice(`Saved ${name}.`);
  }

  async function loadSaveState(saveState: SimulationSaveState) {
    setBusy(true);
    setError(null);

    try {
      const nextClient = await RaidDefenseSimulationClient.create(saveState.seed);
      const response = nextClient.loadSnapshot(saveState.snapshot_json);
      if (!response.accepted) {
        throw new Error(response.error ?? "Could not load save state.");
      }
      startTransition(() => {
        setClient(nextClient);
        setView(response.view);
        setSeedInput(saveState.seed);
        setSelectedTile(null);
      });
      setNotice(`Loaded ${saveState.name}.`);
    } catch (nextError) {
      setError(getErrorMessage(nextError));
    } finally {
      setBusy(false);
    }
  }

  function deleteSaveState(saveId: string) {
    startTransition(() => {
      setSaveStates((current) => current.filter((saveState) => saveState.id !== saveId));
    });
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

  if (loading || !view) {
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
            Simulation
          </h1>
          <p className="mt-4 max-w-md text-sm leading-6 text-stone-300">
            Compiling the Raid Defense engine for the browser and opening the sandbox.
          </p>
          {error && <p className="mt-4 text-sm text-red-200">{error}</p>}
        </div>
      </div>
    );
  }

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

      <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_top,rgba(255,183,126,0.12),transparent_32%),linear-gradient(180deg,rgba(14,8,8,0.08),rgba(14,8,8,0.72))]" />

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
                      Simulation
                    </span>
                  </div>
                  <h1
                    className="mt-4 text-4xl font-black tracking-[0.06em] text-stone-50 uppercase sm:text-5xl"
                    style={{ fontFamily: '"Syne", sans-serif' }}
                  >
                    Raid Defense
                  </h1>
                  <p className="mt-3 max-w-2xl text-sm text-stone-300 sm:text-base">
                    Place buildings, advance time, unlock the frontier, and branch the run into as
                    many save states as you need.
                  </p>
                </div>

                <div className="min-w-52 rounded-3xl border border-orange-200/15 bg-white/5 px-4 py-3">
                  <p className="text-[0.7rem] tracking-[0.28em] text-stone-400 uppercase">
                    Simulation Clock
                  </p>
                  <p className="mt-2 text-3xl font-bold text-orange-100">
                    {formatTick(view.now_seconds)}
                  </p>
                  <p className="mt-2 text-xs text-stone-400">
                    Rank {view.summary.influence_rank} frontier mandate
                  </p>
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
            <section className="rounded-[1.8rem] border border-white/10 bg-black/30 p-5 backdrop-blur-xl">
              <div className="flex items-center justify-between gap-4">
                <div>
                  <p className="text-[0.72rem] tracking-[0.28em] text-orange-200/80 uppercase">
                    Simulation State
                  </p>
                  <p className="mt-2 text-sm text-stone-300">Seeded runs with unlimited snapshots.</p>
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
                  onChange={(event) => setSaveName(event.target.value)}
                  placeholder="Snapshot name"
                  value={saveName}
                />
                <button
                  className="rounded-2xl border border-orange-200/20 bg-orange-300/12 px-4 py-3 text-xs font-semibold tracking-[0.18em] text-orange-50 uppercase transition hover:bg-orange-300/18 disabled:cursor-not-allowed disabled:opacity-60"
                  disabled={busy}
                  onClick={saveCurrentState}
                  type="button"
                >
                  Save
                </button>
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
                      <button
                        className="text-xs tracking-[0.18em] text-stone-400 uppercase transition hover:text-red-200"
                        onClick={() => deleteSaveState(saveState.id)}
                        type="button"
                      >
                        Delete
                      </button>
                    </div>

                    <button
                      className="mt-3 w-full rounded-2xl border border-white/10 bg-black/20 px-4 py-3 text-xs font-semibold tracking-[0.18em] text-stone-100 uppercase transition hover:border-white/20 hover:bg-black/30 disabled:cursor-not-allowed disabled:opacity-60"
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

              <div className="mt-5 grid gap-2 sm:grid-cols-3">
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
                        `Advanced the simulation by ${step.label}.`,
                      );
                    }}
                    type="button"
                  >
                    {step.label}
                  </button>
                ))}
              </div>

              <div className="mt-5 grid gap-2">
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
            </section>
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
                      : "Shape the sandbox from the heartland outward"}
                </p>
                <p className="mt-3 text-sm leading-6 text-stone-300">
                  {placementKind
                    ? "Click the terrain to queue construction on that tile. Keep the placement mode armed to sketch alternate layouts quickly."
                    : selectedTile
                      ? `Selected tile ${selectedTile.x}, ${selectedTile.y}. Arm a building from the palette and click the map to place it here.`
                      : "Build on the map, advance time, and save branching snapshots whenever a layout reaches an interesting breakpoint."}
                </p>
              </div>

              {(notice || error) && (
                <div
                  className={`mt-4 rounded-[1.4rem] border px-4 py-3 text-sm ${
                    error
                      ? "border-red-400/40 bg-red-500/10 text-red-100"
                      : "border-emerald-300/25 bg-emerald-400/10 text-emerald-50"
                  }`}
                >
                  {error ?? notice}
                </div>
              )}

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
                  <button
                    className="rounded-full border border-white/10 bg-white/6 px-4 py-2 text-xs tracking-[0.2em] text-stone-200 uppercase transition hover:border-white/20 hover:bg-white/10"
                    onClick={() => setPlacementKind(null)}
                    type="button"
                  >
                    Clear
                  </button>
                </div>

                <div className="mt-4 grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
                  {buildPalette.map((building) => {
                    const active = placementKind === building.kind;
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
                        <p className="mt-2 text-xs text-stone-400">Click the battlefield to queue.</p>
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
                      Add the tech, then branch the result into a new save state if it opens a
                      useful build path.
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
                    No provinces claimed yet. Build a `Frontier Fort`, grant resources if needed,
                    then claim it into a province.
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
              <p className="text-[0.72rem] tracking-[0.28em] text-orange-200/80 uppercase">
                Alerts
              </p>
              <div className="mt-4 grid gap-3">
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

function MetricCard({ label, value, tone }: { label: string; value: string; tone: string }) {
  return (
    <div className="rounded-[1.4rem] border border-white/10 bg-white/5 px-4 py-4">
      <p className="text-[0.66rem] tracking-[0.26em] text-stone-400 uppercase">{label}</p>
      <p className={`mt-3 text-2xl font-semibold ${tone}`}>{value}</p>
    </div>
  );
}

function formatValue(value: number) {
  return numberFormatter.format(value);
}

function formatTick(seconds: number) {
  const minutes = Math.floor(seconds / 60);
  const remainder = seconds % 60;
  return `${minutes}:${remainder.toString().padStart(2, "0")}`;
}

function formatKind(kind: string) {
  return kind.replaceAll("_", " ");
}

function loadSaveStates() {
  if (typeof window === "undefined") {
    return [];
  }

  try {
    const stored = window.localStorage.getItem(saveStorageKey);
    if (!stored) {
      return [];
    }
    const parsed = JSON.parse(stored) as SimulationSaveState[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

function getErrorMessage(error: unknown) {
  if (error instanceof Error) {
    return error.message;
  }
  return "Unexpected simulation error.";
}
