# KB Android Emulator Manager

Modern standalone desktop app built with **React** and **Tauri v2** to manage, configure and optimise Android Virtual Devices (AVDs).

It includes sophisticated auto-tuning, system resource clamping, graphics pipeline selection and recovery tools to deliver a high-performance, developer-friendly emulator management experience.

---

## Core Features

*   **GPU Accelerator Configuration:** Native dropdown controls to toggle between Gfxstream (`auto`), Hardware Passthrough (`host`), and CPU fallback (`software`) rendering pipelines.
*   **Smart OS Auto-Tuning:** Dynamic configuration locks for specialized system images:
    *   **Wear OS Watches:** Locks resources to 1GB RAM / 1 Core, sets a 128MB Dalvik heap, and strips conflicting resolution properties to guarantee circular watch face layouts boot stably.
    *   **Android TV:** Locks resources to 2GB RAM / 2 Cores and tunes Dalvik heap size.
    *   **Android Automotive:** Locks resources to 4GB RAM / 4 Cores for optimal dashboard UI emulation.
*   **Wipe & Boot Recovery:** A one-click diagnostic tool that deletes corrupted Quick Boot snapshots from disk and launches the emulator with `-wipe-data -no-snapshot-load` to reset user space storage, instantly resolving boot loops (works for Phone, Wear OS, TV and Automotive images).
*   **Manual SDK Path Configuration:** Auto-detects existing Android SDKs (`ANDROID_SDK_ROOT` / `ANDROID_HOME`, Android Studio defaults) with a Settings fallback to browse and pin a non-standard SDK folder, so the app never breaks on custom installs.
*   **Ahead-Of-Time (AOT) compiler optimizer:** Compile all user-installed guest applications directly to native machine code via ADB (`cmd package compile -m speed`) for up to 40% performance gains.
*   **Network Acceleration:** Bypasses slirp DNS latencies by routing emulator traffic through Cloudflare's public resolver (`1.1.1.1`) and tunes QEMU TCP write window buffers on launch.

---

## Getting Started

### Prerequisites

Ensure you have the following installed on your machine:
*   [Node.js](https://nodejs.org/) (LTS version recommended)
*   [Rust Toolchain](https://www.rust-lang.org/tools/install) (via rustup)
*   [Android SDK](https://developer.android.com/studio) (with command-line tools and system-images configured)
*   Windows Hypervisor Platform (WHPX) or HAXM active in Windows Features.

### Installation

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/kil0bit-kb/kb-android-emulator-manager.git
    cd kb-android-emulator-manager
    ```

2.  **Install dependencies:**
    ```bash
    npm install
    ```

3.  **Configure environment:**
    The application automatically detects your local Android SDK (`ANDROID_SDK_ROOT` / `ANDROID_HOME`, Android Studio defaults, or the local `android-sdk/` folder). If your SDK lives in a non-standard directory, open **Settings → Android SDK Location** and browse to it as a manual fallback.

---

## Running & Building

### Run Development server
Launches the hot-reloading Vite dev server and spawns the Tauri native Windows desktop container:
```bash
npm run tauri dev
```

### Build Production installer
Compiles frontend assets, builds the Rust launcher backend, and generates a standalone Windows `.msi` or `.exe` installer bundle in `src-tauri/target/release/bundle/`:
```bash
npm run tauri build
```

---

## Project Structure

```
├── src/                  # React Frontend UI
│   ├── components/       # UI Components (Devices, GPU cards, Logs, etc.)
│   ├── App.jsx           # Main Shell Layout
│   └── index.css         # Custom Premium CSS Design System
├── src-tauri/            # Rust Backend Launcher
│   ├── src/
│   │   ├── commands.rs   # Core commands (AVD configuration, ADB, compiler opts)
│   │   └── main.rs       # Entrypoint & Tauri system shell configuration
│   └── Cargo.toml        # Rust package manifest
└── package.json          # Node dependencies and scripts
```
