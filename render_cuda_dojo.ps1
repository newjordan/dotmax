# Render cuda_dojo.png at 5 progressive sizes via dotmax (invokes WSL).
# Edit the settings below, then run: .\render_cuda_dojo.ps1

# ---- render settings (tweak these) ----
$DITHER     = "none"    # none | floyd | bayer | atkinson
$THRESHOLD  = "128"     # 0-255, or "auto" for Otsu
$BRIGHTNESS = "1.0"     # 0.1-2.0
$CONTRAST   = "1.3"     # 0.1-2.0
$GAMMA      = "1.0"     # 0.1-3.0
$INVERT     = "true"    # true for dark-figure-on-light-bg sources
$FONT_PX    = "16"      # font size for SVG output

# ---- paths (WSL-style; edit if source/dest move) ----
$SRC = "/mnt/d/BattleSchool/Gladiators/cuda_dojo.png"
$OUT = "/mnt/c/Users/newjo/Documents/cuda_dojo"

$cmd = "DITHER=$DITHER THRESHOLD=$THRESHOLD BRIGHTNESS=$BRIGHTNESS CONTRAST=$CONTRAST GAMMA=$GAMMA INVERT=$INVERT FONT_PX=$FONT_PX SRC='$SRC' OUT='$OUT' /mnt/e/dotmax/render_cuda_dojo.sh"
wsl --exec bash -lc $cmd
