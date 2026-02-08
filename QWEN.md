# PortView Project Context

## Project Overview

This is a cross-platform desktop application called "PortView" built using the Tauri framework. It combines:
- **Frontend**: Vue 3 with TypeScript, using Vite as the build tool
- **Backend**: Rust for system-level operations (specifically for network connection monitoring)
- **Architecture**: Tauri enables the combination of web technologies (Vue) with Rust for native desktop applications

The project is designed for monitoring TCP/UDP connections on the system, displaying detailed information about active network connections including protocol, local/remote addresses and ports, connection state, process ID (PID), and process name.

## Key Technologies and Dependencies

- **Rust/Cargo**: Backend logic and system operations using `netstat2` and `sysinfo` crates for network connection monitoring
- **Vue 3**: Frontend component framework with TypeScript
- **TypeScript**: Type-safe JavaScript for the frontend
- **Tauri**: Framework for creating native desktop apps with web UI
- **Vite**: Modern build tool and development server
- **pnpm**: Package manager

## Project Structure

```
tcpview/
├── src/                    # Vue 3 frontend source
│   ├── App.vue            # Main application component with connection table
│   ├── main.ts            # Vue application entry point
│   ├── i18n.ts            # Internationalization setup
│   ├── assets/            # Static assets
│   ├── locales/           # Language files for i18n
│   ├── components/        # Vue components
│   │   ├── ConnectionsTable.vue
│   │   ├── ContextMenu.vue
│   │   ├── MenuBar.vue
│   │   ├── ProcessDetailsModal.vue
│   │   └── StatusBar.vue
│   └── vite-env.d.ts      # Vite environment type definitions
├── src-tauri/             # Rust backend source
│   ├── src/
│   │   └── lib.rs         # Rust command implementations (get_connections, get_process_details, kill_process, greet)
│   ├── Cargo.toml         # Rust dependencies and configuration
│   └── tauri.conf.json    # Tauri application configuration
├── public/                # Static assets
├── package.json           # Node.js dependencies and scripts
└── vite.config.ts         # Vite build configuration
```

## Building and Running

### Development Mode
```bash
pnpm tauri dev
```
This starts the development server with hot reloading.

### Production Build
```bash
pnpm tauri build
```
This creates an optimized production build of the application.

### Alternative Commands
- `pnpm dev` - Run the Vite development server only
- `pnpm build` - Build the frontend only (without Tauri wrapper)

## Features

The application provides:
- Real-time display of network connections (TCP/UDP)
- Sorting capabilities by clicking on column headers
- Adjustable column widths by dragging column edges
- Filtering by protocol (TCP/UDP) and connection state
- Search functionality for process names and local addresses
- Detailed connection information including:
  - Protocol (TCP/UDP)
  - Local address and port
  - Remote address and port
  - Connection state
  - Process ID (PID)
  - Process name
  - Process start time
  - Process icons (Windows only)
- Right-click context menu with options to view process details or kill the process
- Double-click to view detailed process information in a modal dialog
- Status bar showing connection statistics
- Internationalization support (Chinese and English)

## Development Conventions

- **Frontend**: Vue 3 Composition API with `<script setup>` syntax and TypeScript
- **Backend**: Rust functions annotated with `#[tauri::command]` are accessible from the frontend
- **Communication**: Frontend communicates with backend via Tauri's `invoke()` mechanism
- **Data Types**: Shared interfaces between frontend and backend for type safety
- **Styling**: Component-scoped CSS with responsive design considerations and dark mode support
- **Internationalization**: Uses vue-i18n with language files in the locales directory

## Key Components

### Frontend (App.vue)
- Connection table with sorting and resizing capabilities
- Filtering and search functionality
- Right-click context menu for process actions
- Process details modal dialog
- Status bar with connection statistics
- Responsive layout using flexbox
- Dark mode support
- Internationalization support

### Backend (src-tauri/src/lib.rs)
- `get_connections()` command to fetch network connection data
- `get_process_details()` command to fetch detailed process information
- `kill_process()` command to terminate processes
- Integration with `netstat2` crate for socket information
- Integration with `sysinfo` crate for process information
- Cross-platform process icon retrieval (Windows only currently)
- Data transformation and mapping to frontend-compatible types