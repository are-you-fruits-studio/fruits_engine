#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

api_dir="docs/api-reference"
base_path="/api-reference"

if [[ "${SKIP_RUSTDOC_JSON:-0}" != "1" ]]; then
  RUSTDOCFLAGS="${RUSTDOCFLAGS:--Z unstable-options --output-format json}" \
    cargo +nightly doc --workspace --no-deps
fi

find "$api_dir" -mindepth 1 ! -name '.gitkeep' -exec rm -rf {} +
mkdir -p "$api_dir"

cargo doc-docusaurus components init docs

workspace_crates="$(find target/doc -maxdepth 1 -name '*.json' -printf '%f\n' \
  | sed 's/\.json$//' \
  | sort \
  | paste -sd, -)"

if [[ -z "$workspace_crates" ]]; then
  echo "No rustdoc JSON files found in target/doc. Run rustdoc JSON generation first." >&2
  exit 1
fi

for json in target/doc/*.json; do
  cargo doc-docusaurus "$json" \
    -o "$api_dir" \
    --base-path "$base_path" \
    --workspace-crates "$workspace_crates"
done

find "$api_dir" -type f \( -name '*.md' -o -name '*.mdx' \) -print0 \
  | xargs -0 -r sed -i '/^displayed_sidebar:/d'

find "$api_dir" -type f \( -name '*.md' -o -name '*.mdx' \) -print0 \
  | xargs -0 -r perl -0pi -e 's/(?<!!)\[([^\]\n]+)\]\((?!https?:\/\/|mailto:|#)([^)\n]+)\)/$1/g'

find "$api_dir" -type f \( -name '*.md' -o -name '*.mdx' \) -print0 \
  | xargs -0 -r perl -0pi -e 's/<Link\b(?=[^>]*\bto="(?!https?:\/\/|mailto:|#)[^"]*")[^>]*>(.*?)<\/Link>/$1/gs; s/^import Link from .@docusaurus\/Link.;\r?\n//mg'
