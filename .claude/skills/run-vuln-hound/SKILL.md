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

## Click through the app (real mouse/keyboard input)

`ydotool` works for this — requires passwordless sudo to set up once per boot (ask the user to activate it: same flow as installing any package here).

```bash
# one-time per boot:
sudo pacman -S --noconfirm ydotool                 # if not already installed
sudo nohup ydotoold --socket-perm=0666 &            # MUST pass --socket-perm=0666, default 0600 blocks the normal user
export YDOTOOL_SOCKET=/tmp/.ydotool_socket          # needed in every shell that calls ydotool
```

Click at an exact screen position:

```bash
hyprctl cursorpos                                    # find current position
ydotool mousemove -x <dx> -y <dy>                     # RELATIVE move only — see gotcha below
hyprctl cursorpos                                     # check where it actually landed, correct with 1-2 more relative moves
ydotool click 0xC0                                    # left click, once cursor is exactly on target
```

For a small target (e.g. a 24px icon button), zoom in on the cursor before clicking to confirm placement:
```bash
grim -g "<x-50>,<y-50> 200x150" /tmp/check.png   # crop around expected position, then Read it
```

## Stop

```bash
pkill -f "target/debug/vuln-hound"; pkill -f "node .*vite"
sudo pkill ydotoold   # optional — harmless if left running
```

## Gotchas

- **`Gdk-Message: Error 71 (Protocol error)` on launch** — native Wayland webkit2gtk fails on this compositor. Fix: `GDK_BACKEND=x11` (routes through XWayland, which is available — `DISPLAY=:0`). `WEBKIT_DISABLE_COMPOSITING_MODE=1` avoids a related GL compositing issue.
- Window's `class` in `hyprctl clients` is `Vuln-hound` (lowercase h), not the display title `Vuln-Hound`.
- `npm run tauri dev` backgrounded with `nohup ... &` detaches from the launching shell — check via `ps aux | grep vuln-hound` / `hyprctl clients`, not shell job control.
- `ydotool mousemove --absolute` does **not** map to real pixel coordinates reliably on this compositor (pointer acceleration applies inconsistently — roughly 1:1 for small moves, up to ~1.8x for large ones). Always use `-x/-y` **relative** moves and verify with `hyprctl cursorpos` before clicking, correcting 1-2 times as needed.
- `wtype` (keyboard-only synthesis) does reach the app and produces visible focus rings on buttons, but synthetic Enter/Space did not trigger button activation in this webview. Use `ydotool click` for anything that needs to actually happen; `wtype` is only reliable for confirming focus routing.
- `hyprctl dispatch <name> <args>` and `hyprctl keyword ...` are broken on this machine's Hyprland config — a Lua config layer (Omarchy) intercepts them and errors (`"...your syntax might need to be updated"`). Don't spend time on these; `hyprctl cursorpos`, `hyprctl clients -j`, `hyprctl activewindow -j`, and `hyprctl monitors -j` (all read-only) work fine.
- Window/monitor layout can shift between launches (multi-monitor setup) — always re-check `hyprctl clients -j` for current position before computing click targets from a screenshot.
- The native folder-picker dialog spans the full virtual screen (both monitors) — screenshot it with plain `grim` (no `-g`) and account for the "displayed at WxH, multiply by N" scaling note the image viewer gives you when computing click coordinates, rather than assuming the app window's own crop geometry applies.
