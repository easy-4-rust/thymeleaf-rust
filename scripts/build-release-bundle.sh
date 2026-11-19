#!/usr/bin/env bash
# Build the GitHub release bundle from a Cargo package, outside the workspace.
set -euo pipefail

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ] || { [ "$#" -eq 3 ] && [ "$3" != "--test" ]; }; then
  echo "Usage: $0 <thymeleaf.crate> <output-dir> [--test]" >&2
  exit 2
fi
repo_root="$(cd "$(dirname "$0")/.." && pwd)"
archive="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
output_dir="$2"
test_package="${3:-}"
version="$(python3 -c 'import sys, tomllib; print(tomllib.load(open(sys.argv[1], "rb"))["workspace"]["package"]["version"])' "$repo_root/Cargo.toml")"
package="thymeleaf-$version"

scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT
python3 "$repo_root/scripts/render_release_notes.py" "$version" > "$scratch/release-notes.md"
tar -xzf "$archive" -C "$scratch"
(
  cd "$scratch/$package"
  # Isolate build outputs and compile-time paths from other extractions.
  export CARGO_TARGET_DIR="$scratch/target"
  if [ "$test_package" = "--test" ]; then
    cargo test --locked --no-default-features
    cargo test --locked --all-features
  fi
  cargo doc --locked --all-features --no-deps
)
test -s "$scratch/target/doc/thymeleaf/index.html"

cp "$archive" "$scratch/$package.crate"
tar -czf "$scratch/$package-api-docs.tar.gz" -C "$scratch/target" doc
(
  cd "$scratch"
  shasum -a 256 "$package.crate" "$package-api-docs.tar.gz" > SHA256SUMS
  shasum -a 256 -c SHA256SUMS
)

mkdir -p "$output_dir"
mv "$scratch/$package.crate" "$scratch/$package-api-docs.tar.gz" \
  "$scratch/SHA256SUMS" "$scratch/release-notes.md" "$output_dir/"
