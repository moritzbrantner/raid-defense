# Raid Defense

## Status

- Active game slice built on the shared engine primitives.
- Less complete than Zoo, but clearly beyond a pure prototype.
- Repo-backed by the `raid_defense_game` crate, generated contracts, examples, deterministic seeded state creation, and a browser visualization shell.

## High-Level Pitch

Raid Defense is a deterministic frontier-governance slice about extending a capital into contested provinces while keeping the border stable under pressure. The current implementation centers on logistics, production, control, loyalty, stability, and province claiming rather than on tactical combat or narrative campaign content.

## Player Fantasy

The player is governing an expansionist state. The intended fantasy is to convert core economic strength into frontier infrastructure, secure outposts, claim provinces, and keep the empire stable enough that expansion does not collapse under weak control or empty coffers.

## Current Implemented Experience

The current playable shape is grounded in `new_raid_defense_state`, `raid_defense_view`, province-claim commands such as `claim_named_province`, the `opening_campaign` example, and a React/R3F browser presentation layer.

The simulation already includes a seeded map, economy buildings, roads and areas, units, province and rival NPCs, frontier-fort-driven province claims, tech nodes, upgrades, alerts, and objective tracking. The slice is deterministic and designed to be advanced by code or command surfaces rather than by a bespoke standalone app in this repo.

## Core Gameplay Loop

- Build up the capital and nearby economy with extraction and support buildings.
- Extend infrastructure and worker coverage into the frontier.
- Establish eligible outposts such as frontier forts.
- Spend the required resources to claim provinces at valid locations.
- Raise control and loyalty while containing threat across claimed territory.
- Maintain treasury, stability, and legion strength so expansion remains sustainable.
- Convert stable governance into influence and player-rank progression.

## Core Systems

- Resources: crowns, grain, timber, stone, iron, influence, legion strength, stability, intelligence, trade goods, and citizens.
- Buildings: capital, farmstead, lumber camp, quarry, iron mine, barracks, market, senate hall, watchtower, embassy, and frontier fort.
- Units: prefect, engineer, legate, and envoy.
- NPCs: provinces as explicit claimable NPCs, plus rival houses and caravans as non-province actors.
- Map infrastructure: roads, areas, and seeded map topology support territorial expansion.
- Province claim rules: claims require eligible outposts, minimum outpost level, minimum security, and specific resource costs.
- Governance pressure: tax, stability, control, loyalty, prosperity, and threat are tracked as first-class gameplay signals.
- Alerts and objectives: `raid_defense_view` summarizes frontier health and exposes progress targets.

## World, Content, and Entities

The slice uses a 30x30 seeded map and a provincial world model rather than a single settlement lot. Provinces are represented as NPC-backed entities with control, loyalty, threat, and prosperity values. Buildings and units exist to support extraction, logistics, administration, and frontier projection.

The world is narrower than a full grand-strategy game, but the existing content already establishes a clear relationship between core economy, border infrastructure, and frontier governance.

## Progression, Objectives, and Failure Pressure

The current encoded win pressure is explicit in `raid_defense_summary` and `raid_defense_objectives`:

- Claim 4 provinces.
- Reach average control 70.
- Reach average loyalty 60.
- Bank 100 influence.
- Reach player rank 3.

The current encoded failure pressure is also explicit:

- `critical` is set when crowns drop below 25.
- `critical` is set when stability drops below 20.
- `critical` is set when any province control value drops below 25.
- Alerts additionally warn on low treasury, thin legion strength, missing first claim, and slipping control.

This makes the current slice a governance-and-expansion loop, not just a resource-growth sandbox.

## Current Interfaces and Surfaces

- Rust crate: `games/raid-defense` exposes catalog setup, seeded state creation, seeded world creation, logic hooks, view generation, checksums, and claim helpers.
- JSON DTOs and contracts: the crate defines command request and response types and participates in generated schema and TypeScript outputs.
- Command layer: the slice is intended to run through deterministic `GameCommand` flows and view generation.
- Deterministic seeded worlds: `new_raid_defense_state_with_seed` and the world constructors make repeatable setup part of the public surface.
- Example and tests: `games/raid-defense/examples/opening_campaign.rs` and crate tests anchor the current game shape.

## Constraints and Known Gaps

- Diplomacy is implied by the fantasy but still narrow in the implemented slice.
- Warfare is represented more as pressure and preparedness than as a deep military layer.
- Long-form campaign breadth, faction differentiation, and political event variety are still limited.
- The slice has clear systems, but it does not yet deliver the full scope suggested by the imperial theme.

## Near-Term Roadmap

The next coherent milestone is to deepen claim-to-govern play without turning the project into a full grand-strategy rewrite.

- Strengthen province management consequences after a claim so provinces feel more distinct to govern.
- Improve frontier feedback loops so control, loyalty, security, and logistics create clearer operational tradeoffs.
- Enrich the progression from first outpost to stable multi-province governance.
- Keep the design centered on deterministic logistics and political stability rather than branching into broad diplomacy or battlefield systems.

## Source Anchors

- [../../README.md](../../README.md)
- [src/lib.rs](src/lib.rs)
- [examples/opening_campaign.rs](examples/opening_campaign.rs)
