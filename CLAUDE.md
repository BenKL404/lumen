# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Lumen is a Linux video player inspired by PotPlayer, built with **Rust + CXX-Qt 0.7 + QML (Qt 6) + libmpv**. Status: v0.1 skeleton.

- `README.md` — setup, architecture, roadmap, known pitfalls (French).
- `FONCTIONNALITES.md` — user-facing feature list, design identity (colors, spacing, animation timings) and the v0.2 → v1.0 roadmap. Check it before implementing UI or roadmap features so the result matches the planned design.

All docs, code comments and UI strings are in French; keep that convention.

## Commands

Run from the repo root:

```bash
cargo run                      # debug build + launch
cargo run --release
cargo test                     # unit tests (pure Rust helpers in src/bridge/utils.rs)
cargo test formats_time        # single test by name
```

- Requires Rust ≥ 1.85, Qt 6 dev packages (base, declarative, OpenGL) and `libmpv-dev`.
- CXX-Qt needs Qt 6's `qmake`; `.cargo/config.toml` sets `QMAKE=/usr/bin/qmake6` for every cargo command.
- First build is slow (CXX-Qt generates and compiles a lot of C++).

## Architecture

Three layers, all wired together by `build.rs` (`cxx-qt-build`):

1. **QML UI** (`qml/`) — compiled into the QML module `com.lumen.player`, loaded from `qrc:/qt/qml/com/lumen/player/qml/Main.qml`. `Main.qml` owns the window, the `theme` design-token object (single source for colors/radius/animation duration), keyboard shortcuts, drag-and-drop, auto-hide timer and OSD. Child components receive the theme from it.
2. **Rust (CXX-Qt)** (`src/bridge/`) — application logic. `utils.rs` exposes the `Utils` QML element (`formatTime`, `fileName`); the pure-Rust functions are kept separate from the `qobject` impl so they can be unit-tested without Qt. `history.rs` exposes `History` (resume playback: position, speed, audio/sub track, sub delay, manually added sub file; SQLite via bundled `rusqlite` in `$XDG_DATA_HOME/lumen/history.db`, keyed by decoded local path, schema migrations via `PRAGMA user_version`; QML saves every 5 s, on pause, on file change and on close; `load()` fills Q_PROPERTYs that `Main.qml`'s `resume()` reads). `playlist.rs` exposes `Playlist` (editable `Items` model tracking the current file by path, with `nextIndex`/`previousIndex`; durations probed off-thread via `cpp/probe.cpp` (libavformat) and queued back with cxx-qt `Threading`; auto playlist: same-folder files sharing the series key from `series_key()` — SxxEyy / Épisode / Partie / CD… markers or last lone number — sorted with `natural_cmp`; `items` are encoded file:// URLs; `Main.qml` calls `playlist.load()` on every open and advances on `MpvVideo.eofReached`). `settings.rs` exposes `Settings` (`$XDG_CONFIG_HOME/lumen/settings.toml` via serde/toml; missing keys default, out-of-range values clamped, unreadable file → defaults; atomic write via tmp + rename). `Main.qml` applies it in `Component.onCompleted` and writes it back in `saveSettings()` on close; window width is stored without the attached playlist width. `video.rs` and `app.rs` are plain FFI bridges to C++ helpers (`lumen_init_video()`, and `lumen_qml_loaded()` for Qt APIs `cxx-qt-lib` doesn't expose).
3. **C++ `MpvItem`** (`cpp/mpvitem.*`) — a `QQuickFramebufferObject` that renders libmpv into the Qt Quick scene. Registered manually as `MpvVideo` in module `com.lumen.video` (not via the CXX-Qt QML module). Exists only because subclassing `QQuickFramebufferObject` isn't feasible from CXX-Qt. It exposes a few observed properties (`position`, `duration`, `paused`, `volume`, `speed`, `mediaTitle`, `hasMedia`, `subDelay`, and `tracks` = mpv `track-list` converted from `mpv_node` to a QVariantList) plus a generic `command([...])` that forwards any mpv command asynchronously.

**Rule: keep the C++ layer a thin rendering/command pipe. Any feature that doesn't touch rendering goes in Rust (`src/`), not `cpp/`.** New mpv state needed in QML is either queried via `command(...)` or added as a new `mpv_observe_property` + `Q_PROPERTY` in `MpvItem`.

