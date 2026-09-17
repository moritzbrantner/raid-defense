#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

cargo build --locked --release -p raid-defense-sim >/dev/null
binary="$root/target/release/raid-defense-sim"

measure() {
  local name="$1"
  shift
  local first second start end elapsed
  first="$(mktemp)"
  second="$(mktemp)"
  trap 'rm -f "$first" "$second"' RETURN

  start="$(date +%s%N)"
  "$binary" "$@" >"$first"
  end="$(date +%s%N)"
  elapsed=$((end - start))

  "$binary" "$@" >"$second"
  cmp --silent "$first" "$second" || {
    echo "raid-defense performance scenario $name became nondeterministic" >&2
    diff -u "$first" "$second" >&2 || true
    exit 1
  }

  printf '{"scenario":"%s","elapsedNs":%s,"deterministic":true,"timing":"advisory-hosted-linux"}\n' \
    "$name" "$elapsed"
  cat "$first"
}

measure "balanced-representative" \
  --seed 41 --runs 1 --ticks 4000 --policy balanced

measure "defense-scaling" \
  --seed 41 --runs 1 --ticks 8000 --policy defense
