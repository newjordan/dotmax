#!/usr/bin/env bash
# Render any image to braille at 5 progressive sizes (TXT + SVG).
# Edit SRC/OUT + settings below, then run: ./render_braille.sh

set -euo pipefail

# ---- paths ----
SRC="${SRC:-/mnt/d/BattleSchool/Gladiators/shrekked.jpg}"
OUT="${OUT:-/mnt/c/Users/newjo/Documents/braille_out}"

# ---- render settings ----
export DITHER="${DITHER:-atkinson}"      # none | floyd | bayer | atkinson
export THRESHOLD="${THRESHOLD:-auto}"    # 0-255, or "auto" for Otsu
export BRIGHTNESS="${BRIGHTNESS:-1.0}"
export CONTRAST="${CONTRAST:-1.3}"
export GAMMA="${GAMMA:-1.0}"
export INVERT="${INVERT:-true}"          # invert source brightness before rendering
export INVERT_OUTPUT="${INVERT_OUTPUT:-false}"  # flip final braille bits (filled↔empty)
export FONT_PX="${FONT_PX:-16}"          # SVG font size

# ---- ambient (temporal) dithering ----
# Same image evolves across frames. AMBIENT=0 disables.
export AMBIENT="${AMBIENT:-0.55}"        # 0.0 (static) – 1.0 (heavy churn)
export FRAMES="${FRAMES:-8}"             # frames emitted per size
export FRAME_MS="${FRAME_MS:-120}"       # preview cadence
export AMBIENT_SEED="${AMBIENT_SEED:-0}" # 0 = clock-driven, else pinned

cd "$(dirname "$0")"
cargo run --release --example render_braille --features image -- "$SRC" "$OUT"
