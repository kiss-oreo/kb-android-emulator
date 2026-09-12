# 📖 User & Configuration Guide

This guide describes how to customize, configure, and optimize Android Virtual Devices (AVDs) using the **KB Android Emulator Manager**.

---

## 🎮 GPU Acceleration Modes

Choosing the correct graphics pipeline is the single most important factor for gaming framerates:

1.  **Auto Rendering (`auto`):**
    *   *Behavior:* Passes rendering decisions directly to the emulator engine.
    *   *Usage:* Recommended for standard operations. On modern Windows with dedicated Nvidia GPUs, this defaults to native Vulkan translation (`gfxstream`).
2.  **Host Hardware Rendering (`host`):**
    *   *Behavior:* Forces the guest OS to route GLES and Vulkan calls directly to your host GPU (dGPU).
    *   *Usage:* Best for 3D games (like *Free Fire MAX* or *Zombie Shooter*).
3.  **Software rendering (`software`):**
    *   *Behavior:* Uses SwiftShader on your CPU to translate graphics.
    *   *Usage:* Extremely slow. Use only as a fallback if your graphics driver crashes.

---

## ⌚ Smart Tuning Profiles

The manager automatically adjusts hardware layouts and VM Dalvik heaps based on target image classification:

| Device Type | RAM Clamp | CPU Cores Clamp | VM Heap | Growth Limit | LCD Layout Override |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Wear OS Watch** | Max 1024 MB | Max 2 Cores | `128MB` | `64MB` | **Bypassed** (Keeps original circular faces) |
| **Android TV** | Max 2048 MB | Max 2 Cores | `256MB` | `128MB` | Allowed (`1920x1080` default) |
| **Android Car** | Max 4096 MB | Max 4 Cores | `512MB` | `256MB` | Allowed (`1280x720` default) |
| **Phone/Tablet** | User Defined | User Defined | `512/1024MB`| `256/512MB`| Allowed (`1280x720` default) |

---

## 🧹 Recovery Actions (Wipe & Boot)

If a device hangs at the Google/Android boot animation loop:
1.  **Cause:** The virtual device crashed or was edited during a quick-boot session, writing a corrupted memory snapshot to disk.
2.  **Fix:** Click the **`🧹 Wipe & Boot`** button on your device card. This deletes the AVD's `snapshots/` folder (plus stale snapshot locks) and launches QEMU with `-wipe-data -no-snapshot-load`, discarding saved state files and cold-booting fresh using your revised `config.ini` parameters. Recovery works for Phone, Wear OS, Android TV and Automotive images, and always forces a cold boot even when Quick Boot is enabled.

---

## 📁 Manual SDK Location (Fallback)

The app resolves the SDK in this order: manual override → app-managed `android-sdk/sdk` (if installed) → `ANDROID_SDK_ROOT` / `ANDROID_HOME` → standard OS locations → managed path.

If you installed the SDK somewhere unusual (e.g. `D:\Dev\Android\Sdk`):
1.  Open **Settings → Android SDK Location**.
2.  Click **Browse…** and pick the folder that directly contains `emulator/`, `platform-tools/` or `cmdline-tools/`.
3.  Click **Validate** (optional) then **Use This SDK**.
4.  The Setup page and Console Logs will now report the manual path. Use **Reset to Auto** at any time to return to auto-detect.

---

## 🚀 Performance Optimizations

### 1. Ahead-Of-Time (AOT) Compilation
By default, Android compiles app code JIT (Just-In-Time) when running, causing micro-stutters during gameplay.
*   To resolve this, click **`⚡ Compile Apps (AOT)`** on your device card.
*   This triggers ADB to run:
    ```bash
    cmd package compile -m speed -f <package_name>
    ```
*   This pre-compiles app bytecodes to native x86_64 machine code, eliminating compile-time stuttering.

### 2. Network Optimizations
*   The launcher overrides guest DNS routing to Cloudflare (`1.1.1.1`) via `-dns-server 1.1.1.1` to bypass local ISP DNS lookups.
*   TCP window sizes are expanded on boot via guest system properties to boost network download speeds inside Google Play Store.
