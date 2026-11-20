use crate::config::VmConfig;
use crate::manager::{VmManager, VmSummary};
use eframe::egui;
use std::sync::Arc;
use std::time::Instant;

#[derive(PartialEq, Clone, Copy)]
enum ViewMode {
    Overview,
    Settings,
}

pub struct AeroVmApp {
    manager: Arc<VmManager>,
    vms: Vec<VmSummary>,
    selected_vm_id: Option<String>,
    current_view: ViewMode,

    // Trạng thái màn hình Cài đặt (Settings)
    edit_config: Option<VmConfig>,
    status_notification: Option<(String, bool, Instant)>, // (message, is_error, time)

    // Trạng thái hộp thoại Tạo máy ảo mới
    show_create_modal: bool,
    new_vm_name: String,
    new_vm_os: String,
    new_vm_cores: u32,
    new_vm_ram: u32,
    new_vm_disk: u32,
    new_vm_iso: String,
    new_vm_low_spec: bool,

    // Log viewer
    log_content: String,
    last_refresh: Instant,
}

impl AeroVmApp {
    pub fn new(cc: &eframe::CreationContext<'_>, manager: Arc<VmManager>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        setup_dark_theme(&cc.egui_ctx);

        let vms = manager.list_vms();
        let first_id = vms.first().map(|v| v.id.clone());

        Self {
            manager,
            vms,
            selected_vm_id: first_id,
            current_view: ViewMode::Overview,

            edit_config: None,
            status_notification: None,

            show_create_modal: false,
            new_vm_name: "Ubuntu 24.04".to_string(),
            new_vm_os: "linux".to_string(),
            new_vm_cores: 2,
            new_vm_ram: 2048,
            new_vm_disk: 32,
            new_vm_iso: String::new(),
            new_vm_low_spec: false,

            log_content: String::new(),
            last_refresh: Instant::now(),
        }
    }

    fn refresh_vms(&mut self) {
        self.vms = self.manager.list_vms();
        if let Some(ref id) = self.selected_vm_id {
            if !self.vms.iter().any(|v| &v.id == id) {
                self.selected_vm_id = self.vms.first().map(|v| v.id.clone());
            }
        } else {
            self.selected_vm_id = self.vms.first().map(|v| v.id.clone());
        }
    }

    fn open_settings_for_selected(&mut self) {
        if let Some(ref id) = self.selected_vm_id {
            if let Some(cfg) = self.manager.get_vm(id) {
                self.edit_config = Some(cfg);
                self.current_view = ViewMode::Settings;
            }
        }
    }

    fn save_current_settings(&mut self) {
        if let Some(cfg) = self.edit_config.clone() {
            match self.manager.update_vm_config(cfg) {
                Ok(saved) => {
                    self.set_notification(format!("Đã lưu cài đặt cho máy ảo '{}' thành công!", saved.name), false);
                    self.refresh_vms();
                }
                Err(err) => {
                    self.set_notification(format!("Lỗi lưu cài đặt: {}", err), true);
                }
            }
        }
    }

    fn set_notification(&mut self, msg: String, is_error: bool) {
        self.status_notification = Some((msg, is_error, Instant::now()));
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let font_candidates = [
        "C:\\Windows\\Fonts\\segoeui.ttf",
        "C:\\Windows\\Fonts\\arial.ttf",
        "C:\\Windows\\Fonts\\tahoma.ttf",
    ];

    for path in &font_candidates {
        if let Ok(font_bytes) = std::fs::read(path) {
            fonts.font_data.insert(
                "vietnamese_font".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(font_bytes)),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "vietnamese_font".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "vietnamese_font".to_owned());
            break;
        }
    }

    ctx.set_fonts(fonts);
}

fn setup_dark_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = egui::Color32::from_rgb(13, 17, 26);
    visuals.window_fill = egui::Color32::from_rgb(18, 24, 38);
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(22, 29, 44);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(27, 36, 56);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(38, 52, 80);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(56, 139, 253);
    visuals.widgets.open.bg_fill = egui::Color32::from_rgb(35, 48, 74);
    ctx.set_visuals(visuals);
}

