use sha2::{Digest, Sha256};
use sysinfo::System;

#[cfg(target_os = "windows")]
fn get_platform_machine_id() -> Option<String> {
    use std::os::windows::process::CommandExt;
    let output = std::process::Command::new("reg")
        .args(["query", r"HKLM\SOFTWARE\Microsoft\Cryptography", "/v", "MachineGuid"])
        .creation_flags(0x08000000)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains("MachineGuid") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                return Some(parts[parts.len() - 1].trim().to_string());
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn get_platform_machine_id() -> Option<String> {
    if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
        let trimmed = id.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if let Ok(id) = std::fs::read_to_string("/var/lib/dbus/machine-id") {
        let trimmed = id.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn get_platform_machine_id() -> Option<String> {
    let output = std::process::Command::new("ioreg")
        .args(["-rd1", "-c", "IOPlatformExpertDevice"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains("IOPlatformUUID") {
            let parts: Vec<&str> = line.split('"').collect();
            if parts.len() >= 4 {
                return Some(parts[3].trim().to_string());
            }
        }
    }
    None
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn get_platform_machine_id() -> Option<String> {
    None
}

pub fn get_hardware_fingerprint() -> String {
    let mut hasher = Sha256::new();
    let mut has_hardware_id = false;

    if let Some(machine_id) = get_platform_machine_id() {
        hasher.update(machine_id.as_bytes());
        has_hardware_id = true;
    }

    let mut sys = System::new();
    sys.refresh_cpu_all();

    let cpus = sys.cpus();
    if !cpus.is_empty() {
        let cpu = &cpus[0];
        hasher.update(cpu.brand().as_bytes());
        hasher.update(cpu.vendor_id().as_bytes());
    }

    if !has_hardware_id {
        let total_memory = sys.total_memory();
        hasher.update(&total_memory.to_le_bytes());
        if !cpus.is_empty() {
            hasher.update(&(cpus.len() as u32).to_le_bytes());
        }
    }

    let result = hasher.finalize();
    format!("hw_{:x}", result)
}
