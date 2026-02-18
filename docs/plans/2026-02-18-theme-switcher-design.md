# Theme Switcher Design

**Goal:** Add a theme switcher dropdown to the connection panel with three nostalgic Windows aesthetics.

**Architecture:** CSS custom properties per theme. A class on the root `<div>` activates the theme. All hardcoded colours in components become `var(--x)` references. Theme choice persists to `localStorage`.

**Tech Stack:** CSS custom properties, Dioxus signals, `localStorage` via `web_sys` or `dioxus_desktop` eval.

---

## Theme Switcher UI

A `<select>` dropdown sits above the connect button in the connection panel. Options:

- `wmp` → "🎵 WMP 2008"
- `win2k` → "🖥 Win2K High Contrast"
- `win7` → "🪟 Win7 Aero"

A `theme` signal in `App` holds the current theme key. On change it updates the signal and writes to `localStorage`. On startup it reads `localStorage` (defaulting to `wmp`).

---

## CSS Variable Contract

Every component references these variables instead of hardcoded values:

| Variable | Purpose |
|---|---|
| `--bg-primary` | Main background |
| `--bg-secondary` | Panel / toolbar background |
| `--bg-tertiary` | Input / log background |
| `--border` | Default border colour |
| `--text-primary` | Primary text |
| `--text-dim` | Secondary/label text |
| `--accent` | Primary accent (buttons, links) |
| `--accent-text` | Text on accent-coloured backgrounds |
| `--danger` | Disconnect / error colour |
| `--success` | Streaming / ok colour |
| `--warning` | Reconnecting / in-progress colour |
| `--log-error-bg` | Error log row background |
| `--font-ui` | UI font stack |
| `--font-mono` | Monospace font stack |
| `--radius` | Default border-radius |
| `--btn-gradient` | Button background (gradient or flat) |
| `--btn-shine` | Button top-highlight (WMP gloss) |
| `--panel-bg` | Glass panel background (Win7) |
| `--panel-blur` | backdrop-filter value (Win7) |
| `--panel-shadow` | box-shadow on panels |

---

## Theme Specs

### WMP 2008

```css
.theme-wmp {
  --bg-primary: #080c14;
  --bg-secondary: #0d1520;
  --bg-tertiary: #060a10;
  --border: #1a3a5c;
  --text-primary: #e0e8f0;
  --text-dim: #5a7a9a;
  --accent: #00b4d8;
  --accent-text: #000;
  --danger: #ff4444;
  --success: #00d4aa;
  --warning: #ffa500;
  --log-error-bg: #1a0a0a;
  --font-ui: 'Segoe UI', Arial, sans-serif;
  --font-mono: 'Consolas', 'Courier New', monospace;
  --radius: 2px;
  --btn-gradient: linear-gradient(180deg, #1a6a8a 0%, #0a3a5a 50%, #062a4a 100%);
  --btn-shine: linear-gradient(180deg, rgba(255,255,255,0.25) 0%, rgba(255,255,255,0) 50%);
  --panel-bg: transparent;
  --panel-blur: none;
  --panel-shadow: 0 0 8px rgba(0, 180, 216, 0.2), inset 0 1px 0 rgba(0,180,216,0.15);
}
```

Buttons have a gloss effect: a `::before` pseudo-element with `--btn-shine` overlaid on `--btn-gradient`. Borders emit a subtle teal glow via `box-shadow`.

### Win2K High Contrast

```css
.theme-win2k {
  --bg-primary: #000000;
  --bg-secondary: #000000;
  --bg-tertiary: #000000;
  --border: #ffffff;
  --text-primary: #ffffff;
  --text-dim: #ffffff;
  --accent: #ffff00;
  --accent-text: #000000;
  --danger: #ff0000;
  --success: #00ff00;
  --warning: #ffff00;
  --log-error-bg: #200000;
  --font-ui: 'MS Sans Serif', 'Arial', sans-serif;
  --font-mono: 'Courier New', monospace;
  --radius: 0px;
  --btn-gradient: #000000;
  --btn-shine: none;
  --panel-bg: transparent;
  --panel-blur: none;
  --panel-shadow: none;
}
```

Raised 3D borders on buttons: `border-top: 2px solid #fff; border-left: 2px solid #fff; border-bottom: 2px solid #808080; border-right: 2px solid #808080`. No gradients. No border-radius anywhere.

### Win7 Aero

```css
.theme-win7 {
  --bg-primary: #1a2535;
  --bg-secondary: rgba(255,255,255,0.06);
  --bg-tertiary: rgba(0,0,0,0.4);
  --border: rgba(255,255,255,0.15);
  --text-primary: #ffffff;
  --text-dim: #a0b4c8;
  --accent: #3399ff;
  --accent-text: #ffffff;
  --danger: #ff5555;
  --success: #55cc88;
  --warning: #ffaa33;
  --log-error-bg: rgba(180,0,0,0.15);
  --font-ui: 'Segoe UI', Arial, sans-serif;
  --font-mono: 'Consolas', 'Courier New', monospace;
  --radius: 6px;
  --btn-gradient: linear-gradient(180deg, rgba(255,255,255,0.18) 0%, rgba(255,255,255,0.04) 100%);
  --btn-shine: none;
  --panel-bg: rgba(255,255,255,0.06);
  --panel-blur: blur(12px);
  --panel-shadow: 0 4px 16px rgba(0,80,200,0.25), inset 0 1px 0 rgba(255,255,255,0.12);
}
```

Panels use `backdrop-filter: var(--panel-blur)` and `background: var(--panel-bg)`. Soft rounded corners everywhere. Buttons are translucent glass.

---

## Component Migration

All inline `style="..."` colour/font values in these files become `var(--x)` references:

- `styles.css` — new theme classes + base reset
- `components/status_indicator.rs` — colours
- `components/connection_panel.rs` — bg, buttons, border, label
- `components/stats_header.rs` — bg, text, accent
- `components/filter_bar.rs` — bg, input, buttons, tags
- `components/log_view.rs` — bg, text, level colours
- `main.rs` — root div gets `class: "theme-{theme}"`, theme signal + localStorage

## localStorage

On startup: read `localStorage.getItem("theme")`, default to `"wmp"`.
On change: `localStorage.setItem("theme", value)`.
Use `dioxus_desktop`'s `eval()` to call JS for localStorage access.