fn pick_iso_file() -> Option<String> {
    let script = r#"
Add-Type -AssemblyName System.Windows.Forms
$dlg = New-Object System.Windows.Forms.OpenFileDialog
$dlg.Filter = "Tệp đĩa ISO (*.iso)|*.iso|Tất cả tệp (*.*)|*.*"
$dlg.Title = "Chọn tệp ảnh ISO cài đặt cho AeroVM"
if ($dlg.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
    Write-Output $dlg.FileName
}
"#;
    let mut cmd = std::process::Command::new("powershell");
    cmd.args(["-NoProfile", "-Command", script]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    if let Ok(output) = cmd.output() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() && std::path::Path::new(&path).exists() {
            return Some(path);
        }
    }
    None
}

impl eframe::App for AeroVmApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Tự động làm mới danh sách máy ảo mỗi 2 giây
        if self.last_refresh.elapsed().as_secs() >= 2 {
            self.refresh_vms();
            self.last_refresh = Instant::now();
        }

        // ==========================================
        // 1. THANH BÊN TRÁI (SIDEBAR DANH SÁCH MÁY ẢO)
        // ==========================================
        egui::SidePanel::left("sidebar_panel")
            .resizable(false)
            .default_width(310.0)
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(16, 22, 34)).inner_margin(16.0))
            .show(ctx, |ui| {
                // Header thương hiệu
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🚀").size(24.0));
                    ui.vertical(|ui| {
                        ui.heading(egui::RichText::new("AeroVM Studio").size(17.0).color(egui::Color32::WHITE).strong());
                        ui.label(egui::RichText::new("100% Native Rust Hypervisor").size(11.0).color(egui::Color32::from_rgb(88, 166, 255)));
                    });
                });

                ui.add_space(14.0);

                // Nút Tạo máy ảo mới
                let btn_create = egui::Button::new(egui::RichText::new("+ Tạo máy ảo mới").size(14.0).strong().color(egui::Color32::WHITE))
                    .fill(egui::Color32::from_rgb(46, 160, 67))
                    .min_size(egui::vec2(ui.available_width(), 38.0));
                if ui.add(btn_create).clicked() {
                    self.show_create_modal = true;
                }

                ui.add_space(12.0);
                ui.label(egui::RichText::new("DANH SÁCH MÁY ẢO").size(11.0).strong().color(egui::Color32::from_rgb(110, 118, 129)));
                ui.add_space(4.0);

                // Danh sách máy ảo
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if self.vms.is_empty() {
                        ui.add_space(20.0);
                        ui.label(egui::RichText::new("Chưa có máy ảo nào.\nNhấn nút trên để tạo máy ảo mới.").size(12.5).color(egui::Color32::GRAY));
                    } else {
                        for vm in &self.vms {
                            let is_selected = self.selected_vm_id.as_deref() == Some(&vm.id);
                            let bg_color = if is_selected {
                                egui::Color32::from_rgb(30, 48, 78)
                            } else {
                                egui::Color32::from_rgb(22, 29, 44)
                            };

                            let icon = match vm.os_type.as_str() {
                                "windows" => "🪟",
                                "android" => "🤖",
                                "slitaz" => "⚡",
                                _ => "🐧",
                            };

                            let status_dot = if vm.running { "🟢" } else { "⚪" };
                            let accel_badge = if vm.accel == "whpx" { "WHPX" } else { "TCG" };

                            let frame = egui::Frame::none()
                                .fill(bg_color)
                                .rounding(8.0)
                                .inner_margin(10.0);

                            frame.show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(icon).size(20.0));
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new(&vm.name).size(13.5).strong().color(egui::Color32::WHITE));
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                ui.label(egui::RichText::new(status_dot).size(10.0));
                                            });
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new(accel_badge).size(10.5).color(egui::Color32::from_rgb(88, 166, 255)));
                                            ui.label(egui::RichText::new(format!("· {} Cores · {} MB", vm.cpu_cores, vm.memory_mb)).size(11.0).color(egui::Color32::GRAY));
                                        });
                                    });
                                });
                            });

                            // Xử lý click chọn máy ảo
                            if ui.interact(ui.min_rect(), egui::Id::new(&vm.id), egui::Sense::click()).clicked() {
                                self.selected_vm_id = Some(vm.id.clone());
                                self.current_view = ViewMode::Overview;
                                self.edit_config = None;
                            }
                            ui.add_space(4.0);
                        }
                    }
                });

                // Footer sidebar
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.add_space(8.0);
                    let running_count = self.vms.iter().filter(|v| v.running).count();
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("Đang chạy: {} máy ảo", running_count)).size(11.5).color(egui::Color32::LIGHT_GRAY));
                        if running_count > 0 {
                            if ui.button(egui::RichText::new("🛑 Dừng tất cả").size(11.0).color(egui::Color32::from_rgb(248, 81, 73))).clicked() {
                                self.manager.stop_all_vms();
                                self.refresh_vms();
                                self.set_notification("Đã dừng tất cả máy ảo đang chạy!".to_string(), false);
                            }
                        }
                    });
                });
            });

        // ==========================================
        // 2. KHU VỰC CHÍNH (MAIN WORKSPACE)
        // ==========================================
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(11, 15, 23)).inner_margin(24.0))
            .show(ctx, |ui| {
                // Hiển thị thông báo nếu có
                if let Some((msg, is_error, time)) = &self.status_notification {
                    if time.elapsed().as_secs() < 4 {
                        let bg = if *is_error { egui::Color32::from_rgb(90, 20, 20) } else { egui::Color32::from_rgb(20, 70, 30) };
                        egui::Frame::none().fill(bg).rounding(6.0).inner_margin(8.0).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(if *is_error { "⚠️" } else { "✅" }).size(14.0));
                                ui.label(egui::RichText::new(msg).size(13.0).color(egui::Color32::WHITE));
                            });
                        });
                        ui.add_space(10.0);
                    }
                }

                let current_vm = self.selected_vm_id.as_ref().and_then(|id| self.vms.iter().find(|v| &v.id == id)).cloned();

                match current_vm {
                    None => {
                        // Empty State khi chưa chọn máy ảo
                        ui.vertical_centered(|ui| {
                            ui.add_space(120.0);
                            ui.label(egui::RichText::new("🪐").size(56.0));
                            ui.add_space(10.0);
                            ui.heading(egui::RichText::new("Chưa chọn máy ảo nào").size(20.0).color(egui::Color32::WHITE));
                            ui.add_space(6.0);
                            ui.label(egui::RichText::new("Chọn một máy ảo từ thanh danh sách bên trái hoặc nhấn nút để tạo mới.").color(egui::Color32::GRAY));
                            ui.add_space(16.0);
                            if ui.button(egui::RichText::new("+ Tạo máy ảo mới").size(14.0)).clicked() {
                                self.show_create_modal = true;
                            }
                        });
                    }
                    Some(vm) => {
                        // Thanh công cụ phía trên
                        ui.horizontal(|ui| {
                            ui.heading(egui::RichText::new(&vm.name).size(22.0).color(egui::Color32::WHITE).strong());
                            let (badge_text, badge_color) = if vm.running {
                                ("🟢 Đang chạy", egui::Color32::from_rgb(63, 185, 80))
                            } else {
                                ("⚪ Đã dừng", egui::Color32::GRAY)
                            };
                            ui.label(egui::RichText::new(badge_text).size(12.0).color(badge_color));

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // Nút Xóa
                                if ui.button(egui::RichText::new("🗑️ Xóa").size(12.5).color(egui::Color32::from_rgb(248, 81, 73))).clicked() {
                                    let _ = self.manager.delete_vm(&vm.id);
                                    self.selected_vm_id = None;
                                    self.refresh_vms();
                                    self.set_notification("Đã xóa máy ảo thành công.".to_string(), false);
                                    return;
                                }

                                // Nút Mở thư mục
                                if ui.button(egui::RichText::new("📂 Thư mục VM").size(12.5)).clicked() {
                                    let _ = self.manager.open_vm_folder(&vm.id);
                                    self.set_notification("Đã mở thư mục máy ảo trong Windows Explorer.".to_string(), false);
                                }

                                // Nút Cài đặt
                                if self.current_view == ViewMode::Settings {
                                    // Nút Thoát cài đặt ở thanh tiêu đề theo yêu cầu
                                    if ui.button(egui::RichText::new("❌ Thoát cài đặt").size(12.5).color(egui::Color32::LIGHT_RED)).clicked() {
                                        self.current_view = ViewMode::Overview;
                                        self.edit_config = None;
                                    }
                                } else {
                                    if ui.button(egui::RichText::new("⚙️ Cài đặt").size(12.5)).clicked() {
                                        self.open_settings_for_selected();
                                    }
                                }

                                // Nút Dừng / Khởi động
                                if vm.running {
                                    if ui.add(egui::Button::new(egui::RichText::new("⏹ Dừng máy").size(13.0).strong().color(egui::Color32::WHITE))
                                        .fill(egui::Color32::from_rgb(180, 40, 40))).clicked() {
                                        let _ = self.manager.stop_vm(&vm.id);
                                        self.refresh_vms();
                                        self.set_notification("Máy ảo đã dừng an toàn.".to_string(), false);
                                    }
                                } else {
                                    if ui.add(egui::Button::new(egui::RichText::new("▶ Khởi động").size(13.0).strong().color(egui::Color32::WHITE))
                                        .fill(egui::Color32::from_rgb(46, 160, 67))).clicked() {
                                        match self.manager.start_vm(&vm.id) {
                                            Ok(_) => {
                                                self.refresh_vms();
                                                self.set_notification("Máy ảo đã khởi động qua Native SDL!".to_string(), false);
                                            }
                                            Err(err) => {
                                                self.set_notification(format!("Lỗi khởi động: {}", err), true);
                                            }
                                        }
                                    }
                                }
                            });
                        });

                        ui.add_space(14.0);
                        ui.separator();
                        ui.add_space(10.0);

                        // Hiển thị theo chế độ: Cài đặt hoặc Tổng quan
                        if self.current_view == ViewMode::Settings {
                            self.render_settings_view(ui, &vm);
                        } else {
                            self.render_overview_view(ui, &vm);
                        }
                    }
                }
            });

        // ==========================================
        // 3. HỘP THOẠI TẠO MÁY ẢO MỚI (MODAL)
        // ==========================================
        if self.show_create_modal {
            egui::Window::new("Tạo máy ảo mới (UTM Style)")
                .collapsible(false)
                .resizable(false)
                .fixed_size(egui::vec2(520.0, 520.0))
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new("Chọn Hệ Điều Hành Mẫu:").strong());
                    ui.horizontal(|ui| {
                        let os_list = [("linux", "🐧 Linux"), ("windows", "🪟 Windows"), ("android", "🤖 Android"), ("slitaz", "⚡ SliTaz")];
                        for (key, label) in os_list {
                            let is_sel = self.new_vm_os == key;
                            if ui.selectable_label(is_sel, label).clicked() {
                                self.new_vm_os = key.to_string();
                                match key {
                                    "windows" => {
                                        self.new_vm_name = "Windows 11 Pro".to_string();
                                        self.new_vm_cores = 4;
                                        self.new_vm_ram = 4096;
                                        self.new_vm_disk = 64;
                                        self.new_vm_low_spec = false;
                                    }
                                    "android" => {
                                        self.new_vm_name = "Android-x86".to_string();
                                        self.new_vm_cores = 2;
                                        self.new_vm_ram = 2048;
                                        self.new_vm_disk = 16;
                                        self.new_vm_low_spec = false;
                                    }
                                    "slitaz" => {
                                        self.new_vm_name = "SliTaz GNU/Linux (Live)".to_string();
                                        self.new_vm_cores = 1;
                                        self.new_vm_ram = 512;
                                        self.new_vm_disk = 4;
                                        self.new_vm_low_spec = true;
                                    }
                                    _ => {
                                        self.new_vm_name = "Ubuntu 24.04 LTS".to_string();
                                        self.new_vm_cores = 2;
                                        self.new_vm_ram = 2048;
                                        self.new_vm_disk = 32;
                                        self.new_vm_low_spec = false;
                                    }
                                }
                            }
                        }
                    });

                    ui.add_space(10.0);
                    ui.label("Tên máy ảo:");
                    ui.text_edit_singleline(&mut self.new_vm_name);

                    ui.add_space(8.0);
                    ui.checkbox(&mut self.new_vm_low_spec, "⚡ Chế độ máy cấu hình yếu (Tối ưu RAM & CPU thấp)");

                    ui.add_space(8.0);
                    ui.label(format!("Số nhân CPU: {} Cores", self.new_vm_cores));
                    ui.add(egui::Slider::new(&mut self.new_vm_cores, 1..=8));

                    ui.add_space(8.0);
                    ui.label(format!("Dung lượng RAM: {} MB", self.new_vm_ram));
                    ui.add(egui::Slider::new(&mut self.new_vm_ram, 256..=16384).step_by(256.0));

                    ui.add_space(8.0);
                    ui.label(format!("Kích thước ổ đĩa: {} GB", self.new_vm_disk));
                    ui.add(egui::Slider::new(&mut self.new_vm_disk, 4..=256).step_by(4.0));

                    ui.add_space(8.0);
                    ui.label("Tệp ISO cài đặt (Tùy chọn):");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.new_vm_iso);
                        if ui.button("Duyệt...").clicked() {
                            if let Some(path) = pick_iso_file() {
                                self.new_vm_iso = path;
                            }
                        }
                    });

                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("Hủy").size(13.0)).clicked() {
                            self.show_create_modal = false;
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add(egui::Button::new(egui::RichText::new("✨ Tạo máy ảo").size(13.5).strong().color(egui::Color32::WHITE))
                                .fill(egui::Color32::from_rgb(46, 160, 67))).clicked() {
                                let iso_opt = if self.new_vm_iso.trim().is_empty() { None } else { Some(self.new_vm_iso.trim().to_string()) };
                                let mut cfg = VmConfig::new_template(&self.new_vm_name, &self.new_vm_os, iso_opt);
                                cfg.cpu_cores = self.new_vm_cores;
                                cfg.memory_mb = self.new_vm_ram;
                                cfg.low_spec_mode = self.new_vm_low_spec;
                                if self.new_vm_low_spec {
                                    cfg.machine_type = "pc".to_string();
                                    cfg.sound_enabled = false;
                                    cfg.vga_card = "std".to_string();
                                }
                                if let Some(drive) = cfg.drives.get_mut(0) {
                                    drive.size_gb = self.new_vm_disk;
                                }

                                match self.manager.create_vm(cfg) {
                                    Ok(created) => {
                                        self.show_create_modal = false;
                                        self.selected_vm_id = Some(created.id.clone());
                                        self.refresh_vms();
                                        self.set_notification(format!("Đã tạo máy ảo '{}' thành công!", created.name), false);
                                    }
                                    Err(err) => {
                                        self.set_notification(format!("Lỗi tạo máy ảo: {}", err), true);
                                    }
                                }
                            }
                        });
                    });
                });
        }
    }
}

