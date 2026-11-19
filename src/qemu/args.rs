use crate::config::VmConfig;
use std::path::Path;

pub struct QemuArgs;

impl QemuArgs {
    pub fn build(cfg: &VmConfig, base_dir: &Path) -> Vec<String> {
        let mut args = Vec::new();

        // Tên máy ảo
        args.push("-name".to_string());
        args.push(cfg.name.clone());

        // Máy ảo Q35 & Tăng tốc phần cứng WHPX
        args.push("-machine".to_string());
        if cfg.accel == "whpx" {
            args.push("q35,accel=whpx:tcg".to_string());
            args.push("-cpu".to_string());
            args.push("host".to_string());
        } else {
            args.push("q35,accel=tcg".to_string());
            args.push("-cpu".to_string());
            args.push("max".to_string());
        }

        // CPU Cores & RAM
        args.push("-smp".to_string());
        args.push(format!("cores={}", cfg.cpu_cores));

        args.push("-m".to_string());
        args.push(format!("{}M", cfg.memory_mb));

        // Bàn phím & Chuột tuyệt đối (USB Tablet không bị kẹt chuột)
        args.push("-device".to_string());
        args.push("nec-usb-xhci".to_string());
        args.push("-device".to_string());
        args.push("usb-tablet".to_string());

        // Card âm thanh Intel HDA
        args.push("-device".to_string());
        args.push("intel-hda".to_string());
        args.push("-device".to_string());
        args.push("hda-duplex".to_string());

        // Card đồ họa tối ưu theo hệ điều hành
        match cfg.os_type.as_str() {
            "windows" => {
                args.push("-vga".to_string());
                args.push("qxl".to_string());
            }
            "android" | "linux" | _ => {
                args.push("-device".to_string());
                args.push("virtio-vga".to_string());
            }
        }

        // Mạng VirtIO NAT
        args.push("-netdev".to_string());
        args.push("user,id=net0".to_string());
        args.push("-device".to_string());
        args.push("virtio-net-pci,netdev=net0".to_string());

        // Ổ đĩa ảo chính
        if let Some(disk_path) = cfg.primary_disk_path(base_dir) {
            args.push("-drive".to_string());
            args.push(format!(
                "file={},if=virtio,format=qcow2,cache=writeback",
                disk_path.to_string_lossy()
            ));
        }

        // Ổ đĩa CD-ROM cài đặt (ISO)
        if let Some(ref iso) = cfg.iso_path {
            if !iso.trim().is_empty() && std::path::Path::new(iso).exists() {
                args.push("-drive".to_string());
                args.push(format!("file={},media=cdrom,readonly=on", iso));
            }
        }

        // Menu Boot
        args.push("-boot".to_string());
        args.push("menu=on".to_string());

        // Hiển thị giao diện cửa sổ khách
        match cfg.display_type.as_str() {
            "sdl" => {
                args.push("-display".to_string());
                args.push("sdl".to_string());
            }
            "gtk" | _ => {
                args.push("-display".to_string());
                args.push("gtk,zoom-to-fit=on".to_string());
            }
        }

        args
    }
}
