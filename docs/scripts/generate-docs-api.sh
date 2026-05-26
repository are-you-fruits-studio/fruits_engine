#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

target_doc_dir="target/doc"
target_main_crate_index="$target_doc_dir/fruits_engine/index.html"

if [[ "${SKIP_RUSTDOC:-0}" != "1" ]]; then
  cargo +nightly doc --workspace --no-deps
elif [[ ! -f "$target_main_crate_index" ]]; then
  echo "Cannot skip rustdoc generation because target/doc/fruits_engine/index.html does not exist. Run with SKIP_RUSTDOC unset once." >&2
  exit 1
fi

if [[ ! -d "$target_doc_dir" ]]; then
  echo "Rustdoc output was not found at target/doc. Run cargo doc first." >&2
  exit 1
fi

if [[ ! -f "$target_main_crate_index" ]]; then
  echo "Main rustdoc entry was not found: $target_main_crate_index" >&2
  exit 1
fi

echo "Rustdoc API reference generated at: $target_doc_dir"