impl AeroVmApp {
    // ==========================================
    // MÀN HÌNH TỔNG QUAN (OVERVIEW)
    // ==========================================
    fn render_overview_view(&mut self, ui: &mut egui::Ui, vm: &VmSummary) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading(egui::RichText::new("Thông số phần cứng máy ảo").size(15.0).color(egui::Color32::WHITE));
            ui.add_space(8.0);

            egui::Grid::new("vm_specs_grid")
                .num_columns(2)
                .spacing([24.0, 12.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("⚡ Engine Ảo Hóa:").strong());
                    let accel_text = if vm.accel == "whpx" { "WHPX (Windows Hypervisor Platform - Phần cứng)" } else { "TCG (Mô phỏng phần mềm - Tương thích 100%)" };
                    ui.label(accel_text);
                    ui.end_row();

                    ui.label(egui::RichText::new("🧠 Vi Xử Lý & RAM:").strong());
                    ui.label(format!("{} Nhân CPU · {} MB RAM (Chế độ máy yếu: {})", vm.cpu_cores, vm.memory_mb, if vm.low_spec_mode { "Bật" } else { "Tắt" }));
                    ui.end_row();

                    ui.label(egui::RichText::new("💾 Ổ Đĩa Ảo Chính:").strong());
                    ui.label(format!("{} GB (Định dạng QCOW2 nén động)", vm.disk_size_gb));
                    ui.end_row();

                    ui.label(egui::RichText::new("💿 Đĩa Khởi Động (ISO):").strong());
                    let iso_disp = vm.iso_path.as_deref().unwrap_or("Chưa gắn tệp ISO");
                    ui.label(iso_disp);
                    ui.end_row();

                    ui.label(egui::RichText::new("🖥️ Giao Diện Hiển Thị:").strong());
                    ui.label("Native SDL2 Display (DirectX / Win32)");
                    ui.end_row();

                    ui.label(egui::RichText::new("📂 Thư Mục Lưu Trữ:").strong());
                    ui.label(&vm.folder_path);
                    ui.end_row();
                });

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            // Khung xem nhật ký hoạt động QEMU
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("Nhật ký hoạt động QEMU Console").size(15.0).color(egui::Color32::WHITE));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🔄 Làm mới nhật ký").clicked() {
                        self.log_content = self.manager.get_vm_logs(&vm.id);
                    }
                });
            });

            ui.add_space(8.0);
            if self.log_content.is_empty() {
                self.log_content = self.manager.get_vm_logs(&vm.id);
            }

            egui::Frame::none()
                .fill(egui::Color32::from_rgb(4, 6, 10))
                .rounding(6.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(240.0)
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new(&self.log_content)
                                .font(egui::FontId::monospace(12.0))
                                .color(egui::Color32::from_rgb(152, 195, 121)));
                        });
                });
        });
    }

    // ==========================================
    // MÀN HÌNH CÀI ĐẶT MÁY ẢO (SETTINGS)
    // ==========================================
    fn render_settings_view(&mut self, ui: &mut egui::Ui, _vm: &VmSummary) {
        if self.edit_config.is_none() {
            self.open_settings_for_selected();
        }

        let mut exit_settings = false;
        let mut save_settings = false;

        if let Some(ref mut cfg) = self.edit_config {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new("⚙️ Tùy Chỉnh Cấu Hình Máy Ảo").size(17.0).color(egui::Color32::WHITE).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Nút Thoát theo yêu cầu của người dùng
                        if ui.button(egui::RichText::new("❌ Thoát").size(13.0).color(egui::Color32::LIGHT_RED)).clicked() {
                            exit_settings = true;
                        }
                    });
                });
                ui.add_space(10.0);

                // 1. Tên máy ảo
                ui.label(egui::RichText::new("Tên máy ảo:").strong());
                ui.text_edit_singleline(&mut cfg.name);
                ui.add_space(10.0);

                // 2. Chế độ Tăng tốc ảo hóa
                ui.label(egui::RichText::new("Chế độ Tăng tốc (Engine):").strong());
                ui.horizontal(|ui| {
                    ui.radio_value(&mut cfg.accel, "whpx".to_string(), "⚡ WHPX (Phần cứng - Tốc độ cao)");
                    ui.radio_value(&mut cfg.accel, "tcg".to_string(), "🔄 TCG (Mô phỏng - Tương thích 100% cho SliTaz)");
                });
                ui.label(egui::RichText::new("* Lưu ý: Chọn TCG cho SliTaz hoặc Linux cổ để không bị lỗi ngắt timer.").size(11.0).color(egui::Color32::GRAY));
                ui.add_space(10.0);

                // 3. Chế độ máy cấu hình yếu
                ui.checkbox(&mut cfg.low_spec_mode, "⚡ Chế độ máy cấu hình yếu (Tối ưu RAM 512MB, 1 CPU, bo mạch i440fx nhẹ)");
                ui.add_space(10.0);

                // 4. Số nhân CPU & RAM
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(format!("Số nhân CPU: {} Cores", cfg.cpu_cores)).strong());
                        ui.add(egui::Slider::new(&mut cfg.cpu_cores, 1..=8));
                    });
                    ui.add_space(20.0);
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(format!("Dung lượng RAM: {} MB", cfg.memory_mb)).strong());
                        ui.add(egui::Slider::new(&mut cfg.memory_mb, 256..=16384).step_by(256.0));
                    });
                });
                ui.add_space(10.0);

                // 5. Chipset Bo Mạch & Card Màn Hình
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Bo Mạch Chủ (Chipset):").strong());
                        egui::ComboBox::from_id_salt("combo_machine")
                            .selected_text(match cfg.machine_type.as_str() {
                                "q35" => "q35 (Hiện đại PCIe)",
                                _ => "pc (i440fx - Khuyên dùng cho máy yếu)",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut cfg.machine_type, "pc".to_string(), "pc (i440fx - Nhẹ)");
                                ui.selectable_value(&mut cfg.machine_type, "q35".to_string(), "q35 (Hiện đại PCIe)");
                            });
                    });

                    ui.add_space(20.0);

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Card Màn Hình (VGA):").strong());
                        egui::ComboBox::from_id_salt("combo_vga")
                            .selected_text(match cfg.vga_card.as_str() {
                                "cirrus" => "cirrus (Cực nhẹ, tiết kiệm VRAM)",
                                "virtio" => "virtio-vga (Tăng tốc Linux)",
                                "qxl" => "qxl (SPICE)",
                                _ => "std (Chuẩn VGA ổn định nhất)",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut cfg.vga_card, "std".to_string(), "std (Chuẩn)");
                                ui.selectable_value(&mut cfg.vga_card, "cirrus".to_string(), "cirrus (Siêu nhẹ)");
                                ui.selectable_value(&mut cfg.vga_card, "virtio".to_string(), "virtio-vga");
                                ui.selectable_value(&mut cfg.vga_card, "qxl".to_string(), "qxl");
                            });
                    });
                });
                ui.add_space(10.0);

                // 6. Âm thanh
                ui.checkbox(&mut cfg.sound_enabled, "Bật card âm thanh Intel HDA (Tắt để giảm tải CPU trên máy yếu)");
                ui.add_space(10.0);

                // 7. Đĩa ISO cài đặt
                ui.label(egui::RichText::new("Tệp ảnh ISO nạp lúc khởi động:").strong());
                let mut iso_str = cfg.iso_path.clone().unwrap_or_default();
                ui.horizontal(|ui| {
                    if ui.text_edit_singleline(&mut iso_str).changed() {
                        cfg.iso_path = if iso_str.trim().is_empty() { None } else { Some(iso_str.clone()) };
                    }
                    if ui.button("Duyệt ISO...").clicked() {
                        if let Some(path) = pick_iso_file() {
                            cfg.iso_path = Some(path);
                        }
                    }
                    if ui.button("Gỡ ISO").clicked() {
                        cfg.iso_path = None;
                    }
                });

                ui.add_space(24.0);
                ui.separator();
                ui.add_space(10.0);

                // NÚT HÀNH ĐỘNG DƯỚI CÙNG: CÓ NÚT "THOÁT" RÕ RÀNG
                ui.horizontal(|ui| {
                    // Nút Thoát cài đặt
                    if ui.button(egui::RichText::new("❌ Thoát (Không lưu)").size(13.5)).clicked() {
                        exit_settings = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Nút Lưu cài đặt
                        if ui.add(egui::Button::new(egui::RichText::new("💾 Lưu cài đặt").size(13.5).strong().color(egui::Color32::WHITE))
                            .fill(egui::Color32::from_rgb(46, 160, 67))).clicked() {
                            save_settings = true;
                        }
                    });
                });
            });
        }

        if save_settings {
            self.save_current_settings();
            self.current_view = ViewMode::Overview;
            self.edit_config = None;
        } else if exit_settings {
            self.current_view = ViewMode::Overview;
            self.edit_config = None;
        }
    }
}

pub fn run_native_app(manager: Arc<VmManager>) -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 780.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title("AeroVM - Trình Quản Lý Máy Ảo UTM")
            .with_decorations(true) // Cửa sổ Windows native chuẩn: có nút thu nhỏ, phóng to, đóng góc phải trên
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "AeroVM",
        options,
        Box::new(|cc| Ok(Box::new(AeroVmApp::new(cc, manager)))),
    )
}
