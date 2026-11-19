use crate::config::VmConfig;
use crate::qemu::{QemuArgs, QemuImg};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmSummary {
    pub id: String,
    pub name: String,
    pub os_type: String,
    pub os_variant: String,
    pub cpu_cores: u32,
    pub memory_mb: u32,
    pub disk_size_gb: u32,
    pub running: bool,
    pub accel: String,
    pub machine_type: String,
    pub vga_card: String,
    pub low_spec_mode: bool,
    pub sound_enabled: bool,
    pub iso_path: Option<String>,
    pub folder_path: String,
    pub created_at: String,
}

pub struct VmManager {
    base_dir: PathBuf,
    qemu_dir: PathBuf,
    running: Arc<Mutex<HashMap<String, Child>>>,
}

impl VmManager {
    pub fn new(base_dir: PathBuf, qemu_dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(&base_dir);
        Self {
            base_dir,
            qemu_dir,
            running: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn qemu_exe(&self) -> PathBuf {
        let preferred = self.qemu_dir.join("qemu-system-x86_64.exe");
        if preferred.exists() {
            return preferred;
        }
        let sys_path = PathBuf::from("C:\\Program Files\\qemu\\qemu-system-x86_64.exe");
        if sys_path.exists() {
            return sys_path;
        }
        preferred
    }

    pub fn qemu_img_exe(&self) -> PathBuf {
        let preferred = self.qemu_dir.join("qemu-img.exe");
        if preferred.exists() {
            return preferred;
        }
        let sys_path = PathBuf::from("C:\\Program Files\\qemu\\qemu-img.exe");
        if sys_path.exists() {
            return sys_path;
        }
        preferred
    }

    pub fn list_vms(&self) -> Vec<VmSummary> {
        self.cleanup_dead_processes();
        let running_map = self.running.lock().unwrap();

        let mut list = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.base_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    let config_file = p.join("config.json");
                    if let Ok(content) = fs::read_to_string(&config_file) {
                        if let Ok(mut cfg) = serde_json::from_str::<VmConfig>(&content) {
                            // Tự động kiểm tra và sửa cấu hình lỗi từ phiên bản cũ
                            let mut need_save = false;
                            if (cfg.os_type == "slitaz" || cfg.name.to_lowercase().contains("slitaz")) && cfg.accel == "whpx" {
                                cfg.accel = "tcg".to_string();
                                need_save = true;
                            }
                            if cfg.display_type.is_empty() || cfg.display_type == "gtk" {
                                cfg.display_type = "sdl".to_string();
                                need_save = true;
                            }
                            if need_save {
                                if let Ok(json_str) = serde_json::to_string_pretty(&cfg) {
                                    let _ = fs::write(&config_file, json_str);
                                }
                            }

                            let is_run = running_map.contains_key(&cfg.id);
                            let disk_size = cfg.drives.iter().filter(|d| !d.is_cdrom).map(|d| d.size_gb).sum();
                            list.push(VmSummary {
                                id: cfg.id,
                                name: cfg.name,
                                os_type: cfg.os_type,
                                os_variant: cfg.os_variant,
                                cpu_cores: cfg.cpu_cores,
                                memory_mb: cfg.memory_mb,
                                disk_size_gb: disk_size,
                                running: is_run,
                                accel: cfg.accel,
                                machine_type: cfg.machine_type,
                                vga_card: cfg.vga_card,
                                low_spec_mode: cfg.low_spec_mode,
                                sound_enabled: cfg.sound_enabled,
                                iso_path: cfg.iso_path,
                                folder_path: p.to_string_lossy().to_string(),
                                created_at: cfg.created_at,
                            });
                        }
                    }
                }
            }
        }
        list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        list
    }

    pub fn get_vm(&self, id: &str) -> Option<VmConfig> {
        let config_file = self.base_dir.join(id).join("config.json");
        if let Ok(content) = fs::read_to_string(&config_file) {
            if let Ok(mut cfg) = serde_json::from_str::<VmConfig>(&content) {
                // Tự động bảo vệ và sửa các giá trị không hợp lệ
                let mut need_save = false;
                if (cfg.os_type == "slitaz" || cfg.name.to_lowercase().contains("slitaz")) && cfg.accel == "whpx" {
                    cfg.accel = "tcg".to_string();
                    need_save = true;
                }
                if cfg.display_type != "sdl" {
                    cfg.display_type = "sdl".to_string();
                    need_save = true;
                }
                if need_save {
                    if let Ok(json_str) = serde_json::to_string_pretty(&cfg) {
                        let _ = fs::write(&config_file, json_str);
                    }
                }
                return Some(cfg);
            }
        }
        None
    }

    pub fn create_vm(&self, cfg: VmConfig) -> Result<VmConfig, String> {
        let vm_dir = self.base_dir.join(&cfg.id);
        fs::create_dir_all(&vm_dir).map_err(|e| format!("Không thể tạo thư mục VM: {}", e))?;

        // Tạo đĩa chính nếu chưa có
        if let Some(drive) = cfg.drives.iter().find(|d| !d.is_cdrom) {
            let disk_path = if Path::new(&drive.path).is_absolute() {
                PathBuf::from(&drive.path)
            } else {
                vm_dir.join(&drive.path)
            };

            if !disk_path.exists() {
                let img_tool = QemuImg::new(self.qemu_img_exe());
                img_tool.create_qcow2(&disk_path, drive.size_gb)?;
            }
        }

        // Lưu config.json
        let config_json = serde_json::to_string_pretty(&cfg).map_err(|e| e.to_string())?;
        fs::write(vm_dir.join("config.json"), config_json)
            .map_err(|e| format!("Không thể lưu config.json: {}", e))?;

        Ok(cfg)
    }

    pub fn update_vm_config(&self, updated: VmConfig) -> Result<VmConfig, String> {
        self.cleanup_dead_processes();
        {
            let running_map = self.running.lock().unwrap();
            if running_map.contains_key(&updated.id) {
                return Err("Máy ảo đang chạy! Vui lòng dừng máy ảo trước khi thay đổi cài đặt.".to_string());
            }
        }

        let vm_dir = self.base_dir.join(&updated.id);
        if !vm_dir.exists() {
            return Err("Không tìm thấy thư mục máy ảo!".to_string());
        }

        let config_json = serde_json::to_string_pretty(&updated).map_err(|e| e.to_string())?;
        fs::write(vm_dir.join("config.json"), config_json)
            .map_err(|e| format!("Không thể lưu config.json: {}", e))?;

        Ok(updated)
    }

    pub fn start_vm(&self, id: &str) -> Result<(), String> {
        self.cleanup_dead_processes();
        {
            let running_map = self.running.lock().unwrap();
            if running_map.contains_key(id) {
                return Err("Máy ảo đang chạy rồi!".to_string());
            }
        }

        let cfg = self.get_vm(id).ok_or_else(|| "Không tìm thấy VM".to_string())?;
        let qemu_bin = self.qemu_exe();
        if !qemu_bin.exists() {
            return Err(format!("Không tìm thấy tệp thực thi QEMU tại {:?}", qemu_bin));
        }

        let vm_dir = self.base_dir.join(id);
        let log_path = vm_dir.join("vm.log");

        let log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_path)
            .map_err(|e| format!("Không thể mở file log: {}", e))?;

        let log_err = log_file.try_clone().map_err(|e| e.to_string())?;

        let args = QemuArgs::build(&cfg, &self.base_dir);

        let mut cmd = Command::new(&qemu_bin);
        cmd.args(&args)
            .stdout(log_file)
            .stderr(log_err);

        #[cfg(windows)]
        {
            // 0x08000000 = CREATE_NO_WINDOW: Chạy ngầm hoàn toàn, không tạo cửa sổ console đen trên màn hình desktop
            cmd.creation_flags(0x08000000);
        }

        let child = cmd.spawn()
            .map_err(|e| format!("Lỗi khởi động QEMU: {}", e))?;

        let mut running_map = self.running.lock().unwrap();
        running_map.insert(id.to_string(), child);

        Ok(())
    }

    pub fn stop_vm(&self, id: &str) -> Result<(), String> {
        let mut running_map = self.running.lock().unwrap();
        if let Some(mut child) = running_map.remove(id) {
            let _ = child.kill();
            let _ = child.wait();
            Ok(())
        } else {
            Err("Máy ảo không ở trạng thái chạy".to_string())
        }
    }

    pub fn delete_vm(&self, id: &str) -> Result<(), String> {
        let _ = self.stop_vm(id);
        let vm_dir = self.base_dir.join(id);
        if vm_dir.exists() {
            fs::remove_dir_all(&vm_dir).map_err(|e| format!("Lỗi xóa thư mục VM: {}", e))?;
        }
        Ok(())
    }

    pub fn open_vm_folder(&self, id: &str) -> Result<(), String> {
        let vm_dir = self.base_dir.join(id);
        if !vm_dir.exists() {
            return Err("Không tìm thấy thư mục máy ảo".to_string());
        }

        let mut cmd = Command::new("explorer.exe");
        cmd.arg(&vm_dir);
        #[cfg(windows)]
        cmd.creation_flags(0x08000000);
        let _ = cmd.spawn();

        Ok(())
    }

    pub fn set_vm_iso(&self, id: &str, iso: Option<String>) -> Result<(), String> {
        let mut cfg = self.get_vm(id).ok_or_else(|| "Không tìm thấy VM".to_string())?;
        cfg.iso_path = iso;
        self.update_vm_config(cfg)?;
        Ok(())
    }

    pub fn set_vm_accel(&self, id: &str, accel: &str) -> Result<(), String> {
        let mut cfg = self.get_vm(id).ok_or_else(|| "Không tìm thấy VM".to_string())?;
        cfg.accel = accel.to_string();
        self.update_vm_config(cfg)?;
        Ok(())
    }

    pub fn get_vm_logs(&self, id: &str) -> String {
        let log_path = self.base_dir.join(id).join("vm.log");
        if let Ok(content) = fs::read_to_string(log_path) {
            let lines: Vec<&str> = content.lines().collect();
            let start = if lines.len() > 100 { lines.len() - 100 } else { 0 };
            lines[start..].join("\n")
        } else {
            "Chưa có nhật ký hoạt động.".to_string()
        }
    }

    pub fn get_running_processes(&self) -> Vec<serde_json::Value> {
        self.cleanup_dead_processes();
        let running_map = self.running.lock().unwrap();
        let mut list = Vec::new();
        for (id, child) in running_map.iter() {
            let name = self.get_vm(id).map(|c| c.name).unwrap_or_else(|| id.clone());
            list.push(serde_json::json!({
                "id": id,
                "name": name,
                "pid": child.id(),
                "status": "Running (Tiến trình ngầm)"
            }));
        }
        list
    }

    pub fn stop_all_vms(&self) {
        let mut running_map = self.running.lock().unwrap();
        for (_, mut child) in running_map.drain() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    fn cleanup_dead_processes(&self) {
        let mut running_map = self.running.lock().unwrap();
        running_map.retain(|_, child| match child.try_wait() {
            Ok(Some(_)) => false,
            Ok(None) => true,
            Err(_) => false,
        });
    }
}
