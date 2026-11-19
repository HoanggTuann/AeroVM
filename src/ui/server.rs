use crate::config::VmConfig;
use crate::manager::VmManager;
use serde::Deserialize;
use std::sync::Arc;
use tiny_http::{Header, Response, Server};

const INDEX_HTML: &str = include_str!("assets/index.html");
const STYLE_CSS: &str = include_str!("assets/style.css");
const APP_JS: &str = include_str!("assets/app.js");

#[derive(Deserialize)]
struct CreateVmRequest {
    name: String,
    os_type: String,
    #[serde(default)]
    iso: Option<String>,
    #[serde(default = "default_cores")]
    cpu_cores: u32,
    #[serde(default = "default_ram")]
    memory_mb: u32,
    #[serde(default = "default_disk")]
    disk_size_gb: u32,
    #[serde(default)]
    low_spec_mode: bool,
}

fn default_cores() -> u32 { 1 }
fn default_ram() -> u32 { 1024 }
fn default_disk() -> u32 { 16 }

#[derive(Deserialize)]
struct SetIsoRequest {
    id: String,
    iso: Option<String>,
}

#[derive(Deserialize)]
struct SetAccelRequest {
    id: String,
    accel: String,
}

#[derive(Deserialize)]
struct UpdateConfigRequest {
    name: String,
    cpu_cores: u32,
    memory_mb: u32,
    accel: String,
    machine_type: String,
    vga_card: String,
    low_spec_mode: bool,
    sound_enabled: bool,
    iso_path: Option<String>,
}

