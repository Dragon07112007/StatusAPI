use axum::{Json, response::IntoResponse};
use serde_json::{Map, Value};
use std::io::BufWriter;
use std::io::BufReader;
use std::process::Command;
use std::fs::File;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::system_info::{SysInfo, SystemSample};



pub async fn sysinfo() -> impl IntoResponse {
    
    let sysinfo = SysInfo::new();

    let vram_total = Command::new("nvidia-smi")
        .arg("--query-gpu=memory.total")
        .arg("--format=csv,noheader,nounits")
        .output()
        .expect("failed to run");

    let mut map = Map::new();

    map.insert(
        "CPU Usage".to_string(),
        Value::String(sysinfo.cpu_usage()),
    );
    map.insert(
        "Memory Usage".to_string(),
        Value::String(sysinfo.memory_usage()),
    );
    map.insert(
        "Memory Total".to_string(),
        Value::String(sysinfo.sys.total_memory().to_string()),
    );
    map.insert(
        "GPU Usage".to_string(),
        Value::String(sysinfo.gpu_usage()),
    );
    map.insert(
        "VRAM Usage".to_string(),
        Value::String(sysinfo.vram_usage()),
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


pub async fn syslog() -> impl IntoResponse{

    let file = File::open("logs/data.json").expect("Failed to open file");
    let reader = BufReader::new(file);
    let json: Value = serde_json::from_reader(reader).expect("Failed to pars.");

    Json(json)
}

pub async fn write_logs(sample: SystemSample){

    let file = File::open("logs/data.json").expect("Unable to open file!");
    let reader = BufReader::new(file);
    let mut json: Value = serde_json::from_reader(reader).expect("Failed to pars.");

    if !json.is_array() {
        json = Value::Array(vec![]);
    }

    let mut map = serde_json::Map::new();

    map.insert(
        "Time".to_string(),
        Value::String(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string()),
    );
    map.insert(
        "CPU Usage".to_string(),
        Value::String(format!("{:.2}", sample.cpu_usage)),
    );
    map.insert(
        "Memory Usage".to_string(),
        Value::String(sample.memory_usage.to_string()),
    );
    map.insert(
        "GPU Usage".to_string(),
        Value::String(format!("{:.2}", sample.gpu_usage)),
    );
    map.insert(
        "VRAM Usage".to_string(),
        Value::String(sample.vram_usage.to_string()),
    );    

    if let Some(array) = json.as_array_mut() {
        array.push(Value::Object(map));
        
    } 

    let file = File::create("logs/data.json").expect("Unabler to create file");
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &json).expect("Failed to write!");
    

    println!("Logs written!")
}
