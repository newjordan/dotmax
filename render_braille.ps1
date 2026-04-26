# Render any image to braille at 5 progressive sizes (TXT + SVG) via dotmax.
# Edit the settings below, then run: .\render_braille.ps1

# ---- render settings ----
$DITHER     = "atkinson"  # none | floyd | bayer | atkinson
$THRESHOLD  = "auto"      # 0-255, or "auto" for Otsu
$BRIGHTNESS = "1.0"
$CONTRAST   = "1.3"
$GAMMA      = "1.0"
$INVERT        = "true"
$INVERT_OUTPUT = "false"  # flip final braille bits (filled↔empty)
$FONT_PX       = "16"

# ---- paths ----
$SRC = "/mnt/d/BattleSchool/Gladiators/shrekked.jpg"
$OUT = "/mnt/c/Users/newjo/Documents/braille_out"

$cmd = "DITHER=$DITHER THRESHOLD=$THRESHOLD BRIGHTNESS=$BRIGHTNESS CONTRAST=$CONTRAST GAMMA=$GAMMA INVERT=$INVERT INVERT_OUTPUT=$INVERT_OUTPUT FONT_PX=$FONT_PX SRC='$SRC' OUT='$OUT' /mnt/e/dotmax/render_braille.sh"
wsl --exec bash -lc $cmd
