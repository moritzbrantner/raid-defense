import { useEffect, useRef, useState } from "react";
import "./GameWiki.css";

const FOCUSABLE_SELECTOR =
  'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

export function GameWiki() {
  const [open, setOpen] = useState(false);
  const launcherRef = useRef<HTMLButtonElement>(null);
  const dialogRef = useRef<HTMLElement>(null);

  useEffect(() => {
    if (!open) return;

    const gameShell = document.querySelector<HTMLElement>(".game-shell");
    gameShell?.setAttribute("inert", "");

    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setOpen(false);
        return;
      }

      if (event.key !== "Tab") return;

      const dialog = dialogRef.current;
      if (!dialog) return;

      const focusable = Array.from(dialog.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR));
      if (focusable.length === 0) {
        event.preventDefault();
        return;
      }

      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      const active = document.activeElement;

      if (event.shiftKey && (active === first || !dialog.contains(active))) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && (active === last || !dialog.contains(active))) {
        event.preventDefault();
        first.focus();
      }
    }

    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
      gameShell?.removeAttribute("inert");
      launcherRef.current?.focus();
    };
  }, [open]);

  return (
    <>
      <button
        ref={launcherRef}
        className="wiki-launcher"
        type="button"
        onClick={() => setOpen(true)}
        aria-haspopup="dialog"
        aria-expanded={open}
        data-testid="open-wiki"
      >
        Field guide
      </button>

      {open ? (
        <div
          className="wiki-backdrop"
          onMouseDown={(event) => {
            if (event.currentTarget === event.target) {
              setOpen(false);
            }
          }}
        >
          <section
            ref={dialogRef}
            className="wiki-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="wiki-title"
            data-testid="game-wiki"
          >
            <header className="wiki-header">
              <div>
                <span className="eyebrow">Reference</span>
                <h2 id="wiki-title">Raid Defense field guide</h2>
              </div>
              <button
                className="wiki-close"
                type="button"
                onClick={() => setOpen(false)}
                autoFocus
                aria-label="Close field guide"
                data-testid="close-wiki"
              >
                Close
              </button>
            </header>

            <div className="wiki-content">
              <section>
                <h3>How to play</h3>
                <p>
                  Build an economy during the peaceful phase, turn stored wood into defenses, and
                  survive the raids. The simulation decides whether commands are legal and what
                  happens next; this guide only explains those systems.
                </p>
              </section>

              <section>
                <h3>Forests, sawmills, and storage</h3>
                <p>
                  Forests are finite seeded resources. A sawmill must be placed close enough to a
                  harvestable forest, and harvested wood first waits at the sawmill instead of
                  becoming instantly spendable.
                </p>
                <p>
                  Workers carry that wood into the Town Hall or Storage Houses. All of those stores
                  form the settlement inventory used by authoritative build and upgrade commands.
                </p>
              </section>

              <section>
                <h3>Workers and construction</h3>
                <p>
                  Workers are real units with limited cargo and travel time. Tower placement creates
                  a construction site rather than an active weapon. Workers fetch wood from stocked
                  settlement storage and deliver it to the site; the tower activates only after the
                  required material arrives.
                </p>
              </section>

              <section>
                <h3>Defenses</h3>
                <p>
                  Arrow and cannon towers attack only after construction is complete. Completed
                  towers can be upgraded when the settlement has enough wood. Placement must leave
                  required routes open for raiders and workers.
                </p>
              </section>

              <section>
                <h3>Raiders</h3>
                <p>
                  Raiders seek reachable settlement storage that contains wood and can retarget as
                  the situation changes. They steal stored wood when they reach it. The Town Hall is
                  the fallback objective when there is no better stocked target, and raids can then
                  threaten the settlement directly.
                </p>
              </section>

              <section>
                <h3>Population and progression</h3>
                <p>
                  Houses unlock from completed-wave progression. Houses increase population capacity
                  and add workers, which improves the settlement's ability to move resources and
                  finish construction.
                </p>
              </section>

              <section>
                <h3>Deterministic simulation</h3>
                <p>
                  The world seed, active rules, current state, and ordered commands determine the
                  outcome. Replaying the same inputs produces the same authoritative state and
                  checksum.
                </p>
              </section>
            </div>
          </section>
        </div>
      ) : null}
    </>
  );
}
