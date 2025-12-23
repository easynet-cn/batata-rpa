# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Batata RPA is a cross-platform desktop RPA (Robotic Process Automation) application built with Tauri 2, Vue 3, and TypeScript. The frontend uses Vue 3 with Element Plus for UI, Vue Flow for workflow designer, and Pinia for state management. The backend is Rust with Tauri for native desktop capabilities.

## Development Commands

### Frontend (pnpm)
- `pnpm dev` - Start Vite dev server on port 1420
- `pnpm build` - Type-check with vue-tsc and build frontend

### Tauri (full app)
- `pnpm tauri dev` - Run full Tauri app in development mode
- `pnpm tauri build` - Build production desktop app bundle

### Rust Backend
- `cd src-tauri && cargo check` - Fast type checking for Rust code
- `cd src-tauri && cargo build` - Build Rust backend

## Architecture

```
src/                           # Vue 3 frontend
├── views/                     # Page components
│   ├── Designer.vue           # Workflow designer with Vue Flow
│   ├── ElementLibrary.vue     # UI element management
│   ├── Runner.vue             # Execution monitor
│   └── Settings.vue           # App settings
├── components/
│   ├── designer/              # Workflow designer components
│   │   ├── NodePalette.vue    # Draggable node panel
│   │   ├── PropertyPanel.vue  # Node property editor
│   │   └── nodes/             # Custom node components
│   └── layout/                # Layout components
├── stores/                    # Pinia state management
│   ├── workflow.ts            # Workflow state
│   ├── element.ts             # Element library state
│   └── execution.ts           # Execution state
├── types/                     # TypeScript type definitions
└── router/                    # Vue Router config

src-tauri/src/                 # Rust backend
├── lib.rs                     # Tauri entry, command registration
├── commands/                  # Tauri Commands (IPC handlers)
│   ├── workflow.rs            # Workflow CRUD
│   ├── element.rs             # Element capture/management
│   └── execution.rs           # Workflow execution control
├── automation/                # Platform automation
│   ├── desktop/               # Desktop UI automation
│   │   ├── windows.rs         # Windows UI Automation
│   │   ├── macos.rs           # macOS Accessibility
│   │   └── linux.rs           # Linux AT-SPI
│   ├── web/                   # Browser automation (CDP)
│   └── file/                  # File operations
├── engine/                    # Execution engine
│   ├── executor.rs            # Workflow executor
│   ├── runtime.rs             # Runtime state management
│   └── variable.rs            # Variable system
└── element/                   # Element definitions
```

## Key Patterns

### Rust-Vue Communication
Tauri commands in `src-tauri/src/commands/` are called from Vue via:
```typescript
import { invoke } from "@tauri-apps/api/core";
await invoke("execute_workflow", { workflow });
```

### Workflow Data Structure
Workflows are JSON-serializable with nodes, edges, and variables. Node types: `start`, `end`, `click`, `input`, `getText`, `delay`, `log`.

### Platform-Specific Code
Use conditional compilation for platform features:
```rust
#[cfg(target_os = "windows")]
use crate::automation::desktop::windows;
```

## Key Dependencies

### Frontend
- Vue Flow: Workflow canvas
- Element Plus: UI components
- Pinia: State management

### Backend
- tokio: Async runtime
- uiautomation (Windows): UI Automation API
- async-trait: Async trait support
