# PortView - Network Connection Monitor

## Project Overview

PortView is a cross-platform network connection monitoring application built with Tauri, Vue 3, and TypeScript. The application provides real-time visualization of system TCP/UDP connections, allowing users to monitor active network connections, associated processes, and perform actions like killing processes or opening process directories.

### Key Technologies
- **Frontend**: Vue 3 with TypeScript, using Composition API
- **Backend**: Rust with Tauri framework
- **UI Framework**: Native OS windows via Tauri
- **Internationalization**: Vue I18n with Chinese and English support
- **Build Tool**: Vite

### Architecture
The application follows a client-server architecture where:
- The **frontend** (Vue 3) provides the user interface and handles user interactions
- The **backend** (Rust) accesses system-level network information and process details
- **Tauri** bridges the frontend and backend, providing secure communication between JavaScript and Rust

## Features

1. **Real-time Network Monitoring**: Displays active TCP/UDP connections with details like protocol, addresses, ports, and connection state
2. **Process Information**: Shows associated process names, PIDs, and icons for each connection
3. **Process Management**: Allows killing processes or opening their containing directories
4. **Filtering & Search**: Filter connections by protocol, state, process name, or local port
5. **Auto-refresh**: Automatic periodic updates of connection data
6. **Sorting**: Sort table columns by clicking headers
7. **Dark/Light Theme**: Support for system theme detection and manual theme switching
8. **Multi-language**: Chinese and English localization
9. **Process Details Modal**: Detailed view of process information including memory usage, CPU usage, and command line

## Building and Running

### Prerequisites
- Node.js (with pnpm package manager)
- Rust and Cargo
- Tauri CLI

### Development Setup
```bash
# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Or separately:
pnpm dev  # Frontend only (for web development)
```

### Production Build
```bash
# Build the application
pnpm tauri build

# Alternative: Build frontend first, then Tauri
pnpm build
pnpm tauri build
```

### Available Scripts
- `pnpm dev`: Start development server
- `pnpm build`: Build frontend for production
- `pnpm preview`: Preview production build
- `pnpm tauri`: Run Tauri CLI commands

## Project Structure

```
portview/
├── src/                    # Vue 3 frontend source
│   ├── App.vue            # Main application component
│   ├── main.ts            # Application entry point
│   ├── i18n.ts            # Internationalization setup
│   ├── components/        # Vue components
│   └── locales/           # Translation files
├── src-tauri/             # Rust backend source
│   ├── src/
│   │   └── lib.rs         # Main Rust implementation
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── public/                # Static assets
├── index.html             # HTML template
├── package.json           # Node.js dependencies and scripts
├── vite.config.ts         # Vite build configuration
└── tsconfig.json          # TypeScript configuration
```

## Development Conventions

### Frontend (Vue/TypeScript)
- Component-based architecture using Vue 3 Composition API
- TypeScript for type safety
- Internationalization using vue-i18n with JSON translation files
- Responsive design with CSS variables for theme support
- Event-driven communication between components

### Backend (Rust)
- Tauri commands for exposing Rust functionality to frontend
- Cross-platform network monitoring using `netstat2` crate
- Process information retrieval using `sysinfo` crate
- Platform-specific implementations for process icons
- Error handling with Result types

### Internationalization
- Translation keys defined in `src/locales/zh-CN.json` and `src/locales/en-US.json`
- Language preference stored in localStorage
- Automatic system language detection

## Key Components

### Frontend Components
- `App.vue`: Main application with connection management and UI controls
- `ConnectionsTable`: Displays network connections in a sortable table
- `MenuBar`: Top menu with filtering, search, and theme controls
- `StatusBar`: Shows connection statistics and refresh info
- `ContextMenu`: Right-click menu for process actions
- `ProcessDetailsModal`: Modal showing detailed process information
- `AboutDialog`: About application dialog

### Backend Commands
- `get_connections`: Retrieves all active network connections
- `get_process_details`: Gets detailed information for a specific process
- `kill_process`: Terminates a process by PID
- `open_folder`: Opens the directory containing a process executable

## Platform Support

The application is designed to work across platforms:
- **Windows**: Full support with process icon extraction
- **Linux**: Network monitoring and process management
- **macOS**: Network monitoring and process management

Platform-specific features are handled through conditional compilation in Rust code.

## Security Considerations

- Tauri provides secure communication between frontend and backend
- Access to system-level information is restricted to backend Rust code
- Process management capabilities are limited to the current user's permissions
- CSP (Content Security Policy) is disabled in this configuration for development flexibility

## Testing

While no explicit test files were found in the initial exploration, typical Tauri + Vue applications would include:
- Unit tests for Vue components
- Integration tests for Tauri commands
- End-to-end tests for the complete application flow

## Deployment

The Tauri build process creates native executables for each platform:
- Windows: `.exe` file
- macOS: `.app` bundle
- Linux: AppImage or deb/rpm packages depending on configuration

The application bundles both frontend assets and the Rust backend into a single executable.

---

## Acknowledgments

Special thanks to [Qwen Code](https://github.com/QwenLM/qwen-code) for helping to create and maintain this project documentation.

---

[中文 README](./README_zh.md)