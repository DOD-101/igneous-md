#!/usr/bin/env bash

export TS_RS_EXPORT_DIR
TS_RS_EXPORT_DIR="$(pwd)/crates/igneous-md-protocol/bindings"

output=$(cargo test --package igneous-md-protocol --features export export_bindings_ 2>&1)

if [ $? -ne 0 ]; then
  echo "Failed to export ts bindings:"
  echo "$output"
  exit 1
else
  echo "Successfully exported ts bindings to $TS_RS_EXPORT_DIR"
fi
