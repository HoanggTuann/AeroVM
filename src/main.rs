mod config;
mod manager;
mod qemu;
mod ui;

use config::VmConfig;
use manager::VmManager;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

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

    println!("[*] Thư mục máy ảo: {:?}", vms_dir);
    println!("[*] Thư mục QEMU: {:?}", qemu_dir);

    let manager = Arc::new(VmManager::new(vms_dir.clone(), qemu_dir.clone()));

    // Nếu chưa có máy ảo nào, tự động tạo sẵn một máy ảo mẫu Ubuntu
    if manager.list_vms().is_empty() {
        println!("[*] Khởi tạo máy ảo mẫu Ubuntu 24.04...");
        let template = VmConfig::new_template("Ubuntu 24.04 LTS", "linux", None);
        let _ = manager.create_vm(template);
    }

    let port = 45678;
    let url = format!("http://127.0.0.1:{}", port);

    // Mở giao diện UTM dạng App Window bằng Edge / Chrome
    let url_clone = url.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(600));
        println!("[*] Đang mở giao diện AeroVM UTM Window...");
        
        let launched = Command::new("msedge.exe")
            .arg(format!("--app={}", url_clone))
            .arg("--window-size=1200,780")
            .spawn()
            .is_ok();

        if !launched {
            // Fallback mở trình duyệt mặc định
            let _ = Command::new("cmd.exe")
                .arg("/c")
                .arg("start")
                .arg(&url_clone)
                .spawn();
        }
    });

    // Chạy máy chủ giao diện
    ui::run_server(manager, port);
}
