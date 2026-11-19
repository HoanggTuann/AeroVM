use crate::config::VmConfig;
use crate::manager::VmManager;
use serde::Deserialize;
use std::sync::Arc;
use tiny_http::{Header, Response, Server};

const INDEX_HTML: &str = include_str!("assets/index.html");

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
}

fn default_cores() -> u32 { 2 }
fn default_ram() -> u32 { 4096 }
fn default_disk() -> u32 { 32 }

pub fn run_server(manager: Arc<VmManager>, port: u16) {
    let addr = format!("127.0.0.1:{}", port);
    let server = match Server::http(&addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[!] Không thể lắng nghe trên {}: {}", addr, e);
            return;
        }
    };

    println!("[*] AeroVM Dashboard đang chạy tại: http://{}", addr);

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

        if method == "GET" && url == "/api/vms" {
            let list = manager.list_vms();
            let json = serde_json::to_string(&list).unwrap_or_else(|_| "[]".to_string());
            let resp = Response::from_string(json).with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
            );
            let _ = request.respond(resp);
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
                    let json = serde_json::json!({ "ok": false, "error": format!("Lỗi dữ liệu: {} (body: {})", err, body) }).to_string();
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

        let resp = Response::from_string("Not Found").with_status_code(404);
        let _ = request.respond(resp);
    }
}
