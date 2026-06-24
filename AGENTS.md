# EasyLabel Tauri — Agent Guide

## First read

**`SPEC.md` is the accurate spec** — the `README.md` is stale (describes an old Slint-based UI that no longer exists).

## Project type

Tauri 2.x desktop app: **Vue 3 + TypeScript** frontend (Vite-bundled) + **Rust** backend via Tauri IPC.

## Developer commands

| Command | What it does |
|---------|-------------|
| `npm run dev` (root) | Starts Vite dev server (port 1420) |
| `npm run build` (root) | Runs `vue-tsc --noEmit && vite build` |
| `npm run tauri` (root) | Invokes `@tauri-apps/cli` (not global `tauri`) |
| `cd frontend && npm run dev` | Frontend-only dev |
| `cargo build` (root workspace) | Builds Rust backend |

**No tests**, **no linters**, **no CI** — don't look for them.

## Architecture

- **State**: Pinia stores (`app.ts` for annotations/tools, `project.ts` for tasks)
- **IPC bridge**: ~22 Tauri commands registered in `src-tauri/src/main.rs`, called via `@tauri-apps/api/core` `invoke()`
- **Canvas**: `frontend/src/components/Canvas.vue` (~2100 lines) — handles all annotation drawing, SVG overlay, pan/zoom for all 6 annotation types
- **All coordinates are normalized [0,1]** on both Rust and TypeScript sides
- **Persistence**: JSON files only — task list in OS data dir, per-image annotations as sidecar `.annotations.json` files

## Key conventions

- Frontend: Vue 3 Composition API + TypeScript strict mode, `@/` path alias to `./frontend/src`
- UI: Naive UI component library + lucide-vue-next icons, dark theme (CSS vars in `assets/main.css`)
- Custom title bar (`AppHeader.vue`) — window has `decorations: false`, uses `@tauri-apps/api/window`
- Path alias `@` maps to `./src` (vite.config.ts)
- Build target: chrome105 (Windows) / safari13 (others)

## Tauri 2 specifics

- Permissions/capabilities in `src-tauri/capabilities/default.json` (not `tauri.conf.json`)
- Plugins: `dialog`, `fs` — must be registered in both Rust (`main.rs`) and capabilities
- `convertFileSrc()` for image display (images served via asset protocol, not base64 IPC)
- `src-tauri/gen/schemas/` is auto-generated — do not edit

## Settings system

- Settings persisted via IPC commands (`load_settings`/`save_settings`) as JSON in OS data dir
- Pinia store at `stores/settings.ts` with auto-save on change (deep watcher)
- Dynamic theme: `App.vue` computes Naive UI `themeOverrides` reactively from `settings.accent_color`
- Accent color updates CSS custom properties `--accent` and `--accent-dim` on `document.documentElement`
- Dense mode adjusts CSS layout variables (`--topbar-height`, `--leftbar-width`, `--statusbar-height`)
- Settings modal component: `components/SettingsModal.vue`, triggered from gear icon in `AppHeader.vue`
- Add new settings by: (1) extend `AppSettings` in Rust `models.rs`, (2) add field to Pinia store defaults, (3) add UI in `SettingsModal.vue`

## Canvas.vue

~2400 lines, handles all drawing logic for all 6 annotation types. Contains pre-existing TypeScript strict-mode type comparison issues that don't block dev/build (only `vue-tsc` type-check). 

### Box (AxisAlignedBox) drawing
- Preview div (HTML overlay, not SVG) follows mouse during drag
- Minimum drag threshold (5px) prevents accidental clicks
- Auto-saves on annotation creation/update/removal

### RotatedBox
- 3-step drawing: click point1 → drag angle → click height
- Resize via 4 corner handles (tl/tr/bl/br), mouse-based positioning
- Rotation handle (circle + line above top center), dragged via angle from annotation center
- Fixed-corner math ensures opposite corner stays stationary during resize
- Aspect ratio correction (`ch/cw`) applied to all rotation formulas

### Keypoint
- Two-phase: place corner points → draw bounding box
- Keypoint vertices draggable (constrained within bounding box)
- Bounding box resizable (8 handles) and movable
- Push-to-fit via Enter in corner phase

