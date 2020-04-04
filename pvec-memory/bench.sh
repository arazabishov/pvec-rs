#!/bin/bash

echo "Compiling benchmarks in the release mode."


pushd "benches"
eval "cargo build --release"

echo
echo "Compiling and running the benchmark runner."

popd > /dev/null
eval "cargo run --release"

echo
echo "Benchmark results can be found at target/release/report"
