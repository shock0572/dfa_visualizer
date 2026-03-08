# DFA Visualizer

System tray application that tracks your [Data for Azeroth](https://www.dataforazeroth.com) ladder rankings.

Built with [Tauri v2](https://v2.tauri.app/) (Rust + React/TypeScript).

## Features

- Native system tray icon (Windows, Linux, macOS menu bar)
- Displays Completion Score, Achievement Points, Mounts, and all Account Wide rankings
- World, Region, and Realm rank for each category
- Modern dark-themed UI with Overview and Account Wide tabs
- Configurable refresh interval and tracked categories
- Settings stored locally

## Prerequisites

- [Node.js](https://nodejs.org/) >= 20
- [Rust](https://rustup.rs/) (stable)
- System dependencies for your platform:
  - **Linux**: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev libssl-dev patchelf`
  - **macOS / Windows**: no extra deps needed

## Setup

```bash
npm install
```

## Development

```bash
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

Produces:
- **Windows**: `.exe` and `.msi` installer
- **macOS**: `.app` bundle and `.dmg`
- **Linux**: `.deb`, `.AppImage`

## Usage

On first launch, right-click the tray icon and select **Settings** to configure your profile (Region, Realm, Character name). Pick which ranking categories to track, then hit Save.

Click the tray icon or select **Show Details** to open the dashboard window.

## Configuration

Settings are stored at:
- **Linux**: `~/.config/dfa_visualizer/settings.json`
- **macOS**: `~/Library/Application Support/dfa_visualizer/settings.json`
- **Windows**: `%APPDATA%/dfa_visualizer/settings.json`