### Polygon (Segmentation)
- Click to add vertices, click start point to close
- Preview uses `<polyline>` (open lines) during drawing, no fill
- Completed annotation uses `<path>` with `fill-rule="evenodd"` to support holes
- Bounding box rect (solid line) + label tag displayed
- Semi-transparent filled region at all times
- Minimum 3 points to close (click near first point, Enter, or double-click)
- Holes supported via `H` key toggle:
  - Select a polygon, press `H` to enter hole mode
  - Draw inner polygons that become holes (evenodd rendering)
  - Holes validated via ray-casting point-in-polygon check
  - Invalid holes show warning indicator on canvas
- Both outer vertices and hole vertices are draggable

### OCR
- Two modes (toggle with **T**): quadrilateral (click 4 points) or rectangle (drag)
- Rectangle mode reuses the box preview div for visual feedback
- Quadrilateral mode works like polygon: click to place points, click start point to close
- Text input popup appears after closing, preview switches to closed `<polygon>` during popup
- Follow line from last point to cursor during drawing
- First point enlarges on mouse proximity for easy closing
- Max 4 points for quadrilateral
- OCR mode indicator banner at top of canvas
- Editing: rect mode uses box-style corner handles (4 corners + center move), quad mode uses individual vertex handles
- Rect mode auto-detected by checking if 4 points form an axis-aligned rectangle

### Right panel
- Three sections (image list, classes, annotation list) equally divide available height
- Each section has independent scrollbar when content overflows
- Title row is fixed (`flex-shrink: 0`), content area scrolls

### Critical bugs found & fixed during development
| Bug | Fix |
|-----|-----|
| `data-kp-annId` vs `data-kp-ann-id` (capitalization typo) | Changed JS to match template attribute name |
| Empty `else if (type == Keypoint)` / `else if (type == Polygon)` blocked real handlers | Removed empty branches |
| Rotation matrix used counterclockwise formula instead of SVG's clockwise | Corrected to `x*cos + y*sin` for width axis |
| `dx/dy` already normalized (÷ `displayW/H`) but resize divided by `cw/ch` again (double normalization) | Removed extra division |
| Missing aspect ratio (`ch/cw`) in rotation formulas for corner positions | Added `aspect` factor to all corner + unrotate computations |
| Extra `}` in `onMouseMove` prematurely closed the function | Removed the extra brace |
| `onMouseMove` `drag.ann.type` crashed when `drag.ann` was null | Added optional chaining `drag.ann?.type` throughout |
| `stopPropagation()` in `onAnnotationMousedown` blocked drawing inside existing annotations | Moved after tool check |
| Polygon changed from `<polygon>` to `<path>` broke drag handler | Added `tag === "path"` and `pointer-events:none` on path |
| RotatedBox resize handles missing aspect ratio correction for center offset | Rewrote with fixed-corner math using explicit `ch/cw` factor |
| OCR quadrilateral proximity check double-added container offset | Removed `crect.left + imgLeft` from `fpClientX` calculation |
| `onKeyDown` T key handler nested inside `if (e.key === "Enter")` brace | Restructured keydown handler with proper brace matching |
| `tag === "polygon"` no longer matches after `<path>` conversion | Added `tag === "path"` case to annotation mousedown handler |

## Canvas rendering quirks
- SVG viewBox aspect ratio `cw:ch` (image dimensions) requires `aspect = ch/cw` in all rotation math
- `mouseToImage()` returns image PIXEL coordinates; divide by `cw/ch` to get normalized [0,1]
- `dx/dy` at line 1071 are ALREADY normalized by `displayW/displayH`
- Preview uses HTML div overlay (not SVG rect) to bypass Vue reactivity issues
- Box preview created/destroyed via direct DOM API (`createElement`, `removeChild`)

## Things to avoid

- Don't edit `src-tauri/gen/` (auto-generated)
- Don't edit `frontend/dist/` (build output)
- Don't trust the `README.md` content — use `SPEC.md` instead
- `open` command in Rust is not a standard library function; app uses `dirs` crate for paths
