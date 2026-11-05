#!/usr/bin/env bash
# Build the crate crates.io receives, then compile the features `cargo package`
# skips. Default features are empty, so a green `cargo package` does not mean
# `--features image` works.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

cargo package --allow-dirty

version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
crate="$root/target/package/dotmax-${version}.crate"
test -f "$crate"

# Keep the unpacked sources outside CARGO_TARGET_DIR. Cargo will not compile a
# package whose manifest lives inside the target directory.
dest=$(mktemp -d "${TMPDIR:-/tmp}/dotmax-crate.XXXXXX")
trap 'rm -rf "$dest"' EXIT
tar -xzf "$crate" -C "$dest"
crate_dir="$dest/dotmax-${version}"

test -f "$crate_dir/src/image/mod.rs"
test -f "$crate_dir/src/image/loader.rs"
test -f "$crate_dir/src/image/svg.rs"
test -f "$crate_dir/src/media/mod.rs"

export CARGO_TARGET_DIR="$root/target"

(
  cd "$crate_dir"
  cargo check --features image
  cargo check --features "image,svg"
  cargo test --doc --features "image,svg"
)
