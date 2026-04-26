import { startTransition, useDeferredValue, useState } from "react";
import { BattlefieldScene } from "./components/BattlefieldScene";
import { raidDefenseView } from "./data/mockRaidDefense";

const numberFormatter = new Intl.NumberFormat("en-US");

const severityStyles: Record<string, string> = {
  critical: "border-red-400/50 bg-red-500/12 text-red-100",
  warning: "border-amber-300/50 bg-amber-400/12 text-amber-50",
  info: "border-sky-300/40 bg-sky-400/10 text-sky-50",
};

const resourceAccent: Record<string, string> = {
  crowns: "from-amber-200/35 to-amber-600/20",
  grain: "from-lime-200/30 to-lime-500/18",
  influence: "from-cyan-200/30 to-cyan-500/18",
  legion_strength: "from-red-200/30 to-red-500/18",
  stability: "from-orange-200/30 to-orange-500/18",
};

export default function App() {
  const [selectedProvinceId, setSelectedProvinceId] = useState<bigint | null>(
    raidDefenseView.provinces[0]?.id ?? null,
  );
  const deferredProvinceId = useDeferredValue(selectedProvinceId);
  const selectedProvince =
    raidDefenseView.provinces.find((province) => province.id === deferredProvinceId) ??
    raidDefenseView.provinces[0];
  const linkedOutpost = raidDefenseView.buildings.find(
    (building) => building.id === selectedProvince?.stats.outpost_id,
  );

  return (
    <div className="relative min-h-screen overflow-hidden bg-[#120d0b] text-stone-100">
      <BattlefieldScene
        onSelectProvince={(provinceId) => {
          startTransition(() => setSelectedProvinceId(provinceId));
        }}
        selectedProvinceId={selectedProvinceId}
        view={raidDefenseView}
      />

      <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_top,rgba(255,183,126,0.12),transparent_32%),linear-gradient(180deg,rgba(14,8,8,0.08),rgba(14,8,8,0.72))]" />

      <div className="relative z-10 flex min-h-screen flex-col">
        <header className="px-4 pt-4 pb-3 sm:px-6 lg:px-8">
          <div className="grid gap-4 lg:grid-cols-[1.25fr_1fr]">
            <div className="rounded-[2rem] border border-white/10 bg-black/30 px-5 py-5 shadow-[0_24px_80px_rgba(0,0,0,0.35)] backdrop-blur-xl">
              <div className="flex flex-wrap items-start justify-between gap-4">
                <div>
                  <p className="text-[0.72rem] font-semibold tracking-[0.38em] text-orange-200/80 uppercase">
                    Frontier Command
                  </p>
                  <h1
                    className="mt-2 text-4xl font-black tracking-[0.06em] text-stone-50 uppercase sm:text-5xl"
                    style={{ fontFamily: '"Syne", sans-serif' }}
                  >
                    Raid Defense
                  </h1>
                  <p className="mt-3 max-w-2xl text-sm text-stone-300 sm:text-base">
                    Hold the ash frontier together. Stabilize the border, rotate convoys into
                    exposed outposts, and keep the next raid wave from collapsing control.
                  </p>
                </div>

                <div className="min-w-44 rounded-3xl border border-orange-200/15 bg-white/5 px-4 py-3 text-right">
                  <p className="text-[0.7rem] tracking-[0.28em] text-stone-400 uppercase">
                    Campaign Time
                  </p>
                  <p className="mt-2 text-3xl font-bold text-orange-100">
                    {formatTick(raidDefenseView.now_seconds)}
                  </p>
                  <p className="mt-1 text-xs text-stone-400">
                    Rank {raidDefenseView.summary.influence_rank} frontier mandate
                  </p>
                </div>
              </div>
            </div>

            <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-5">
              {raidDefenseView.resources.map((resource) => (
                <div
                  className={`rounded-[1.6rem] border border-white/10 bg-gradient-to-br ${resourceAccent[resource.id] ?? "from-white/10 to-white/0"} px-4 py-4 shadow-[0_16px_40px_rgba(0,0,0,0.24)] backdrop-blur-xl`}
                  key={resource.id}
                >
                  <p className="text-[0.68rem] tracking-[0.28em] text-stone-300 uppercase">
                    {resource.label}
                  </p>
                  <p className="mt-3 text-2xl font-semibold text-stone-50">
                    {formatBig(resource.amount)}
                  </p>
                  <p className="mt-2 text-xs text-stone-300/80">
                    {resource.capacity ? `Cap ${formatBig(resource.capacity)}` : "Uncapped"}
                  </p>
                </div>
              ))}
            </div>
          </div>
        </header>

        <main className="grid flex-1 gap-4 px-4 pb-4 sm:px-6 lg:grid-cols-[320px_minmax(0,1fr)_330px] lg:px-8">
          <aside className="flex flex-col gap-4">
            <section className="rounded-[1.8rem] border border-white/10 bg-black/28 p-5 backdrop-blur-xl">
              <p className="text-[0.72rem] tracking-[0.28em] text-orange-200/80 uppercase">
                Pressure
              </p>
              <div className="mt-4 grid gap-3 sm:grid-cols-3 lg:grid-cols-1">
                <MetricCard
                  label="Average Control"
                  tone="text-orange-100"
                  value={`${formatBig(raidDefenseView.summary.average_control)}%`}
                />
                <MetricCard
                  label="Average Loyalty"
                  tone="text-stone-100"
                  value={`${formatBig(raidDefenseView.summary.average_loyalty)}%`}
                />
                <MetricCard
                  label="Active Legions"
                  tone="text-red-100"
                  value={formatBig(raidDefenseView.summary.active_legions)}
                />
              </div>
            </section>

            <section className="rounded-[1.8rem] border border-white/10 bg-black/32 p-5 backdrop-blur-xl">
              <p className="text-[0.72rem] tracking-[0.28em] text-orange-200/80 uppercase">
                Alerts
              </p>
              <div className="mt-4 grid gap-3">
                {raidDefenseView.alerts.map((alert) => (
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

          <section className="flex items-end">
            <div className="w-full rounded-[2rem] border border-white/10 bg-black/18 p-5 backdrop-blur-sm sm:p-6">
              <div className="max-w-lg rounded-[1.7rem] border border-orange-100/12 bg-black/40 px-5 py-4 shadow-[0_18px_48px_rgba(0,0,0,0.26)]">
                <p className="text-[0.7rem] tracking-[0.28em] text-stone-400 uppercase">
                  Live Situation
                </p>
                <p
                  className="mt-2 text-2xl font-black tracking-[0.06em] text-stone-50 uppercase"
                  style={{ fontFamily: '"Syne", sans-serif' }}
                >
                  {selectedProvince?.name} is the hinge point.
                </p>
                <p className="mt-3 text-sm leading-6 text-stone-300">
                  Threat is cresting on the highlighted province. The scene marks active raid lanes,
                  garrisoned outposts, and the imperial road network feeding the line.
                </p>
              </div>
            </div>
          </section>

          <aside className="flex flex-col gap-4">
            <section className="rounded-[1.8rem] border border-white/10 bg-black/30 p-5 backdrop-blur-xl">
              <div className="flex items-center justify-between gap-4">
                <div>
                  <p className="text-[0.72rem] tracking-[0.28em] text-orange-200/80 uppercase">
                    Focus Province
                  </p>
                  <h2
                    className="mt-2 text-3xl font-black tracking-[0.05em] text-stone-50 uppercase"
                    style={{ fontFamily: '"Syne", sans-serif' }}
                  >
                    {selectedProvince?.name}
                  </h2>
                </div>
                <div className="rounded-full border border-red-300/20 bg-red-400/10 px-3 py-2 text-center">
                  <p className="text-[0.65rem] tracking-[0.25em] text-red-100/80 uppercase">
                    Threat
                  </p>
                  <p className="mt-1 text-xl font-semibold text-red-100">
                    {formatBig(selectedProvince?.threat ?? 0n)}
                  </p>
                </div>
              </div>

              <div className="mt-5 grid grid-cols-2 gap-3">
                <MetricCard
                  label="Control"
                  tone="text-orange-100"
                  value={`${formatBig(selectedProvince?.control ?? 0n)}%`}
                />
                <MetricCard
                  label="Loyalty"
                  tone="text-stone-100"
                  value={`${formatBig(selectedProvince?.loyalty ?? 0n)}%`}
                />
                <MetricCard
                  label="Prosperity"
                  tone="text-lime-100"
                  value={`${formatBig(selectedProvince?.prosperity ?? 0n)}%`}
                />
                <MetricCard
                  label="Militia"
                  tone="text-sky-100"
                  value={formatBig(selectedProvince?.stats.militia ?? 0n)}
                />
              </div>

              <div className="mt-5 rounded-[1.4rem] border border-white/10 bg-white/5 p-4">
                <p className="text-[0.68rem] tracking-[0.26em] text-stone-400 uppercase">
                  Linked Outpost
                </p>
                <p className="mt-2 text-lg font-semibold text-stone-100">
                  {linkedOutpost?.label ?? "No registered fort"}
                </p>
                <p className="mt-2 text-sm leading-6 text-stone-300">
                  {linkedOutpost
                    ? `${linkedOutpost.production} with security ${formatBig(linkedOutpost.stats.security ?? 0n)}.`
                    : "Claims here are unstable until a fort or tower anchors the line."}
                </p>
              </div>
            </section>

            <section className="rounded-[1.8rem] border border-white/10 bg-black/30 p-5 backdrop-blur-xl">
              <p className="text-[0.72rem] tracking-[0.28em] text-orange-200/80 uppercase">
                Province Board
              </p>
              <div className="mt-4 grid gap-3">
                {raidDefenseView.provinces.map((province) => {
                  const active = province.id === selectedProvince?.id;
                  return (
                    <button
                      className={`rounded-[1.4rem] border px-4 py-3 text-left transition ${
                        active
                          ? "border-orange-200/45 bg-orange-300/10 text-stone-50"
                          : "border-white/10 bg-white/4 text-stone-300 hover:border-white/20 hover:bg-white/7"
                      }`}
                      key={province.id.toString()}
                      onClick={() => {
                        startTransition(() => setSelectedProvinceId(province.id));
                      }}
                      type="button"
                    >
                      <div className="flex items-center justify-between gap-4">
                        <p className="text-sm font-semibold tracking-[0.14em] uppercase">
                          {province.name}
                        </p>
                        <p className="text-xs text-stone-400">
                          Threat {formatBig(province.threat)}
                        </p>
                      </div>
                      <p className="mt-2 text-xs text-stone-400">
                        Control {formatBig(province.control)} / Loyalty{" "}
                        {formatBig(province.loyalty)}
                      </p>
                    </button>
                  );
                })}
              </div>
            </section>

            <section className="rounded-[1.8rem] border border-white/10 bg-black/30 p-5 backdrop-blur-xl">
              <p className="text-[0.72rem] tracking-[0.28em] text-orange-200/80 uppercase">
                Victory Track
              </p>
              <div className="mt-4 grid gap-3">
                {raidDefenseView.objectives.map((objective) => (
                  <div
                    className="rounded-[1.4rem] border border-white/10 bg-white/5 px-4 py-3"
                    key={objective.id}
                  >
                    <div className="flex items-center justify-between gap-4">
                      <p className="text-sm font-medium text-stone-100">{objective.label}</p>
                      <span
                        className={`rounded-full px-2 py-1 text-[0.65rem] tracking-[0.25em] uppercase ${
                          objective.complete
                            ? "bg-lime-400/14 text-lime-100"
                            : "bg-stone-200/8 text-stone-300"
                        }`}
                      >
                        {objective.complete ? "secured" : "pending"}
                      </span>
                    </div>
                    <p className="mt-2 text-xs text-stone-400">
                      {formatBig(objective.current)} / {formatBig(objective.target)}
                    </p>
                  </div>
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

function formatBig(value: bigint) {
  return numberFormatter.format(Number(value));
}

function formatTick(seconds: bigint) {
  const totalSeconds = Number(seconds);
  const minutes = Math.floor(totalSeconds / 60);
  const remainder = totalSeconds % 60;
  return `${minutes}:${remainder.toString().padStart(2, "0")}`;
}
