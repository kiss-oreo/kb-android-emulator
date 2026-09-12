use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ─── Managed base path helpers ────────────────────────────────────────────────
/// Managed base folder (`android-sdk/`) next to the executable (production)
/// or at the project root (development, detected via `src-tauri/` marker).
pub fn sdk_base() -> PathBuf {
    let exe = std::env::current_exe().unwrap_or_default();
    let dir = exe.parent().unwrap_or(Path::new("."));

    // Traverse upwards to check if we are in development mode (project contains src-tauri folder)
    let mut current = dir.to_path_buf();
    for _ in 0..10 {
        if current.join("src-tauri").exists() {
            return current.join("android-sdk");
        }
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }

    // Production fallback: next to the executable
    dir.join("android-sdk")
}

/// The app-managed SDK location (`<sdk_base>/sdk`).
/// This is where the Setup flow installs packages when no custom path is set.
pub fn managed_sdk_dir() -> PathBuf {
    sdk_base().join("sdk")
}

pub fn jdk_dir() -> PathBuf {
    sdk_base().join("jdk")
}

/// Resolved SDK directory.
///
/// Resolution order:
///   1. Manual override from Settings (persisted config file) — `custom`
///   2. Managed SDK if it already has components installed — `managed`
///   3. `ANDROID_SDK_ROOT` / `ANDROID_HOME` env vars (if they look like an SDK) — `env:...`
///   4. Standard OS install locations (if they look like an SDK) — `detected`
///   5. Managed SDK path (may not exist yet — Setup will create it) — `managed`
pub fn sdk_dir() -> PathBuf {
    resolve_sdk_dir().0
}

/// Same as [`sdk_dir`] but also returns a human-readable source label.
pub fn resolve_sdk_dir() -> (PathBuf, String) {
    // 1 ── Manual override (Settings → SDK Path) ──────────────────────────────
    if let Some(custom) = read_custom_sdk_path() {
        return (custom, "custom".to_string());
    }

    let managed = managed_sdk_dir();

    // 2 ── Managed SDK wins if it already has real components ─────────────────
    // (preserves backwards compatibility for existing installs)
    if dir_looks_like_sdk(&managed) {
        return (managed, "managed".to_string());
    }

    // 3 ── Environment variables ──────────────────────────────────────────────
    for var in ["ANDROID_SDK_ROOT", "ANDROID_HOME"] {
        if let Ok(val) = std::env::var(var) {
            let trimmed = val.trim();
            if trimmed.is_empty() {
                continue;
            }
            let candidate = PathBuf::from(trimmed);
            if dir_looks_like_sdk(&candidate) {
                return (candidate, format!("env:{}", var));
            }
        }
    }

    // 4 ── Standard OS install locations ──────────────────────────────────────
    for candidate in standard_sdk_locations() {
        if dir_looks_like_sdk(&candidate) {
            return (candidate, "detected".to_string());
        }
    }

    // 5 ── Fallback to managed path (Setup flow installs here) ────────────────
    (managed, "managed".to_string())
}

pub fn cmdline_dir() -> PathBuf {
    sdk_dir().join("cmdline-tools").join("latest")
}
pub fn emulator_dir() -> PathBuf {
    sdk_dir().join("emulator")
}
pub fn avd_dir() -> PathBuf {
    sdk_base().join("avd")
}

pub fn ensure_dirs(base: &PathBuf) {
    let _ = base; // we use sdk_base() internally
    for d in [sdk_base(), jdk_dir(), managed_sdk_dir(), avd_dir()] {
        let _ = std::fs::create_dir_all(&d);
    }
}

// ─── SDK detection helpers ────────────────────────────────────────────────────
/// Returns true if `dir` exists and contains at least one recognisable SDK component.
pub fn dir_looks_like_sdk(dir: &Path) -> bool {
    if !dir.is_dir() {
        return false;
    }
    has_emulator(dir) || has_platform_tools(dir) || has_cmdline_tools(dir) || has_system_images(dir)
}

pub fn has_emulator(sdk: &Path) -> bool {
    #[cfg(windows)]
    let exe = sdk.join("emulator").join("emulator.exe");
    #[cfg(not(windows))]
    let exe = sdk.join("emulator").join("emulator");
    exe.exists() || sdk.join("emulator").join("qemu").exists()
}

