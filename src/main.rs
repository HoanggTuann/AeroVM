mod config;
mod manager;
mod qemu;
mod ui;

use config::VmConfig;
use manager::VmManager;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

fn open_browser_window(url: &str) {
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
    let edge_local = format!("{}\\Microsoft\\Edge\\Application\\msedge.exe", local_app_data);
    let chrome_local = format!("{}\\Google\\Chrome\\Application\\chrome.exe", local_app_data);

    let candidates = [
        "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
        "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe",
        edge_local.as_str(),
        "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
        "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe",
        chrome_local.as_str(),
        "C:\\Program Files\\BraveSoftware\\Brave-Browser\\Application\\brave.exe",
    ];

    for path in &candidates {
        if !path.is_empty() && Path::new(path).exists() {
            let res = Command::new(path)
                .arg(format!("--app={}", url))
                .arg("--window-size=1200,800")
                .spawn();
            if res.is_ok() {
                return;
            }
        }
    }

    // Fallback 1: rundll32
    if Command::new("rundll32.exe")
        .args(["url.dll,FileProtocolHandler", url])
        .spawn()
        .is_ok()
    {
        return;
    }

    // Fallback 2: cmd start
    let _ = Command::new("cmd.exe")
        .args(["/c", "start", "", url])
        .spawn();
}

fn main() {
    println!("============================================================");
    println!("     🚀 AeroVM - Trình Quản Lý Máy Ảo Phong Cách UTM       ");
    println!("     100% Rust Backend · Tăng Tốc WHPX Hardware Acceleration");
    println!("============================================================");

    let home = std::env::var("USERPROFILE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("C:\\Users\\Default"));

    let aerovm_dir = home.join(".aerovm");
    let vms_dir = aerovm_dir.join("vms");
    let qemu_dir = aerovm_dir.join("qemu");

    println!("[*] Thư mục lưu máy ảo: {:?}", vms_dir);
    println!("[*] Thư mục QEMU: {:?}", qemu_dir);

    let manager = Arc::new(VmManager::new(vms_dir.clone(), qemu_dir.clone()));

    // Tự động kiểm tra và khởi tạo máy ảo SliTaz GNU/Linux Live nếu có ISO
    let slitaz_iso = aerovm_dir.join("isos").join("slitaz-core64.iso");
    let has_slitaz = manager.list_vms().iter().any(|v| v.name.to_lowercase().contains("slitaz"));
    if !has_slitaz && slitaz_iso.exists() {
        println!("[*] Phát hiện SliTaz Live ISO! Đang khởi tạo máy ảo SliTaz GNU/Linux...");
        let template = VmConfig::new_template(
            "SliTaz GNU/Linux (Live)",
            "slitaz",
            Some(slitaz_iso.to_string_lossy().to_string()),
        );
        let _ = manager.create_vm(template);
    } else if manager.list_vms().is_empty() {
        println!("[*] Đang khởi tạo máy ảo mẫu Ubuntu 24.04 LTS...");
        let template = VmConfig::new_template("Ubuntu 24.04 LTS", "linux", None);
        let _ = manager.create_vm(template);
    }

    let default_port = 45678;

    ui::run_server(manager, default_port, |port| {
        let url = format!("http://127.0.0.1:{}", port);
        println!("[*] Đang mở giao diện AeroVM UTM Window: {}", url);
        open_browser_window(&url);
        println!("[*] Giao diện đã sẵn sàng! Bạn có thể sử dụng AeroVM trên cửa sổ vừa mở.");
        println!("[*] (Hãy giữ cửa sổ console này chạy để duy trì dịch vụ máy ảo)");
    });
}