Window layout (`Main.qml`): frameless window with a QML `TitleBar` (drag via `startSystemMove`, edge `ResizeEdge`s via `startSystemResize`); in windowed mode the `ControlBar` sits below the video and the `PlaylistPanel` is docked right, shrinking the `stage` item; in fullscreen both float over the video and auto-hide. `AppMenu` is the shared dropdown (title-bar « Lumen ⌄ », right-click context menu, playlist « Ajouter »); the control-bar sliders button opens `ImagePanel`. Glyph icons must exist in the system fonts — check with `fc-list ":charset=<hex>"` (e.g. 📌 renders as tofu here).

`main.rs` sets `QT_QPA_PLATFORMTHEME=xdgdesktopportal` when the variable is unset or `qt5ct` (Qt5-only), so `FileDialog`/`FolderDialog` use the desktop's native picker through the XDG portal (needs `qt6-xdgdesktopportal-platformtheme`). `AppMenu` panel height is computed from its entries, not from the column, so `popup(anchor, above)` positions correctly on first open.

`lumen <file|folder>` opens a file (or a whole folder as the playlist) at startup. For visual checks without screen capture: `LUMEN_SNAPSHOT=out.png [LUMEN_SNAPSHOT_DELAY=ms] cargo run -- video.mkv` saves the window image then closes the window (a normal close: position and settings are saved, as with ✕) (set `XDG_DATA_HOME` and `XDG_CONFIG_HOME` to temp dirs to keep the real history and settings untouched). Prefer `QT_QPA_PLATFORM=offscreen` for UI-only checks: an on-screen window steals keyboard focus from the user (their keystrokes trigger Lumen shortcuts); offscreen has no GL, so no video frame. `MpvItem` queues `loadFile` until the GL render context exists, otherwise mpv starts without video output; if GL never comes up (2 s, e.g. `QT_QPA_PLATFORM=offscreen`) it loads anyway with `vid=no`, so offscreen snapshots still get duration, chapters and tracks (no picture). mpv options like `ab-loop-*`, `sub-delay`, `video-aspect-override` persist across files — reset them in `onFileLoaded`/`resume()` when they should be per file.

Adding files requires registering them in `build.rs`: QML files and Rust QObject files in the `QmlModule` (`qml_files` / `rust_files`), other CXX bridges via `.file(...)`, C++ sources in `cc_builder`.

### Threading / mpv constraints

- `lumen_init_video()` must be called in `main.rs` **before** `QGuiApplication` is created: it forces the OpenGL scene-graph backend (the renderer is GL-only) and registers `MpvVideo`.
- mpv callbacks (wakeup, redraw) arrive on mpv threads; they are marshalled to the GUI thread with `QMetaObject::invokeMethod(..., Qt::QueuedConnection)`. `MpvRenderer` runs on the Qt Quick render thread and must restore GL state (`QQuickOpenGLUtils::resetOpenGLState()`) after mpv draws.
- libmpv requires `LC_NUMERIC=C`; this is set in the `MpvItem` constructor — don't change locale elsewhere.
- `main.rs` exits with code 1 if the QML root fails to load (otherwise Qt would keep running with no window). Missing `qml6-module-*` packages only show up at this point, not at build time.
- The seek bar only sends the seek on release, to avoid flooding mpv during drags.
- Haruna (KDE) uses the same Qt Quick + libmpv approach and is a useful reference.

## graphify

This project has a knowledge graph at graphify-out/ with god nodes, community structure, and cross-file relationships.

Rules:
- For codebase questions, first run `graphify query "<question>"` when graphify-out/graph.json exists. Use `graphify path "<A>" "<B>"` for relationships and `graphify explain "<concept>"` for focused concepts. These return a scoped subgraph, usually much smaller than GRAPH_REPORT.md or raw grep output.
- If graphify-out/wiki/index.md exists, use it for broad navigation instead of raw source browsing.
- Read graphify-out/GRAPH_REPORT.md only for broad architecture review or when query/path/explain do not surface enough context.
- After modifying code, run `graphify update .` to keep the graph current (AST-only, no API cost).
