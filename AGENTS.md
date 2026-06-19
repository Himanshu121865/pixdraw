# Opendraw — Agent Memory File

## Project Overview
Terminal-based pixel art editor built with Rust + Ratatui + Crossterm.
Single-window, split into: header (tabs/info) → canvas → palette bar → status bar.

## Feature Inventory

### Drawing Tools
- Brush (left click draw, right click erase)
- Line tool (`l` key)
- Rectangle tool (`r`)
- Circle tool (`c`)
- Spray tool (`s`) — adjustable density with `+`/`-`
- Flood fill (`f`)
- Eyedropper (`i`) — pick colour from canvas pixel
- Eraser (`e`)
- Gradient fill (`g`)
- Symmetry mode (`y`) — mirrors across axes
- Selection tool (`v`) — rectangular select, copy/cut/paste
- Text overlay (`t`) — type text, Enter to commit, Esc to cancel
- **Cellular automaton** (`L`) — click to seed, right-click to run GoL generation
- **Posterize** (`P`) — quantise canvas to 8 most common colours
- **Terminal preview** (`Ctrl+P`) — renders pixel-perfect PNG inline (Kitty protocol)

### Colour Management
- Palette bar (14 named colours) at bottom of screen
- Colour picker popup (`Tab` while no popup is open)
- Colour selector (`u` — generates 3 harmonious colours)
- Custom RGB input
- Colour history swatches (last 8 used) in status bar
- Independent colour for each `[ ]` fill quadrant in symmetry

### File Operations
- Save (`Ctrl+S`) — JSON format preserving all drawing data
- Load (`Ctrl+O`) — JSON restore
- Export PNG (`Ctrl+E`) — auto-named `opendraw_export.png`
- Session persistence (`Ctrl+D`) — save/load/delete sessions

### Canvas
- Resizable (`Ctrl+R`) — width/height input
- Grid toggle (`Ctrl+G`)
- Scrolling when canvas larger than terminal (scroll lock toggle: `Space`)

### UX
- Undo/redo (`Ctrl+Z` / `Ctrl+Y`)
- Tab management — multiple canvases, rename (`Ctrl+N` while on tab), close (`Ctrl+W`)
- Context menu (right click) — stamp erase, clear all, copy/paste selection, select all
- Help popup (`?` or `F1`) — categorised keybind reference
- Start-up dialog on first launch

### Popup Order (priority)
1. File browser (Save/Load) — highest priority, blocks everything
2. Tab rename — blocks most keys
3. Colour picker, selector, input — modal on palette area
4. Help, context menu, canvas resize, startup dialog — overlay, dismiss on click-outside

## CLI Tools
- `img2opendraw` — convert any image to opendraw .txt format with exact RGB colours:
  ```sh
  cargo run --bin img2opendraw -- photo.jpg --width 80
  ```
  Options: `--width W`, `--max-dim N` (cap auto-size, default 187), `--output FILE`,
  `--contrast F` (1.0=none), `--flip (h|v)`, `--stats`.
  If one dimension is given, the other is auto-scaled. Height auto-computed from aspect ratio. No `--height` flag.
  No palette quantisation — outputs exact image colours as `R x y r g b` lines.

## Recent Changes (Session History)
- Refactored monolithic codebase into modular structure (app/, ui/ submodules)
- Added learning-oriented comments throughout all files
- Replaced help menu toggle keys (1-8) with selection cursor (j/k + Enter/Space)
- Swapped Tab/Ctrl+Tab: Tab=next tab, Ctrl+Tab=colour picker
- Clear canvas (`c`) now also clears text_entries
- Text mode auto-exit on Enter; Esc resets all tool modes
- Removed 't' as text mode cancel (now types normally)
- Bugfix: session restore preserves current_tab index
- Bugfix: Esc handler resets ALL drawing modes (eyedropper, eraser, fill, symmetry, select, gradient, shape_preview, etc.)
- Bugfix: header stale-block rendering (bg(SURFACE) on Paragraph)
### Misc
- **Help search** (`?`/`/`): search is active by default; type to filter keybinds, Esc exits search to browse mode, another Esc closes help. Typing any printable char in browse mode reactivates search.
- Clippy lint fixes: collapsible if/match guards, redundant closure, len_zero
- **Cellular automaton** (`L`): toggle life mode, left-click to seed random cells, right-click to run one Game of Life generation, auto-advances ~6 gen/s while active
- **Posterize** (`P`): reduces canvas to 8 most common colours via RGB distance matching
- **Quick export** (`Ctrl+P`): exports `opendraw_export.png` without file browser dialog
- **Exact RGB image pipeline**: `img2opendraw` outputs `R x y r g b` lines (no palette quantisation), in-app image loading stores `Color::Rgb` directly. Removed LAB palette-matching + Floyd-Steinberg dithering.

## Known Limitations
- Undo does NOT track text_entries — only points (BTreeMap)
- PNG export uses bounding box, not full canvas dimensions
- Flood fill uses DFS recursion (risk of stack overflow on very large fills)
- No layer system (single layer per tab)
- No undo history limit (memory grows unbounded)
- No zoom
- No custom brush sizes
- Session format uses custom text serialisation (not JSON for sessions, JSON for files)
- Help popup doesn't auto-scroll when content exceeds height
- No export with transparency

## Code Conventions
- Comments: `// ── Section header ──` style, learning-oriented `// why` comments
- Visibility: `pub(crate)` for cross-module access, `pub` only for re-exports
- DrawingApp fields accessed via Deref/DerefMut to TabData
- No unwrap() in normal code paths; prefer `?` or match
- Colour constants in ui/col.rs (Tokyo Night theme)
- Widget trait imported as `ratatui::widgets::Widget`
- bg(SURFACE) used to clear header info bar background
- `saturating_sub` / `saturating_add` for all terminal coordinate math

## Config
- No config file — colours, keybinds hardcoded
- Font: terminal font only (ASCII-based UI)
- No plugin/extension system
