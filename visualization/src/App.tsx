import { useEffect, useState } from "react";
import { RaidDefenseSimulationClient } from "./simulationClient";
import type {
  RaidDefenseCommand,
  RaidDefenseEvent,
  SnapshotView,
} from "./simulationTypes";

const DEFAULT_SEED = 0x5eed;

function provinceName(snapshot: SnapshotView, provinceId: number) {
  return snapshot.provinces.find((province) => province.id === provinceId)?.name ?? "Unknown province";
}

function describeEvent(event: RaidDefenseEvent | null, snapshot: SnapshotView) {
  if (!event) {
    return "Command accepted.";
  }

  switch (event.type) {
    case "fort_built":
      return `${provinceName(snapshot, event.province)} fort upgraded to level ${event.level}.`;
    case "recruited":
      return `${event.soldiers} soldiers reinforced ${provinceName(snapshot, event.province)}.`;
    case "province_claimed":
      return `${provinceName(snapshot, event.province)} joined the frontier.`;
    case "day_advanced":
      return `Day ${event.day}: the frontier held.`;
    case "raid_sighted":
      return `Raid sighted against ${provinceName(snapshot, event.raid.target)}. Strength ${event.raid.strength}; arrival in ${event.raid.eta_days} days.`;
    case "raid_repelled":
      return `${provinceName(snapshot, event.province)} repelled the raid with ${event.losses} losses.`;
    case "raid_breached":
      return `${provinceName(snapshot, event.province)} was breached for ${event.damage} damage.`;
  }
}

function describeError(code: string) {
  const labels: Record<string, string> = {
    unknown_province: "That province no longer exists.",
    province_already_controlled: "That province is already under your control.",
    province_not_controlled: "You must control the province before using that command.",
    frontier_not_connected: "The claim needs an adjacent controlled fort with stable control.",
    fort_at_maximum_level: "The fort is already at its current maximum level.",
    invalid_recruitment: "Recruitment must add at least one soldier.",
    garrison_capacity_exceeded: "The local fort cannot house that many soldiers.",
    insufficient_treasury: "The treasury cannot fund that command yet.",
    invalid_command_json: "The browser produced an invalid command envelope.",
  };

  return labels[code] ?? `Command rejected: ${code}.`;
}

