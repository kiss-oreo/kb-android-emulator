use std::path::PathBuf;
use tauri::{Emitter, Window};

use super::paths::{
    clear_custom_sdk_path_file, detect_sdk_candidates, dir_looks_like_sdk, has_cmdline_tools,
    has_emulator, has_platform_tools, has_system_images, managed_sdk_dir, read_custom_sdk_path,
    resolve_sdk_dir, write_custom_sdk_path,
};
use super::types::{CommandResult, SdkPathInfo, SdkValidation};

// ─── Validate a candidate SDK path (pure check, no persistence) ───────────────
pub fn validate_path(path_str: &str) -> SdkValidation {
    let trimmed = path_str.trim();
    let path = PathBuf::from(trimmed);

    let exists = path.exists();
    let is_dir = path.is_dir();
    let has_emu = has_emulator(&path);
    let has_pt = has_platform_tools(&path);
    let has_ct = has_cmdline_tools(&path);
    let has_si = has_system_images(&path);

    let looks_like_sdk = has_emu || has_pt || has_ct || has_si;

    let (valid, message) = if trimmed.is_empty() {
        (false, "Path is empty.".to_string())
    } else if !exists {
        (false, "Folder does not exist.".to_string())
    } else if !is_dir {
        (false, "Path is not a folder.".to_string())
    } else if looks_like_sdk {
        let mut parts = vec![];
        if has_emu {
            parts.push("emulator");
        }
        if has_pt {
            parts.push("platform-tools");
        }
        if has_ct {
            parts.push("cmdline-tools");
        }
        if has_si {
            parts.push("system-images");
        }
        (
            true,
            format!("Valid Android SDK (found: {}).", parts.join(", ")),
        )
    } else {
        (
            false,
            "Folder exists but contains no recognisable SDK components (expected emulator/, platform-tools/ or cmdline-tools/).".to_string(),
        )
    };

    SdkValidation {
        path: trimmed.to_string(),
        exists,
        is_dir,
        valid,
        has_emulator: has_emu,
        has_platform_tools: has_pt,
        has_cmdline_tools: has_ct,
        has_system_images: has_si,
        message,
    }
}

fn build_path_info() -> SdkPathInfo {
    let (current, source) = resolve_sdk_dir();
    let custom = read_custom_sdk_path().map(|p| p.to_string_lossy().to_string());
    let managed = managed_sdk_dir();
    let candidates: Vec<String> = detect_sdk_candidates()
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let exists = current.exists();
    let has_emu = has_emulator(&current);
    let has_pt = has_platform_tools(&current);
    let has_ct = has_cmdline_tools(&current);
    let has_si = has_system_images(&current);
    let valid = dir_looks_like_sdk(&current);

    let message = if source == "custom" {
        if valid {
            "Using your manually configured SDK path.".to_string()
        } else {
            "Manual SDK path is set but looks invalid — the app may fail to find emulator/adb. Pick a valid SDK folder or reset to Auto.".to_string()
        }
    } else if valid {
        match source.as_str() {
            "managed" => "Using the app-managed SDK (android-sdk/sdk).".to_string(),
            "detected" => "Auto-detected an existing Android SDK on this machine.".to_string(),
            s if s.starts_with("env:") => format!("Using SDK from environment variable {}.", s),
            _ => "SDK resolved.".to_string(),
        }
    } else {
        "No SDK components found yet — complete Setup or point the app at an existing SDK.".to_string()
    };

    SdkPathInfo {
        current_path: current.to_string_lossy().to_string(),
        source,
        custom_path: custom,
        managed_path: managed.to_string_lossy().to_string(),
        detected_candidates: candidates,
        exists,
        valid,
        has_emulator: has_emu,
        has_platform_tools: has_pt,
        has_cmdline_tools: has_ct,
        has_system_images: has_si,
        message,
    }
}

// ─── Tauri commands ───────────────────────────────────────────────────────────
#[tauri::command]
pub fn get_sdk_path_info() -> SdkPathInfo {
    build_path_info()
}

#[tauri::command]
pub fn validate_sdk_path(path: String) -> SdkValidation {
    validate_path(&path)
}

#[tauri::command]
pub fn set_custom_sdk_path(path: String, window: Window) -> CommandResult {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return CommandResult {
            ok: false,
            error: Some("SDK path is empty.".to_string()),
            output: None,
        };
    }

    let candidate = PathBuf::from(trimmed);

    // Canonicalize when possible so the stored path is absolute & normalized.
    let normalized: PathBuf = std::fs::canonicalize(&candidate).unwrap_or(candidate.clone());

    if !normalized.exists() {
        return CommandResult {
            ok: false,
            error: Some("Folder does not exist.".to_string()),
            output: None,
        };
    }
    if !normalized.is_dir() {
        return CommandResult {
            ok: false,
            error: Some("Path is not a folder.".to_string()),
            output: None,
        };
    }
    if !dir_looks_like_sdk(&normalized) {
        return CommandResult {
            ok: false,
            error: Some(
                "Not a valid Android SDK folder — expected emulator/, platform-tools/ or cmdline-tools/ inside."
                    .to_string(),
            ),
            output: None,
        };
    }

    if let Err(e) = write_custom_sdk_path(&normalized) {
        return CommandResult {
            ok: false,
            error: Some(format!("Failed to save SDK path: {}", e)),
            output: None,
        };
    }

    let _ = window.emit(
        "log",
        format!("Manual SDK path set: {}", normalized.display()),
    );
    CommandResult {
        ok: true,
        error: None,
        output: Some(normalized.to_string_lossy().to_string()),
    }
}

#[tauri::command]
pub fn clear_custom_sdk_path(window: Window) -> CommandResult {
    if let Err(e) = clear_custom_sdk_path_file() {
        return CommandResult {
            ok: false,
            error: Some(format!("Failed to clear SDK override: {}", e)),
            output: None,
        };
    }
    // Report what we fell back to.
    let (fallback, source) = resolve_sdk_dir();
    let _ = window.emit(
        "log",
        format!(
            "Manual SDK override cleared — now using {} ({})",
            fallback.display(),
            source
        ),
    );
    CommandResult {
        ok: true,
        error: None,
        output: Some(fallback.to_string_lossy().to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_path_is_invalid() {
        let v = validate_path("");
        assert!(!v.valid);
    }

    #[test]
    fn missing_path_reports_not_exist() {
        let v = validate_path("/definitely/not/a/real/sdk-path-12345");
        assert!(!v.valid);
        assert!(!v.exists);
    }
}
