#![windows_subsystem = "windows"]

mod config;
mod manager;
mod qemu;
mod ui;

use config::VmConfig;
use manager::VmManager;
use std::path::PathBuf;
use std::sync::Arc;

fn resolve_aerovm_dir() -> PathBuf {
    // 1. Kiểm tra biến môi trường
    if let Ok(val) = std::env::var("AEROVM_HOME") {
        let p = PathBuf::from(val);
        if p.exists() {
            return p;
        }
    }

    // 2. Kiểm tra cạnh file thực thi (nếu chạy trong thư mục cài đặt riêng C:\Users\...\AeroVM)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            if exe_dir.join("qemu").exists() || exe_dir.join("vms").exists() {
                return exe_dir.to_path_buf();
            }
            if let Some(parent) = exe_dir.parent() {
                if parent.join("qemu").exists() || parent.join("vms").exists() {
                    return parent.to_path_buf();
                }
            }
        }
    }

    // 3. Mặc định tại ~/.aerovm
    let home = std::env::var("USERPROFILE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("C:\\Users\\Default"));

    home.join(".aerovm")
}

fn main() {
    let aerovm_dir = resolve_aerovm_dir();
    let vms_dir = aerovm_dir.join("vms");
    let qemu_dir = aerovm_dir.join("qemu");

    let _ = std::fs::create_dir_all(&vms_dir);
    let _ = std::fs::create_dir_all(&qemu_dir);
    let _ = std::fs::create_dir_all(aerovm_dir.join("isos"));

    let manager = Arc::new(VmManager::new(vms_dir.clone(), qemu_dir.clone()));

    // Tự động kiểm tra và khởi tạo máy ảo mẫu SliTaz GNU/Linux Live nếu có ISO
    let slitaz_iso = aerovm_dir.join("isos").join("slitaz-core64.iso");
    let has_slitaz = manager.list_vms().iter().any(|v| v.name.to_lowercase().contains("slitaz"));
    if !has_slitaz && slitaz_iso.exists() {
        let template = VmConfig::new_template(
            "SliTaz GNU/Linux (Live)",
            "slitaz",
            Some(slitaz_iso.to_string_lossy().to_string()),
        );
        let _ = manager.create_vm(template);
    } else if manager.list_vms().is_empty() {
        let template = VmConfig::new_template("Ubuntu 24.04 LTS", "linux", None);
        let _ = manager.create_vm(template);
    }

    // Chạy ứng dụng giao diện Native Desktop 100% bằng Rust GUI
    // Không dùng Microsoft Edge, Google Chrome hay web server localhost
    if let Err(e) = ui::run_native_app(manager) {
        eprintln!("Lỗi khởi chạy giao diện Native: {:?}", e);
    }
}