function App() {
  const [client, setClient] = useState<RaidDefenseSimulationClient | null>(null);
  const [snapshot, setSnapshot] = useState<SnapshotView | null>(null);
  const [selectedProvinceId, setSelectedProvinceId] = useState(0);
  const [feedback, setFeedback] = useState("Initializing deterministic frontier…");

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
        setFeedback("The frontier is ready. Fortify before the first raid arrives.");
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

  function issue(command: RaidDefenseCommand) {
    if (!client) {
      return;
    }

    const response = client.dispatch(command);
    setSnapshot(response.snapshot);
    setFeedback(
      response.ok
        ? describeEvent(response.event, response.snapshot)
        : describeError(response.error?.code ?? "unknown_error"),
    );
  }

  if (!snapshot) {
    return (
      <main className="loading-shell" data-testid="loading-state">
        <h1>Raid Defense</h1>
        <p>{feedback}</p>
      </main>
    );
  }

  const selectedProvince =
    snapshot.provinces.find((province) => province.id === selectedProvinceId) ?? snapshot.provinces[0];
  const raidTarget = snapshot.active_raid
    ? provinceName(snapshot, snapshot.active_raid.target)
    : null;

  return (
    <main className="game-shell" data-testid="raid-defense-game">
      <header className="command-bar">
        <div className="identity">
          <span className="eyebrow">Frontier command</span>
          <h1>Raid Defense</h1>
        </div>

        <dl className="campaign-stats" aria-label="Campaign status">
          <div>
            <dt>Day</dt>
            <dd data-testid="day-value">{snapshot.day}</dd>
          </div>
          <div>
            <dt>Treasury</dt>
            <dd data-testid="treasury-value">{snapshot.treasury}</dd>
          </div>
          <div>
            <dt>Influence</dt>
            <dd>{snapshot.influence}</dd>
          </div>
          <div>
            <dt>Capital</dt>
            <dd>{snapshot.capital_health}</dd>
          </div>
        </dl>

        <button
          className="advance-button"
          type="button"
          onClick={() => issue({ type: "advance_day" })}
          data-testid="advance-day"
        >
          Advance day
        </button>
      </header>

      <section className="frontier" aria-labelledby="frontier-heading">
        <div className="frontier-heading-row">
          <div>
            <span className="eyebrow">Operational map</span>
            <h2 id="frontier-heading">The Marches</h2>
          </div>
          {snapshot.active_raid ? (
            <p className="raid-warning" data-testid="raid-warning">
              Raid → {raidTarget} · strength {snapshot.active_raid.strength} · {snapshot.active_raid.eta_days}d
            </p>
          ) : (
            <p className="quiet-frontier">No raid currently sighted</p>
          )}
        </div>

        <div className="province-track" role="list" aria-label="Frontier provinces">
          {snapshot.provinces.map((province, index) => (
            <button
              key={province.id}
              className={`province-node ${province.controlled ? "controlled" : "uncontrolled"} ${
                province.id === selectedProvince.id ? "selected" : ""
              }`}
              type="button"
              role="listitem"
              aria-pressed={province.id === selectedProvince.id}
              onClick={() => setSelectedProvinceId(province.id)}
              data-testid={`province-${province.id}`}
            >
              <span className="province-order">{String(index + 1).padStart(2, "0")}</span>
              <strong>{province.name}</strong>
              <span className="province-state">
                {province.controlled ? `${province.garrison} garrison · fort ${province.fort_level}` : `threat ${province.threat}`}
              </span>
              <span className="control-line" aria-hidden="true">
                <span style={{ width: `${province.control}%` }} />
              </span>
            </button>
          ))}
        </div>
      </section>

      <section className="command-deck" aria-labelledby="province-command-heading">
        <div className="province-brief">
          <span className="eyebrow">Selected province</span>
          <h2 id="province-command-heading">{selectedProvince.name}</h2>
          <p>{selectedProvince.controlled ? "Controlled territory" : "Unclaimed frontier"}</p>
        </div>

        <dl className="province-details">
          <div>
            <dt>Control</dt>
            <dd>{selectedProvince.control}</dd>
          </div>
          <div>
            <dt>Threat</dt>
            <dd>{selectedProvince.threat}</dd>
          </div>
          <div>
            <dt>Fort</dt>
            <dd>{selectedProvince.fort_level}</dd>
          </div>
          <div>
            <dt>Garrison</dt>
            <dd>{selectedProvince.garrison}</dd>
          </div>
        </dl>

        <div className="province-actions" aria-label={`${selectedProvince.name} commands`}>
          <button
            type="button"
            onClick={() => issue({ type: "build_fort", province: selectedProvince.id })}
            data-testid="build-fort"
          >
            Build fort
          </button>
          <button
            type="button"
            onClick={() =>
              issue({ type: "recruit", province: selectedProvince.id, soldiers: 5 })
            }
            data-testid="recruit-five"
          >
            Recruit 5
          </button>
          <button
            type="button"
            onClick={() => issue({ type: "claim_province", province: selectedProvince.id })}
            data-testid="claim-province"
          >
            Claim province
          </button>
        </div>
      </section>

      <p className="feedback-line" role="status" aria-live="polite" data-testid="event-feedback">
        {feedback}
      </p>

      <footer className="evidence-line">
        <span>Seed {snapshot.seed}</span>
        <span>Contract v{snapshot.contract_version}</span>
        <span data-testid="checksum">Checksum {snapshot.checksum}</span>
      </footer>
    </main>
  );
}

export default App;
