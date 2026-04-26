#!/usr/bin/env bash
# Render cuda_dojo.png at 5 progressive sizes via dotmax.
# Edit the settings below, then run: ./render_cuda_dojo.sh

set -euo pipefail

# ---- paths ----
SRC="${SRC:-/mnt/d/BattleSchool/Gladiators/cuda_dojo.png}"
OUT="${OUT:-/mnt/c/Users/newjo/Documents/cuda_dojo}"

# ---- render settings (tweak these) ----
export DITHER="${DITHER:-none}"          # none | floyd | bayer | atkinson
export THRESHOLD="${THRESHOLD:-128}"     # 0-255, or "auto" for Otsu
export BRIGHTNESS="${BRIGHTNESS:-1.0}"   # 0.1-2.0
export CONTRAST="${CONTRAST:-1.3}"       # 0.1-2.0
export GAMMA="${GAMMA:-1.0}"             # 0.1-3.0
export INVERT="${INVERT:-true}"          # true = dark figures on light bg (this image)
export FONT_PX="${FONT_PX:-16}"          # font size for SVG output (controls final size)

# ---- run ----
cd "$(dirname "$0")"
cargo run --release --example render_cuda_dojo --features image -- "$SRC" "$OUT"
