// frontend/src-tauri/src/security/device.rs
use sha2::{Digest, Sha256};
use sysinfo::System;

/// Генерирует стабильный аппаратный отпечаток (Device ID) текущего компьютера.
/// Собирает данные о процессоре, имени хоста, версии ОС и количестве ядер.
pub fn get_hardware_fingerprint() -> String {
    let mut sys = System::new();
    sys.refresh_cpu_all();

    let mut hasher = Sha256::new();

    // 1. Имя хоста и ОС
    let host_name = System::host_name().unwrap_or_else(|| "unknown-host".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "unknown-os".to_string());
    let kernel_version = System::kernel_version().unwrap_or_else(|| "unknown-kernel".to_string());

    hasher.update(host_name.as_bytes());
    hasher.update(os_version.as_bytes());
    hasher.update(kernel_version.as_bytes());

    // 2. Информация о процессоре (Бренд, вендор, количество ядер)
    let cpus = sys.cpus();
    if !cpus.is_empty() {
        let cpu = &cpus[0];
        hasher.update(cpu.brand().as_bytes());
        hasher.update(cpu.vendor_id().as_bytes());
        hasher.update(&(cpus.len() as u32).to_le_bytes());
    }

    // 3. Общий объем оперативной памяти
    let total_memory = sys.total_memory();
    hasher.update(&total_memory.to_le_bytes());

    let result = hasher.finalize();
    format!("hw_{:x}", result)
}
