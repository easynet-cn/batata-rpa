# Batata RPA

<p align="center">
  <img src="public/tauri.svg" width="120" alt="Batata RPA Logo">
</p>

<p align="center">
  A cross-platform desktop RPA (Robotic Process Automation) application built with Tauri 2, Vue 3, and Rust.
</p>

<p align="center">
  <a href="./README_zh.md">中文文档</a>
</p>

## Features

### Workflow Designer
- **Visual Flow Editor** - Drag-and-drop node-based workflow design using Vue Flow
- **Rich Node Library** - Pre-built nodes for common automation tasks
- **Real-time Preview** - See your workflow structure as you build

### Node Types
| Category | Nodes |
|----------|-------|
| **Flow Control** | Start, End, Condition, Loop, ForEach, Try-Catch |
| **Web Automation** | Navigate, Click, Input, Wait, Screenshot |
| **Desktop Automation** | Click, Input, Get Text (platform-specific) |
| **Data Processing** | Read Excel, Write Excel, Set Variable |
| **Utilities** | Delay, Log |

### Execution Engine
- **Breakpoint Debugging** - Set breakpoints and step through workflows
- **Variable Inspection** - Monitor variable values during execution
- **Execution Logs** - Detailed logging with filtering capabilities
- **Error Handling** - Try-Catch nodes for robust error management

### Platform Support
- **Windows** - UI Automation API for native application control
- **macOS** - Accessibility API integration
- **Linux** - AT-SPI and X11 support

### Browser Automation
- **Chrome DevTools Protocol** - Direct browser control via CDP
- **Element Selection** - Visual element picker for web pages
- **Screenshots** - Capture page or element screenshots

### Data Operations
- **Excel Support** - Read and write Excel files (.xlsx)
- **Variable System** - Dynamic variables with expression interpolation

## Tech Stack

### Frontend
- **Vue 3** - Composition API with TypeScript
- **Tailwind CSS 4** - Utility-first styling
- **Vue Flow** - Flow-based workflow canvas
- **Pinia** - State management
- **Lucide Icons** - Beautiful icon library
- **Vue Router** - Client-side routing

### Backend
- **Tauri 2** - Lightweight desktop app framework
- **Rust** - Systems programming language
- **Tokio** - Async runtime
- **SeaORM** - Database ORM (SQLite/MySQL/PostgreSQL)
- **chromiumoxide** - Chrome DevTools Protocol client
- **mlua** - Lua scripting for plugins

## Requirements

- **Node.js** >= 18
- **pnpm** >= 8
- **Rust** >= 1.70
- **Platform SDK**:
  - Windows: Windows SDK
  - macOS: Xcode Command Line Tools
  - Linux: `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`

## Installation

```bash
# Clone the repository
git clone https://github.com/easynet-cn/batata-rpa.git
cd batata-rpa

# Install dependencies
pnpm install
```

## Development

```bash
# Start development server (frontend only)
pnpm dev

# Start full Tauri app in development mode
pnpm tauri dev

# Type check
pnpm build
```

## Building

```bash
# Build production app
pnpm tauri build
```

The built application will be in `src-tauri/target/release/bundle/`.

## Project Structure

```
batata-rpa/
├── src/                          # Vue 3 frontend
│   ├── views/                    # Page components
│   │   ├── Designer.vue          # Workflow designer
│   │   ├── ElementLibrary.vue    # UI element management
│   │   ├── Runner.vue            # Execution monitor
│   │   └── Settings.vue          # App settings
│   ├── components/
│   │   ├── designer/             # Designer components
│   │   │   ├── NodePalette.vue   # Node toolbox
│   │   │   ├── PropertyPanel.vue # Property editor
│   │   │   ├── DebugPanel.vue    # Debug controls
│   │   │   └── nodes/            # Custom node components
│   │   └── layout/               # Layout components
│   ├── stores/                   # Pinia stores
│   ├── types/                    # TypeScript types
│   └── router/                   # Vue Router config
├── src-tauri/                    # Rust backend
│   └── src/
│       ├── commands/             # Tauri IPC commands
│       ├── automation/           # Platform automation
│       │   ├── desktop/          # Desktop UI automation
│       │   ├── web/              # Browser automation
│       │   └── file/             # File operations
│       ├── engine/               # Execution engine
│       └── element/              # Element definitions
├── plugins/                      # Lua plugins
└── public/                       # Static assets
```

## Configuration

### Tauri Config
Edit `src-tauri/tauri.conf.json` for app configuration:
- Window size and title
- App identifier
- Build targets
- Permissions

### Environment
Create a `.env` file for environment-specific settings.

## Plugins

Batata RPA supports Lua plugins for extending functionality. Place plugins in the `plugins/` directory.

```lua
-- plugins/example_plugin.lua
function on_node_execute(node)
    print("Executing node: " .. node.id)
end
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Tauri](https://tauri.app/) - Desktop app framework
- [Vue Flow](https://vueflow.dev/) - Flow-based editor
- [Tailwind CSS](https://tailwindcss.com/) - CSS framework
- [Lucide](https://lucide.dev/) - Icon library
