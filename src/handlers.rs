use axum::{Json, response::IntoResponse};
use serde_json::{Map, Value};
use std::process::Command;
use sysinfo::{self, System};

pub async fn sysinfo() -> impl IntoResponse {
    let mut sys = System::new_all();

    let gpu_usage = Command::new("nvidia-smi")
        .arg("--query-gpu=utilization.gpu")
        .arg("--format=csv,noheader,nounits")
        .output()
        .expect("failed to run");

    let vram_usage = Command::new("nvidia-smi")
        .arg("--query-gpu=memory.used")
        .arg("--format=csv,noheader,nounits")
        .output()
        .expect("failed to run");

    let vram_total = Command::new("nvidia-smi")
        .arg("--query-gpu=memory.total")
        .arg("--format=csv,noheader,nounits")
        .output()
        .expect("failed to run");

    //println!("{}", String::from_utf8_lossy(&gpu_usage.stdout));

    sys.refresh_all();

    let mut map = Map::new();

    map.insert(
        "CPU Usage".to_string(),
        Value::String(sys.global_cpu_usage().to_string()),
    );
    map.insert(
        "Memory Usage".to_string(),
        Value::String(sys.used_memory().to_string()),
    );
    map.insert(
        "GPU Usage".to_string(),
        Value::String(
            String::from_utf8_lossy(&gpu_usage.stdout)
                .trim()
                .to_string(),
        ),
    );
    map.insert(
        "VRAM Usage".to_string(),
        Value::String(
            String::from_utf8_lossy(&vram_usage.stdout)
                .trim()
                .to_string(),
        ),
    );
    map.insert(
        "VRAM Total".to_string(),
        Value::String(
            String::from_utf8_lossy(&vram_total.stdout)
                .trim()
                .to_string(),
        ),
    );

    let data = Value::Object(map);
    Json(data)
}
