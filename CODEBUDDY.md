# CODEBUDDY.md

This file provides guidance to CodeBuddy Code when working with code in this repository.

## Project Overview

PortView is a cross-platform network connection monitoring application built with Tauri 2, Vue 3, and TypeScript. It provides real-time visualization of TCP/UDP connections with process information, icons, and management capabilities.

## Build Commands

```bash
# Install dependencies
pnpm install

# Development (runs both frontend and Tauri)
pnpm tauri dev

# Frontend only (for web development)
pnpm dev

# Production build
pnpm tauri build

# Type check frontend
pnpm build  # runs vue-tsc --noEmit && vite build

# Rust backend
pnpm tauri build -- --release  # Build Rust backend in release mode
cargo check --manifest-path src-tauri/Cargo.toml  # Quick Rust type check
cargo clippy --manifest-path src-tauri/Cargo.toml  # Rust linter
cargo test --manifest-path src-tauri/Cargo.toml  # Run Rust tests (if any)
```

## Architecture

### Frontend (Vue 3 + TypeScript)

- **Entry point**: `src/main.ts`
- **Main component**: `src/App.vue` - handles connection state, filtering, and coordinates all child components
- **Components** (`src/components/`):
  - `ConnectionsTable.vue` - sortable table displaying network connections
  - `MenuBar.vue` - filtering, search, theme toggle, and controls
  - `StatusBar.vue` - connection statistics and refresh info
  - `ContextMenu.vue` - right-click actions for processes
  - `ProcessDetailsModal.vue` - detailed process information dialog
  - `AboutDialog.vue` - application about dialog
  - `MessageBox.vue` - confirmation dialogs

- **Internationalization**: `src/i18n.ts` with locale files in `src/locales/` (`zh-CN.json`, `en-US.json`)
- **Path alias**: `@` maps to `src/`

### Backend (Rust + Tauri)

- **Entry point**: `src-tauri/src/lib.rs`
- **Library name**: `portview_lib` (uses `staticlib`, `cdylib`, `rlib` crate types for Windows compatibility)

#### Tauri Commands

| Command | Description |
|---------|-------------|
| `get_connections` | Retrieves all active TCP/UDP connections with process info |
| `get_process_details(pid)` | Returns detailed process information (memory, CPU, command line) |
| `kill_process(pid)` | Terminates a process by PID |
| `open_folder(path)` | Opens the directory containing a process executable |
| `get_app_version` | Returns the application version |
| `update_window_theme` | Theme update placeholder |

#### Key Dependencies

- `netstat2` - network connection enumeration
- `sysinfo` - process information retrieval
- `base64`, `image` - icon encoding/processing
- `md5` - cache key generation for icons
- `lazy_static` - global cache initialization
- `dirs` - cross-platform home directory resolution

#### Data Structures

The frontend and backend communicate via these Rust structs (serialized to JSON):

```rust
struct TcpConnection {
    protocol: String,      // "TCP" or "UDP"
    local_addr: String,
    local_port: u16,
    remote_addr: String,   // "*" for UDP
    remote_port: u16,      // 0 for UDP
    state: String,         // TCP state or "UNCONN" for UDP
    pid: Option<u32>,
    process_name: Option<String>,
    icon: Option<String>,  // Base64 encoded PNG
    start_time: Option<u64>,
}

struct ProcessDetails {
    pid: u32,
    name: String,
    command_line: String,
    executable_path: String,
    memory_usage: u64,
    cpu_usage: f32,
    parent_pid: Option<u32>,
    start_time: u64,
}
```

### Platform-Specific Code

The Rust backend uses conditional compilation for platform differences:

- **Windows** (`#[cfg(target_os = "windows")]`):
  - Uses WinAPI for process icon extraction via `ExtractIconW`
  - Converts icons to PNG with BGRA→RGBA conversion and vertical flip
  - Process termination via `TerminateProcess`
  - Requires `winapi` crate with specific features (see `Cargo.toml`)

- **macOS** (`#[cfg(target_os = "macos")`):
  - Icon extraction from `.app` bundles via ICNS→PNG conversion using `sips` command
  - Uses `tempfile` crate for temporary file handling
  - Process termination via `kill` command (SIGTERM/SIGKILL)

- **Linux** (`#[cfg(target_os = "linux")`):
  - Icon lookup from system theme directories and `.desktop` files
  - Searches `/usr/share/icons/` and `/usr/share/pixmaps/`
  - Process termination via `kill` command (SIGTERM/SIGKILL)

### Icon Caching

The backend implements a two-level caching system for process icons:

1. **File cache**: `~/.portview/` directory stores PNG files with MD5-hashed filenames
2. **Memory cache**: `ICON_CACHE` HashMap stores base64-encoded icons with timestamps

- Cache is preloaded on startup from `~/.portview/*.png` files
- Missing icons are cached as `(None, timestamp, false)` to avoid repeated extraction attempts
- This significantly improves performance when refreshing connection lists

### Data Flow

1. Frontend calls Tauri command (e.g., `invoke('get_connections')`)
2. Rust backend queries `netstat2` for socket info
3. Backend enriches data with process info from `sysinfo`
4. Platform-specific icon extraction happens per-process
5. Data serialized to JSON and returned to frontend
6. Vue components reactively update the UI

## Key Files

- `src-tauri/tauri.conf.json` - Tauri configuration (window size, build settings, bundle targets)
- `src-tauri/Cargo.toml` - Rust dependencies with platform-specific sections
- `vite.config.ts` - Vite configuration with `@` path alias

## Development Notes

- Frontend dev server runs on port 1420 (strict port mode)
- Vite ignores `src-tauri/` directory for file watching
- CSP is disabled in tauri.conf.json for development flexibility
- Bundle target is `nsis` for Windows installer
