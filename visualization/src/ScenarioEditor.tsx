import { useState } from "react";
import type {
  RaiderArchetype,
  RaiderRules,
  ScenarioOptions,
  ScenarioTowerRules,
  ScenarioWave,
  TowerArchetypeRules,
  TowerLevelRules,
  WaveGroup,
} from "./scenarioOptions";
import { ScenarioWorldEditor } from "./ScenarioWorldEditor";
import "./ScenarioEditor.css";

type EditorSection = "waves" | "units" | "towers" | "world";
type TowerKind = "arrow" | "cannon";

type ScenarioEditorProps = {
  scenario: ScenarioOptions;
  onChange: (scenario: ScenarioOptions) => void;
  hasSave: boolean;
};

function clampInteger(value: string, fallback: number, min: number, max: number) {
  const parsed = Number(value);
  return Number.isInteger(parsed) ? Math.min(max, Math.max(min, parsed)) : fallback;
}

function replaceAt<T>(values: T[], index: number, value: T) {
  return values.map((current, currentIndex) => (currentIndex === index ? value : current));
}

function scenarioSectionFromUrl(): EditorSection {
  const value = new URLSearchParams(window.location.search).get("scenarioView");
  return value === "units" || value === "towers" || value === "world" ? value : "waves";
}

function NumberField({
  label,
  value,
  min = 0,
  max = 0xffff,
  onChange,
  testId,
}: {
  label: string;
  value: number;
  min?: number;
  max?: number;
  onChange: (value: number) => void;
  testId?: string;
}) {
  return (
    <label className="scenario-number-field">
      <span>{label}</span>
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

function UnitEditor({
  name,
  value,
  onChange,
  testId,
}: {
  name: string;
  value: RaiderRules;
  onChange: (value: RaiderRules) => void;
  testId: string;
}) {
  const update = <K extends keyof RaiderRules>(key: K, next: RaiderRules[K]) =>
    onChange({ ...value, [key]: next });

  return (
    <section className="scenario-rule-section" data-testid={testId}>
      <header>
        <h3>{name}</h3>
        <p>These values are used directly whenever this unit appears in a configured wave.</p>
      </header>
      <div className="scenario-field-grid">
        <NumberField label="Health" value={value.health} min={1} onChange={(next) => update("health", next)} />
        <NumberField label="Damage" value={value.damage} min={1} onChange={(next) => update("damage", next)} />
        <NumberField
          label="Speed (milli)"
          value={value.speed_milli}
          min={1}
          onChange={(next) => update("speed_milli", next)}
        />
        <NumberField
          label="Wood stolen"
          value={value.wood_steal}
          min={1}
          max={0xffff_ffff}
          onChange={(next) => update("wood_steal", next)}
        />
      </div>
    </section>
  );
}

function TowerLevelEditor({
  level,
  value,
  onChange,
}: {
  level: number;
  value: TowerLevelRules;
  onChange: (value: TowerLevelRules) => void;
}) {
  const update = <K extends keyof TowerLevelRules>(key: K, next: TowerLevelRules[K]) =>
    onChange({ ...value, [key]: next });

  return (
    <fieldset className="tower-level-editor">
      <legend>Level {level}</legend>
      <div className="scenario-field-grid">
        <NumberField label="Damage" value={value.damage} onChange={(next) => update("damage", next)} />
        <NumberField
          label="Range (milli)"
          value={value.range_milli}
          max={0x7fff_ffff}
          onChange={(next) => update("range_milli", next)}
        />
        <NumberField
          label="Cooldown ticks"
          value={value.cooldown_ticks}
          max={0xff}
          onChange={(next) => update("cooldown_ticks", next)}
        />
        <NumberField
          label="Projectile speed"
          value={value.projectile_speed_milli}
          onChange={(next) => update("projectile_speed_milli", next)}
        />
        <label className="scenario-number-field">
          <span>Upgrade cost</span>
          <input
            type="number"
            min={0}
            max={0xffff_ffff}
            step={1}
            placeholder="Final level"
            value={value.upgrade_cost ?? ""}
            onChange={(event) =>
              update(
                "upgrade_cost",
                event.target.value === ""
                  ? null
                  : clampInteger(event.target.value, value.upgrade_cost ?? 0, 0, 0xffff_ffff),
              )
            }
          />
        </label>
      </div>
    </fieldset>
  );
}

function TowerEditor({
  name,
  value,
  onChange,
}: {
  name: string;
  value: TowerArchetypeRules;
  onChange: (value: TowerArchetypeRules) => void;
}) {
  function updateLevel(index: number, level: TowerLevelRules) {
    const levels = value.levels.map((current, currentIndex) =>
      currentIndex === index ? level : current,
    ) as TowerArchetypeRules["levels"];
    onChange({ ...value, levels });
  }

  return (
    <section className="scenario-rule-section">
      <header>
        <h3>{name}</h3>
        <p>Construction and combat values are authoritative for this tower archetype.</p>
      </header>
      <NumberField
        label="Build cost"
        value={value.build_cost}
        max={0xffff_ffff}
        onChange={(build_cost) => onChange({ ...value, build_cost })}
      />
      <div className="tower-levels">
        {value.levels.map((level, index) => (
          <TowerLevelEditor key={index} level={index + 1} value={level} onChange={(next) => updateLevel(index, next)} />
        ))}
      </div>
    </section>
  );
}

export function ScenarioEditor({ scenario, onChange, hasSave }: ScenarioEditorProps) {
  const [section, setSection] = useState<EditorSection>(() => scenarioSectionFromUrl());
  const [selectedWave, setSelectedWave] = useState(0);
  const [towerKind, setTowerKind] = useState<TowerKind>("arrow");

  function chooseSection(next: EditorSection) {
    const url = new URL(window.location.href);
    url.searchParams.set("scenarioView", next);
    window.history.replaceState({}, "", url);
    setSection(next);
  }

  function updateWave(index: number, wave: ScenarioWave) {
    onChange({ ...scenario, waves: replaceAt(scenario.waves, index, wave) });
  }

  function addWave() {
    if (scenario.waves.length >= 64) return;
    const source = scenario.waves.at(-1) ?? { groups: [{ archetype: "basic" as const, count: 1 }] };
    const wave = { groups: source.groups.map((group) => ({ ...group })) };
    const waves = [...scenario.waves, wave];
    onChange({ ...scenario, waves });
    setSelectedWave(waves.length - 1);
  }

  function duplicateWave() {
    if (scenario.waves.length >= 64) return;
    const source = scenario.waves[selectedWave];
    if (!source) return;
    const wave = { groups: source.groups.map((group) => ({ ...group })) };
    const waves = [
      ...scenario.waves.slice(0, selectedWave + 1),
      wave,
      ...scenario.waves.slice(selectedWave + 1),
    ];
    onChange({ ...scenario, waves });
    setSelectedWave(selectedWave + 1);
  }

  function removeWave() {
    if (scenario.waves.length <= 1) return;
    const waves = scenario.waves.filter((_, index) => index !== selectedWave);
    onChange({ ...scenario, waves });
    setSelectedWave(Math.min(selectedWave, waves.length - 1));
  }

  function updateGroup(groupIndex: number, group: WaveGroup) {
    const wave = scenario.waves[selectedWave];
    if (!wave) return;
    updateWave(selectedWave, { ...wave, groups: replaceAt(wave.groups, groupIndex, group) });
  }

  function addGroup() {
    const wave = scenario.waves[selectedWave];
    if (!wave || wave.groups.length >= 8) return;
    updateWave(selectedWave, {
      ...wave,
      groups: [...wave.groups, { archetype: "basic", count: 1 }],
    });
  }

  function removeGroup(groupIndex: number) {
    const wave = scenario.waves[selectedWave];
    if (!wave || wave.groups.length <= 1) return;
    updateWave(selectedWave, {
      ...wave,
      groups: wave.groups.filter((_, index) => index !== groupIndex),
    });
  }

  function updateUnit(archetype: RaiderArchetype, value: RaiderRules) {
    onChange({
      ...scenario,
      raiders: { ...scenario.raiders, [archetype]: value },
    });
  }

  function updateTower(kind: TowerKind, value: TowerArchetypeRules) {
    onChange({
      ...scenario,
      towers: { ...scenario.towers, [kind]: value },
    });
  }

  function updateTowerCatalog<K extends keyof ScenarioTowerRules>(key: K, value: ScenarioTowerRules[K]) {
    onChange({ ...scenario, towers: { ...scenario.towers, [key]: value } });
  }

  const wave = scenario.waves[selectedWave] ?? scenario.waves[0];

  return (
    <div className="scenario-editor" data-testid="scenario-editor">
      <p className="menu-copy">
        Define the next run directly. Wave composition, unit stats, tower rules, and world values are saved with the
        scenario and consumed by the authoritative simulation; there are no hidden difficulty presets behind the editor.
      </p>

      <nav className="scenario-editor-tabs" aria-label="Scenario editor sections">
        {(["waves", "units", "towers", "world"] as const).map((item) => (
          <button
            key={item}
            type="button"
            className={section === item ? "active" : ""}
            onClick={() => chooseSection(item)}
            data-testid={`scenario-${item}-tab`}
          >
            {item[0].toUpperCase() + item.slice(1)}
          </button>
        ))}
      </nav>

      {section === "waves" && wave ? (
        <div className="wave-editor" data-testid="scenario-wave-editor">
          <aside className="wave-list" aria-label="Configured waves">
            {scenario.waves.map((configuredWave, index) => (
              <button
                key={index}
                type="button"
                className={selectedWave === index ? "active" : ""}
                onClick={() => setSelectedWave(index)}
                data-testid={`scenario-wave-${index + 1}`}
              >
                <strong>Wave {index + 1}</strong>
                <span>{configuredWave.groups.reduce((total, group) => total + group.count, 0)} raiders</span>
              </button>
            ))}
            <button type="button" className="wave-add" onClick={addWave} disabled={scenario.waves.length >= 64}>
              + Add wave
            </button>
          </aside>

          <section className="wave-detail" aria-labelledby="selected-wave-title">
            <header>
              <div>
                <span className="scenario-eyebrow">Explicit composition</span>
                <h3 id="selected-wave-title">Wave {selectedWave + 1}</h3>
              </div>
              <div className="wave-actions">
                <button type="button" onClick={duplicateWave} disabled={scenario.waves.length >= 64}>
                  Duplicate
                </button>
                <button type="button" onClick={removeWave} disabled={scenario.waves.length <= 1}>
                  Remove
                </button>
              </div>
            </header>

            <div className="wave-groups">
              {wave.groups.map((group, index) => (
                <div className="wave-group-row" key={index}>
                  <label>
                    <span>Raider type</span>
                    <select
                      value={group.archetype}
                      onChange={(event) =>
                        updateGroup(index, { ...group, archetype: event.target.value as RaiderArchetype })
                      }
                      data-testid={`scenario-wave-group-type-${index}`}
                    >
                      <option value="basic">Basic raider</option>
                      <option value="advanced">Advanced raider</option>
                    </select>
                  </label>
                  <NumberField
                    label="Count"
                    value={group.count}
                    min={1}
                    onChange={(count) => updateGroup(index, { ...group, count })}
                    testId={`scenario-wave-group-count-${index}`}
                  />
                  <button
                    type="button"
                    className="row-remove"
                    onClick={() => removeGroup(index)}
                    disabled={wave.groups.length <= 1}
                    aria-label={`Remove group ${index + 1}`}
                  >
                    Remove
                  </button>
                </div>
              ))}
            </div>
            <button type="button" className="scenario-inline-button" onClick={addGroup} disabled={wave.groups.length >= 8}>
              + Add raider group
            </button>
          </section>
        </div>
      ) : null}

      {section === "units" ? (
        <div className="scenario-rule-stack" data-testid="scenario-unit-editor">
          <UnitEditor
            name="Basic raider"
            value={scenario.raiders.basic}
            onChange={(value) => updateUnit("basic", value)}
            testId="scenario-unit-basic"
          />
          <UnitEditor
            name="Advanced raider"
            value={scenario.raiders.advanced}
            onChange={(value) => updateUnit("advanced", value)}
            testId="scenario-unit-advanced"
          />
        </div>
      ) : null}

      {section === "towers" ? (
        <div data-testid="scenario-tower-editor">
          <section className="scenario-rule-section tower-global-rules">
            <header>
              <h3>Tower rules</h3>
              <p>Shared tower durability and the active level count are part of the scenario.</p>
            </header>
            <div className="scenario-field-grid">
              <NumberField
                label="Maximum health"
                value={scenario.towers.max_health}
                min={1}
                onChange={(value) => updateTowerCatalog("max_health", value)}
              />
              <NumberField
                label="Maximum level"
                value={scenario.towers.max_level}
                min={1}
                max={3}
                onChange={(value) => updateTowerCatalog("max_level", value)}
              />
            </div>
          </section>
          <div className="tower-kind-tabs" role="group" aria-label="Tower archetype">
            <button type="button" className={towerKind === "arrow" ? "active" : ""} onClick={() => setTowerKind("arrow")}>
              Arrow tower
            </button>
            <button type="button" className={towerKind === "cannon" ? "active" : ""} onClick={() => setTowerKind("cannon")}>
              Cannon tower
            </button>
          </div>
          <TowerEditor
            name={towerKind === "arrow" ? "Arrow tower" : "Cannon tower"}
            value={scenario.towers[towerKind]}
            onChange={(value) => updateTower(towerKind, value)}
          />
        </div>
      ) : null}

      {section === "world" ? <ScenarioWorldEditor scenario={scenario} onChange={onChange} /> : null}

      {hasSave ? (
        <p className="scenario-note">These changes affect the next new game only. The current save keeps its recorded scenario.</p>
      ) : null}
    </div>
  );
}
