#!/usr/bin/env bash
# Rasterize the brand SVGs with headless Chromium, then build favicon.ico.
# Requires: a Chromium binary (CHROME, or Playwright's cached build) and ImageMagick.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
dir="$root/docs/user/assets/brand"
chrome="${CHROME:-$(ls -d "$HOME"/.cache/ms-playwright/chromium-*/chrome-linux64/chrome 2>/dev/null | tail -1)}"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# render <svg> <png> <width> <height>
render() {
  local svg="$1" png="$2" w="$3" h="$4"
  cat >"$tmp/page.html" <<EOF
<!doctype html><html><body style="margin:0;background:transparent">
<img src="file://$svg" width="$w" height="$h" style="display:block"></body></html>
EOF
  "$chrome" --headless --disable-gpu --no-sandbox --hide-scrollbars \
    --default-background-color=00000000 --force-device-scale-factor=1 \
    --window-size="$w,$h" --screenshot="$png" "file://$tmp/page.html" >/dev/null 2>&1
}

render "$dir/social-preview.svg" "$dir/social-preview.png" 1280 640
render "$dir/social-image.svg" "$dir/social-image.png" 1200 630
render "$dir/app-icon.svg" "$dir/apple-touch-icon.png" 180 180
render "$dir/app-icon.svg" "$dir/icon-192.png" 192 192
render "$dir/app-icon.svg" "$dir/icon-512.png" 512 512
render "$dir/mark-light.svg" "$dir/mark-light-512.png" 512 512
render "$dir/mark-dark.svg" "$dir/mark-dark-512.png" 512 512

# The .ico has no color-scheme support; use the light-background ink.
sed 's/currentColor/#16181d/g; s/<style>.*<\/style>//' "$dir/favicon.svg" >"$tmp/favicon-ink.svg"
for s in 16 32 48; do render "$tmp/favicon-ink.svg" "$tmp/fav-$s.png" "$s" "$s"; done
cp "$tmp/fav-32.png" "$dir/favicon-32.png"
magick "$tmp/fav-16.png" "$tmp/fav-32.png" "$tmp/fav-48.png" "$dir/favicon.ico"
echo "rasterized into $dir"
