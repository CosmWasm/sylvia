#!/bin/bash
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
