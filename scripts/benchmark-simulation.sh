#!/usr/bin/env bash
set -euo pipefail

repetitions="${1:-5}"
if ! [[ "$repetitions" =~ ^[1-9][0-9]*$ ]]; then
  echo "usage: $0 [positive-repetitions]" >&2
  exit 2
fi

root="$(git rev-parse --show-toplevel)"
cd "$root"
# Ask Cargo for the effective target path so nested invocation and CARGO_TARGET_DIR stay correct.
target_dir="$(
  cargo metadata --format-version 1 --no-deps |
    python3 -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])'
)"
cargo build --release -p raid-defense-sim >/dev/null
binary="$target_dir/release/raid-defense-sim"
python3 - "$binary" "$repetitions" <<'PY'
import statistics
import subprocess
import sys
import time

binary = sys.argv[1]
repetitions = int(sys.argv[2])
command = [binary, "--seed", "100", "--runs", "16", "--ticks", "6000", "--policy", "balanced"]
samples = []
for _ in range(repetitions):
    start = time.perf_counter()
    subprocess.run(command, check=True, stdout=subprocess.DEVNULL)
    samples.append(time.perf_counter() - start)

print("workload=balanced seeds=16 ticks=6000 seed_start=100")
print(f"repetitions={repetitions}")
print(f"min_seconds={min(samples):.6f}")
print(f"median_seconds={statistics.median(samples):.6f}")
print(f"max_seconds={max(samples):.6f}")
PY
