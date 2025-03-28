#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

# Run command passed as argument to the script and run it for every directory
# found in path.
# Intended usage: while in `examples/contracts` directory, run
# `./ci_contracts.sh "cargo wasm"`

command=${1}

for contract in */; do
    echo "Running ${command} for ${contract}"

    cd $contract
    $command
    cd -
done
