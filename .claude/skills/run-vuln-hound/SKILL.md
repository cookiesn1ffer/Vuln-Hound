---
name: run-vuln-hound
description: Launch and screenshot the Vuln-Hound Tauri desktop app in dev mode. Use when asked to run, start, preview, or screenshot the Vuln-Hound app.
---

Vuln-Hound is a Tauri v2 + React desktop app. On this machine (Hyprland/Wayland via Omarchy), the raw webkit2gtk window crashes on launch with `Gdk-Message: Error 71 (Protocol error) dispatching to Wayland display` unless forced onto XWayland.

## Run (dev, agent path)

```bash
cd /home/cookie/Vuln-Hound
GDK_BACKEND=x11 WEBKIT_DISABLE_COMPOSITING_MODE=1 nohup npm run tauri dev > /tmp/tauri-dev.log 2>&1 &
```

Wait for `Running \`target/debug/vuln-hound\`` in the log (first Rust build ~45s cold, ~3s incremental).

## Screenshot

Find the window and grab it with `grim` (no xvfb/Playwright needed — this is a real Wayland/XWayland session):

```bash
hyprctl clients -j | python3 -c "
import json,sys
for w in json.load(sys.stdin):
    if w.get('class') == 'Vuln-hound':
        print(w['at'], w['size'])
"
# then:
grim -g "<x>,<y> <w>x<h>" /tmp/shot.png
```

## Stop

```bash
pkill -f "target/debug/vuln-hound"; pkill -f "node .*vite"
```

## Gotchas

- **`Gdk-Message: Error 71 (Protocol error)` on launch** — native Wayland webkit2gtk fails on this compositor. Fix: `GDK_BACKEND=x11` (routes through XWayland, which is available — `DISPLAY=:0`). `WEBKIT_DISABLE_COMPOSITING_MODE=1` avoids a related GL compositing issue.
- Window's `class` in `hyprctl clients` is `Vuln-hound` (lowercase h), not the display title `Vuln-Hound`.
- `npm run tauri dev` backgrounded with `nohup ... &` detaches from the launching shell — check via `ps aux | grep vuln-hound` / `hyprctl clients`, not shell job control.
