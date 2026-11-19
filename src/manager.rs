use crate::config::VmConfig;
use crate::qemu::{QemuArgs, QemuImg};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};

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
    pub iso_path: Option<String>,
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
                        if let Ok(cfg) = serde_json::from_str::<VmConfig>(&content) {
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
                                iso_path: cfg.iso_path,
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
        if let Ok(content) = fs::read_to_string(config_file) {
            serde_json::from_str(&content).ok()
        } else {
            None
        }
    }

    pub fn create_vm(&self, cfg: VmConfig) -> Result<VmConfig, String> {
        let vm_dir = self.base_dir.join(&cfg.id);
        fs::create_dir_all(&vm_dir).map_err(|e| format!("Không thể tạo thư mục VM: {}", e))?;

        // Tạo đĩa chính
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

    #[allow(dead_code)]
    pub fn is_running(&self, id: &str) -> bool {
        self.cleanup_dead_processes();
        let running_map = self.running.lock().unwrap();
        running_map.contains_key(id)
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

        let args = QemuArgs::build(&cfg, &self.base_dir);
        let child = Command::new(&qemu_bin)
            .args(&args)
            .spawn()
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

    fn cleanup_dead_processes(&self) {
        let mut running_map = self.running.lock().unwrap();
        running_map.retain(|_, child| match child.try_wait() {
            Ok(Some(_)) => false, // Đã thoát
            Ok(None) => true,     // Đang chạy
            Err(_) => false,
        });
    }
}
