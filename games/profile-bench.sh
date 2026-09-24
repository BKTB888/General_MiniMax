#!/usr/bin/env bash
# Profiles benchmarks with samply, looping each for <seconds> without Criterion's statistics.
# Without a filter, every benchmark in <bench> runs, one after another.
# Examples: games/profile-bench.sh mancala node_ops
#           games/profile-bench.sh mega_tictactoe node_ops five_in_row_node_ops/eval
set -euo pipefail

if [ $# -lt 2 ]; then
    echo "usage: $0 <package> <bench> [filter] [seconds=10]" >&2
    exit 1
fi
package=$1
bench=$2
filter=${3:-}
seconds=${4:-10}

cd "$(dirname "$0")/.."

# The bench binary's file name has a hash in it, so ask cargo where it put it.
exe=$(cargo bench -p "$package" --bench "$bench" --no-run --message-format=json |
    jq -r 'select(.executable != null and (.target.kind | index("bench"))) | .executable')

samply record -o target/profile.json.gz "$exe" --bench --profile-time "$seconds" ${filter:+"$filter"}