pub fn has_platform_tools(sdk: &Path) -> bool {
    #[cfg(windows)]
    let adb = sdk.join("platform-tools").join("adb.exe");
    #[cfg(not(windows))]
    let adb = sdk.join("platform-tools").join("adb");
    adb.exists()
}

pub fn has_cmdline_tools(sdk: &Path) -> bool {
    #[cfg(windows)]
    let manager = sdk
        .join("cmdline-tools")
        .join("latest")
        .join("bin")
        .join("sdkmanager.bat");
    #[cfg(not(windows))]
    let manager = sdk
        .join("cmdline-tools")
        .join("latest")
        .join("bin")
        .join("sdkmanager");
    manager.exists() || sdk.join("cmdline-tools").is_dir()
}

pub fn has_system_images(sdk: &Path) -> bool {
    let base = sdk.join("system-images");
    if !base.is_dir() {
        return false;
    }
    // Non-empty directory counts as having system images.
    std::fs::read_dir(&base)
        .map(|mut d| d.next().is_some())
        .unwrap_or(false)
}

/// Standard per-OS SDK install locations (Android Studio defaults + common alts).
pub fn standard_sdk_locations() -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = vec![];

    if let Some(home) = dirs::home_dir() {
        #[cfg(windows)]
        {
            // Android Studio default on Windows: %LOCALAPPDATA%\Android\Sdk
            if let Some(local_app_data) = dirs::data_local_dir() {
                out.push(local_app_data.join("Android").join("Sdk"));
            }
            out.push(home.join("Android").join("Sdk"));
            out.push(home.join("AppData").join("Local").join("Android").join("Sdk"));
        }
        #[cfg(target_os = "macos")]
        {
            out.push(home.join("Library").join("Android").join("sdk"));
            out.push(home.join("Android").join("sdk"));
        }
        #[cfg(target_os = "linux")]
        {
            out.push(home.join("Android").join("Sdk"));
            out.push(home.join("Android").join("sdk"));
        }
    }

    #[cfg(target_os = "linux")]
    {
        out.push(PathBuf::from("/opt/android-sdk"));
        out.push(PathBuf::from("/usr/lib/android-sdk"));
        out.push(PathBuf::from("/opt/android_sdk"));
    }
    #[cfg(target_os = "macos")]
    {
        out.push(PathBuf::from("/opt/android-sdk"));
        out.push(PathBuf::from("/usr/local/share/android-sdk"));
    }
    #[cfg(windows)]
    {
        out.push(PathBuf::from("C:\\Android\\Sdk"));
        out.push(PathBuf::from("D:\\Android\\Sdk"));
    }

    // De-duplicate while preserving order.
    let mut seen = std::collections::HashSet::new();
    out.into_iter()
        .filter(|p| seen.insert(p.clone()))
        .collect()
}

/// Candidates that actually exist on disk and look like an SDK.
/// Used by the Settings UI so users can one-click adopt a detected SDK.
pub fn detect_sdk_candidates() -> Vec<PathBuf> {
    let mut candidates = vec![];

    for var in ["ANDROID_SDK_ROOT", "ANDROID_HOME"] {
        if let Ok(val) = std::env::var(var) {
            let trimmed = val.trim();
            if !trimmed.is_empty() {
                let p = PathBuf::from(trimmed);
                if dir_looks_like_sdk(&p) && !candidates.contains(&p) {
                    candidates.push(p);
                }
            }
        }
    }

    for p in standard_sdk_locations() {
        if dir_looks_like_sdk(&p) && !candidates.contains(&p) {
            candidates.push(p);
        }
    }

    let managed = managed_sdk_dir();
    if dir_looks_like_sdk(&managed) && !candidates.contains(&managed) {
        candidates.push(managed);
    }

    candidates
}

// ─── Manual override persistence ──────────────────────────────────────────────
/// Writable config file for the manual SDK override.
/// Stored outside the install dir so it survives updates and works without
/// admin rights (e.g. `%APPDATA%\KBAndroidEmulator\sdk-path.txt` on Windows,
/// `~/.config/KBAndroidEmulator/sdk-path.txt` on Linux).
pub fn custom_sdk_override_file() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("KBAndroidEmulator").join("sdk-path.txt")
}

