import type { ScenarioOptions, ScenarioWorldRules } from "./scenarioOptions";
import "./ScenarioWorldEditor.css";

type ScenarioWorldEditorProps = {
  scenario: ScenarioOptions;
  onChange: (scenario: ScenarioOptions) => void;
};

function clampInteger(value: string, fallback: number, min: number, max: number) {
  const parsed = Number(value);
  return Number.isInteger(parsed) ? Math.min(max, Math.max(min, parsed)) : fallback;
}

function NumberField({
  label,
  description,
  value,
  min = 0,
  max = 0xffff,
  onChange,
  testId,
}: {
  label: string;
  description: string;
  value: number;
  min?: number;
  max?: number;
  onChange: (value: number) => void;
  testId: string;
}) {
  return (
    <label className="world-number-field">
      <span>
        <strong>{label}</strong>
        <small>{description}</small>
      </span>
      <input
        type="number"
        min={min}
        max={max}
        step={1}
        value={value}
        onChange={(event) => onChange(clampInteger(event.target.value, value, min, max))}
        data-testid={testId}
      />
    </label>
  );
}

function ToggleField({
  label,
  description,
  checked,
  onChange,
  testId,
}: {
  label: string;
  description: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  testId: string;
}) {
  return (
    <label className="world-toggle-field">
      <input
        type="checkbox"
        checked={checked}
        onChange={(event) => onChange(event.target.checked)}
        data-testid={testId}
      />
      <span>
        <strong>{label}</strong>
        <small>{description}</small>
      </span>
    </label>
  );
}

export function ScenarioWorldEditor({ scenario, onChange }: ScenarioWorldEditorProps) {
  const world = scenario.world;

  if (!world) {
    return (
      <div className="scenario-world-rule-stack" data-testid="scenario-world-editor">
        <section className="scenario-rule-section">
          <p>Loading authoritative world defaults from the Rust simulation…</p>
        </section>
      </div>
    );
  }

  const explicitWorld = world;

  function update<K extends keyof ScenarioWorldRules>(key: K, value: ScenarioWorldRules[K]) {
    onChange({
      ...scenario,
      world: {
        ...explicitWorld,
        [key]: value,
      },
    });
  }

  return (
    <div className="scenario-world-rule-stack" data-testid="scenario-world-editor">
      <section className="scenario-rule-section">
        <header>
          <div>
            <h3>Starting resources and forest</h3>
            <p>Set the actual starting reserve and seeded forest quantities used by the simulation.</p>
          </div>
        </header>
        <div className="world-field-grid">
          <NumberField
            label="Starting wood"
            description="Wood stored in the Town Hall when the scenario begins."
            value={world.starting_wood}
            max={500}
            onChange={(value) => update("starting_wood", value)}
            testId="scenario-starting-wood"
          />
          <NumberField
            label="Forest tiles"
            description="Exact number of renewable forest tiles seeded into the world."
            value={world.forest_tile_count}
            min={1}
            max={0xffff}
            onChange={(value) => update("forest_tile_count", value)}
            testId="scenario-forest-tile-count"
          />
          <NumberField
            label="Wood per forest tile"
            description="Initial wood stock available on each forest tile."
            value={world.forest_tile_wood}
            min={1}
            max={0xffff_ffff}
            onChange={(value) => update("forest_tile_wood", value)}
            testId="scenario-forest-tile-wood"
          />
        </div>
      </section>

      <section className="scenario-rule-section">
        <header>
          <div>
            <h3>Forest regrowth</h3>
            <p>Control the amount and cadence directly instead of choosing a slow/standard/fast preset.</p>
          </div>
        </header>
        <div className="world-field-grid">
          <NumberField
            label="Regrowth amount"
            description="Wood restored to a depleted forest when one regrowth cycle completes."
            value={world.forest_regrowth_amount}
            min={1}
            onChange={(value) => update("forest_regrowth_amount", value)}
            testId="scenario-forest-regrowth-amount"
          />
          <NumberField
            label="Regrowth interval (ticks)"
            description="Simulation ticks between forest regrowth cycles."
            value={world.forest_regrowth_interval_ticks}
            min={1}
            onChange={(value) => update("forest_regrowth_interval_ticks", value)}
            testId="scenario-forest-regrowth-interval"
          />
        </div>
      </section>

      <section className="scenario-rule-section">
        <header>
          <div>
            <h3>Sawmill and forestry</h3>
            <p>Set the concrete production batch, work cadence, and local buffering capacity.</p>
          </div>
        </header>
        <div className="world-field-grid">
          <NumberField
            label="Sawmill output"
            description="Wood produced by one completed forestry work cycle."
            value={world.sawmill_output}
            min={1}
            onChange={(value) => update("sawmill_output", value)}
            testId="scenario-sawmill-output"
          />
          <NumberField
            label="Work interval (ticks)"
            description="Ticks a worker spends gathering one sawmill batch."
            value={world.sawmill_interval_ticks}
            min={1}
            onChange={(value) => update("sawmill_interval_ticks", value)}
            testId="scenario-sawmill-interval"
          />
          <NumberField
            label="Sawmill local capacity"
            description="Maximum wood buffered locally at a sawmill before logistics move it."
            value={world.sawmill_local_wood_capacity}
            min={1}
            max={0xffff_ffff}
            onChange={(value) => update("sawmill_local_wood_capacity", value)}
            testId="scenario-sawmill-capacity"
          />
        </div>
      </section>

      <section className="scenario-rule-section">
        <header>
          <div>
            <h3>Day and raid cycle</h3>
            <p>Configure the exact cycle lengths and the two binary raid behaviors separately.</p>
          </div>
        </header>
        <div className="world-field-grid">
          <NumberField
            label="Day length (ticks)"
            description="Peaceful build-up ticks before an automatic raid begins."
            value={world.day_length_ticks}
            min={1}
            onChange={(value) => update("day_length_ticks", value)}
            testId="scenario-day-length-ticks"
          />
          <NumberField
            label="Raid rally (ticks)"
            description="Preparation ticks between starting a configured wave and active combat."
            value={world.raid_rally_ticks}
            min={1}
            onChange={(value) => update("raid_rally_ticks", value)}
            testId="scenario-raid-rally-ticks"
          />
        </div>
        <div className="world-toggle-grid">
          <ToggleField
            label="Automatic raids"
            description="When disabled, each configured wave must be started manually."
            checked={world.automatic_raids}
            onChange={(value) => update("automatic_raids", value)}
            testId="scenario-automatic-raids"
          />
          <ToggleField
            label="Pause economy during raids"
            description="When disabled, production and logistics continue during combat."
            checked={world.pause_economy_during_raids}
            onChange={(value) => update("pause_economy_during_raids", value)}
            testId="scenario-pause-economy"
          />
        </div>
      </section>
    </div>
  );
}