pub fn run_server<F>(manager: Arc<VmManager>, base_port: u16, on_ready: F)
where
    F: FnOnce(u16) + Send + 'static,
{
    let mut server = None;
    let mut active_port = base_port;

    for p in base_port..(base_port + 10) {
        let addr = format!("127.0.0.1:{}", p);
        if let Ok(s) = Server::http(&addr) {
            server = Some(s);
            active_port = p;
            break;
        }
    }

    let server = match server {
        Some(s) => s,
        None => {
            eprintln!("[!] Không thể lắng nghe trên các cổng từ {} đến {}", base_port, base_port + 10);
            return;
        }
    };

    on_ready(active_port);

    for mut request in server.incoming_requests() {
        let url = request.url().to_string();
        let method = request.method().as_str().to_string();

        if method == "GET" && (url == "/" || url == "/index.html") {
            let resp = Response::from_string(INDEX_HTML).with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap(),
            );
            let _ = request.respond(resp);
            continue;
        }

        if method == "GET" && url == "/style.css" {
            let resp = Response::from_string(STYLE_CSS).with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"text/css; charset=utf-8"[..]).unwrap(),
            );
            let _ = request.respond(resp);
            continue;
        }

        if method == "GET" && url == "/app.js" {
            let resp = Response::from_string(APP_JS).with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"application/javascript; charset=utf-8"[..]).unwrap(),
            );
            let _ = request.respond(resp);
            continue;
        }

        if method == "GET" && url == "/api/vms" {
            let list = manager.list_vms();
            let json = serde_json::to_string(&list).unwrap_or_else(|_| "[]".to_string());
            let resp = Response::from_string(json).with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
            );
            let _ = request.respond(resp);
            continue;
        }

        if method == "GET" && url == "/api/processes" {
            let procs = manager.get_running_processes();
            let json = serde_json::json!({ "ok": true, "processes": procs }).to_string();
            let resp = Response::from_string(json).with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
            );
            let _ = request.respond(resp);
            continue;
        }

        if method == "POST" && url == "/api/processes/stop_all" {
            manager.stop_all_vms();
            let resp = Response::from_string("{\"ok\":true}").with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
            );
            let _ = request.respond(resp);
            continue;
        }

        if method == "GET" && url.starts_with("/api/vms/") && url.ends_with("/logs") {
            let id = url.trim_start_matches("/api/vms/").trim_end_matches("/logs");
            let logs = manager.get_vm_logs(id);
            let json = serde_json::json!({ "ok": true, "logs": logs }).to_string();
            let resp = Response::from_string(json).with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
            );
            let _ = request.respond(resp);
            continue;
        }

        if method == "GET" && url.starts_with("/api/vms/") && !url.contains("/start") && !url.contains("/stop") && !url.contains("/open_folder") {
            let id = url.trim_start_matches("/api/vms/");
            if let Some(cfg) = manager.get_vm(id) {
                let json = serde_json::to_string(&cfg).unwrap_or_default();
                let resp = Response::from_string(json).with_header(
                    Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                );
                let _ = request.respond(resp);
            } else {
                let resp = Response::from_string("{\"ok\":false,\"error\":\"Not found\"}").with_status_code(404);
                let _ = request.respond(resp);
            }
            continue;
        }

        if method == "POST" && url == "/api/vms" {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);

            match serde_json::from_str::<CreateVmRequest>(&body) {
                Ok(req) => {
                    let mut cfg = VmConfig::new_template(&req.name, &req.os_type, req.iso);
                    cfg.cpu_cores = req.cpu_cores;
                    cfg.memory_mb = req.memory_mb;
                    cfg.low_spec_mode = req.low_spec_mode;
                    if req.low_spec_mode {
                        cfg.machine_type = "pc".to_string();
                        cfg.sound_enabled = false;
                        cfg.vga_card = "std".to_string();
                    }
                    if let Some(drive) = cfg.drives.get_mut(0) {
                        drive.size_gb = req.disk_size_gb;
                    }

                    match manager.create_vm(cfg) {
                        Ok(vm) => {
                            let json = serde_json::json!({ "ok": true, "vm": vm }).to_string();
                            let resp = Response::from_string(json).with_header(
                                Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                            );
                            let _ = request.respond(resp);
                            continue;
                        }
                        Err(e) => {
                            let json = serde_json::json!({ "ok": false, "error": e }).to_string();
                            let resp = Response::from_string(json);
                            let _ = request.respond(resp);
                            continue;
                        }
                    }
                }
                Err(err) => {
                    let json = serde_json::json!({ "ok": false, "error": format!("Lỗi dữ liệu: {}", err) }).to_string();
                    let resp = Response::from_string(json);
                    let _ = request.respond(resp);
                    continue;
                }
            }
        }

        if method == "POST" && url.starts_with("/api/vms/") && url.ends_with("/update") {
            let id = url.trim_start_matches("/api/vms/").trim_end_matches("/update");
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);

            match serde_json::from_str::<UpdateConfigRequest>(&body) {
                Ok(req) => {
                    if let Some(mut cfg) = manager.get_vm(id) {
                        cfg.name = req.name;
                        cfg.cpu_cores = req.cpu_cores;
                        cfg.memory_mb = req.memory_mb;
                        cfg.accel = req.accel;
                        cfg.machine_type = req.machine_type;
                        cfg.vga_card = req.vga_card;
                        cfg.low_spec_mode = req.low_spec_mode;
                        cfg.sound_enabled = req.sound_enabled;
                        cfg.iso_path = req.iso_path;

                        match manager.update_vm_config(cfg) {
                            Ok(saved) => {
                                let json = serde_json::json!({ "ok": true, "vm": saved }).to_string();
                                let resp = Response::from_string(json).with_header(
                                    Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                                );
                                let _ = request.respond(resp);
                            }
                            Err(e) => {
                                let json = serde_json::json!({ "ok": false, "error": e }).to_string();
                                let resp = Response::from_string(json);
                                let _ = request.respond(resp);
                            }
                        }
                    } else {
                        let resp = Response::from_string("{\"ok\":false,\"error\":\"Không tìm thấy VM\"}").with_status_code(404);
                        let _ = request.respond(resp);
                    }
                    continue;
                }
                Err(e) => {
                    let json = serde_json::json!({ "ok": false, "error": format!("JSON error: {}", e) }).to_string();
                    let resp = Response::from_string(json);
                    let _ = request.respond(resp);
                    continue;
                }
            }
        }

        if method == "POST" && url.starts_with("/api/vms/") && url.ends_with("/open_folder") {
            let id = url.trim_start_matches("/api/vms/").trim_end_matches("/open_folder");
            let res = manager.open_vm_folder(id);
            let json = match res {
                Ok(_) => serde_json::json!({ "ok": true }).to_string(),
                Err(e) => serde_json::json!({ "ok": false, "error": e }).to_string(),
            };
            let resp = Response::from_string(json).with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
            );
            let _ = request.respond(resp);
            continue;
        }

        if method == "POST" && url == "/api/vms/set_accel" {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            match serde_json::from_str::<SetAccelRequest>(&body) {
                Ok(req) => {
                    let res = manager.set_vm_accel(&req.id, &req.accel);
                    let json = match res {
                        Ok(_) => serde_json::json!({ "ok": true }).to_string(),
                        Err(e) => serde_json::json!({ "ok": false, "error": e }).to_string(),
                    };
                    let resp = Response::from_string(json).with_header(
                        Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                    );
                    let _ = request.respond(resp);
                    continue;
                }
                Err(e) => {
                    let json = serde_json::json!({ "ok": false, "error": format!("JSON error: {}", e) }).to_string();
                    let resp = Response::from_string(json);
                    let _ = request.respond(resp);
                    continue;
                }
            }
        }

        if method == "POST" && url == "/api/vms/set_iso" {
            let mut body = String::new();
            let _ = request.as_reader().read_to_string(&mut body);
            match serde_json::from_str::<SetIsoRequest>(&body) {
                Ok(req) => {
                    let res = manager.set_vm_iso(&req.id, req.iso);
                    let json = match res {
                        Ok(_) => serde_json::json!({ "ok": true }).to_string(),
                        Err(e) => serde_json::json!({ "ok": false, "error": e }).to_string(),
                    };
                    let resp = Response::from_string(json).with_header(
                        Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                    );
                    let _ = request.respond(resp);
                    continue;
                }
                Err(e) => {
                    let json = serde_json::json!({ "ok": false, "error": format!("JSON error: {}", e) }).to_string();
                    let resp = Response::from_string(json);
                    let _ = request.respond(resp);
                    continue;
                }
            }
        }

        if method == "POST" && url.starts_with("/api/vms/") && url.ends_with("/start") {
            let id = url.trim_start_matches("/api/vms/").trim_end_matches("/start");
            let res = manager.start_vm(id);
            let json = match res {
                Ok(_) => serde_json::json!({ "ok": true }).to_string(),
                Err(e) => serde_json::json!({ "ok": false, "error": e }).to_string(),
            };
            let resp = Response::from_string(json).with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
            );
            let _ = request.respond(resp);
            continue;
        }

        if method == "POST" && url.starts_with("/api/vms/") && url.ends_with("/stop") {
            let id = url.trim_start_matches("/api/vms/").trim_end_matches("/stop");
            let res = manager.stop_vm(id);
            let json = match res {
                Ok(_) => serde_json::json!({ "ok": true }).to_string(),
                Err(e) => serde_json::json!({ "ok": false, "error": e }).to_string(),
            };
            let resp = Response::from_string(json).with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
            );
            let _ = request.respond(resp);
            continue;
        }

        if method == "DELETE" && url.starts_with("/api/vms/") {
            let id = url.trim_start_matches("/api/vms/");
            let res = manager.delete_vm(id);
            let json = match res {
                Ok(_) => serde_json::json!({ "ok": true }).to_string(),
                Err(e) => serde_json::json!({ "ok": false, "error": e }).to_string(),
            };
            let resp = Response::from_string(json).with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
            );
            let _ = request.respond(resp);
            continue;
        }

        if method == "POST" && url == "/api/app/ping" {
            let resp = Response::from_string("{\"ok\":true}").with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
            );
            let _ = request.respond(resp);
            continue;
        }

        if method == "POST" && url == "/api/app/exit" {
            manager.stop_all_vms();
            let resp = Response::from_string("{\"ok\":true}");
            let _ = request.respond(resp);
            std::thread::spawn(|| {
                std::thread::sleep(std::time::Duration::from_millis(200));
                std::process::exit(0);
            });
            break;
        }

        let resp = Response::from_string("Not Found").with_status_code(404);
        let _ = request.respond(resp);
    }
}