/// Legacy override locations (checked for backwards compatibility).
fn legacy_override_files() -> Vec<PathBuf> {
    let mut out = vec![];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            out.push(dir.join("sdk-path-override.txt"));
            out.push(dir.join("android-sdk").join("sdk-path-override.txt"));
        }
    }
    // Dev-mode project root copy.
    out.push(sdk_base().join("sdk-path-override.txt"));
    out
}

/// Read the persisted manual SDK override, if any.
/// Also honours the `KB_SDK_PATH` env var (useful for tests / portable use).
pub fn read_custom_sdk_path() -> Option<PathBuf> {
    // Env var takes precedence (explicit, session-scoped).
    if let Ok(val) = std::env::var("KB_SDK_PATH") {
        let trimmed = val.trim();
        if !trimmed.is_empty() {
            return Some(PathBuf::from(trimmed));
        }
    }

    let file = custom_sdk_override_file();
    if let Ok(content) = std::fs::read_to_string(&file) {
        let trimmed = content.trim();
        if !trimmed.is_empty() {
            return Some(PathBuf::from(trimmed));
        }
    }

    for legacy in legacy_override_files() {
        if let Ok(content) = std::fs::read_to_string(&legacy) {
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                return Some(PathBuf::from(trimmed));
            }
        }
    }

    None
}

pub fn write_custom_sdk_path(path: &Path) -> anyhow::Result<()> {
    let file = custom_sdk_override_file();
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&file, path.to_string_lossy().to_string())?;
    Ok(())
}

pub fn clear_custom_sdk_path_file() -> anyhow::Result<()> {
    let file = custom_sdk_override_file();
    if file.exists() {
        std::fs::remove_file(&file)?;
    }
    for legacy in legacy_override_files() {
        if legacy.exists() {
            let _ = std::fs::remove_file(&legacy);
        }
    }
    Ok(())
}

pub fn get_java_exe() -> Option<PathBuf> {
    let jdk = jdk_dir();
    if !jdk.exists() {
        return None;
    }
    let entry = std::fs::read_dir(&jdk)
        .ok()?
        .filter_map(|e| e.ok())
        .find(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("jdk")
        })?;
    #[cfg(windows)]
    let java = entry.path().join("bin").join("java.exe");
    #[cfg(not(windows))]
    let java = entry.path().join("bin").join("java");
    if java.exists() {
        Some(java)
    } else {
        None
    }
}

pub fn get_java_home() -> Option<PathBuf> {
    let jdk = jdk_dir();
    let entry = std::fs::read_dir(&jdk)
        .ok()?
        .filter_map(|e| e.ok())
        .find(|e| e.file_name().to_string_lossy().starts_with("jdk"))?;
    Some(entry.path())
}

pub fn build_env() -> HashMap<String, String> {
    let mut env: HashMap<String, String> = std::env::vars().collect();
    if let Some(jh) = get_java_home() {
        env.insert("JAVA_HOME".to_string(), jh.to_string_lossy().to_string());
        let bin = jh.join("bin").to_string_lossy().to_string();
        let path = env.get("PATH").cloned().unwrap_or_default();
        #[cfg(windows)]
        env.insert("PATH".to_string(), format!("{};{}", bin, path));
        #[cfg(not(windows))]
        env.insert("PATH".to_string(), format!("{}:{}", bin, path));
    }
    env.insert("ANDROID_SDK_ROOT".to_string(), sdk_dir().to_string_lossy().to_string());
    env.insert("ANDROID_AVD_HOME".to_string(), avd_dir().to_string_lossy().to_string());
    env.insert("ANDROID_EMULATOR_HOME".to_string(), avd_dir().to_string_lossy().to_string());

    // Low-latency WASAPI Audio Optimizations (Windows only — harmless to set on other OS)
    env.insert("QEMU_AUDIO_TIMER_PERIOD".to_string(), "0".to_string());
    env.insert("QEMU_WASAPI_BUF_SIZE".to_string(), "512".to_string());

    env
}
