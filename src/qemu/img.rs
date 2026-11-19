use std::path::Path;
use std::process::Command;

pub struct QemuImg {
    exe_path: std::path::PathBuf,
}

impl QemuImg {
    pub fn new(exe_path: std::path::PathBuf) -> Self {
        Self { exe_path }
    }

    pub fn create_qcow2(&self, target_path: &Path, size_gb: u32) -> Result<(), String> {
        if let Some(parent) = target_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let output = Command::new(&self.exe_path)
            .arg("create")
            .arg("-f")
            .arg("qcow2")
            .arg(target_path)
            .arg(format!("{}G", size_gb))
            .output()
            .map_err(|e| format!("Không thể chạy qemu-img: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Lỗi tạo đĩa qcow2: {}", stderr))
        }
    }
}
