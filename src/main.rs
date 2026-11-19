#![windows_subsystem = "windows"]

mod config;
mod manager;
mod qemu;
mod ui;

use config::VmConfig;
use manager::VmManager;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

fn open_browser_window(url: &str, aerovm_dir: &Path) {
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

    let profile_dir = aerovm_dir.join("app_profile");
    let _ = std::fs::create_dir_all(&profile_dir);
    let profile_arg = format!("--user-data-dir={}", profile_dir.to_string_lossy());

    for path in &candidates {
        if !path.is_empty() && Path::new(path).exists() {
            let mut cmd = Command::new(path);
            cmd.arg(format!("--app={}", url))
               .arg(&profile_arg)
               .arg("--window-size=1260,860")
               .arg("--no-first-run")
               .arg("--no-default-browser-check")
               .arg("--disable-extensions")
               .arg("--disable-features=Translate,OptimizationHints,MediaRouter");

            #[cfg(windows)]
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

            if cmd.spawn().is_ok() {
                return;
            }
        }
    }

    // Fallback 1: rundll32
    let mut cmd1 = Command::new("rundll32.exe");
    cmd1.args(["url.dll,FileProtocolHandler", url]);
    #[cfg(windows)]
    cmd1.creation_flags(0x08000000);
    if cmd1.spawn().is_ok() {
        return;
    }

    // Fallback 2: cmd start
    let mut cmd2 = Command::new("cmd.exe");
    cmd2.args(["/c", "start", "", url]);
    #[cfg(windows)]
    cmd2.creation_flags(0x08000000);
    let _ = cmd2.spawn();
}

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

    let default_port = 45678;
    let base_dir = aerovm_dir.clone();

    ui::run_server(manager, default_port, move |port| {
        let url = format!("http://127.0.0.1:{}", port);
        open_browser_window(&url, &base_dir);
    });
}
