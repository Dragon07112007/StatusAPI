use axum::{Json, response::IntoResponse};
use serde_json::{Map, Value, from_reader};
use std::array;
use std::io::BufWriter;
use std::{io::BufReader, process::Command};
use std::fs::File;
use std::time::{self, SystemTime, UNIX_EPOCH};
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
        "Memory Total".to_string(),
        Value::String(sys.total_memory().to_string()),
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


pub async fn syslog() -> impl IntoResponse{

    let file = File::open("logs/data.json").expect("Failed to open file");
    let reader = BufReader::new(file);
    let json: Value = serde_json::from_reader(reader).expect("Failed to pars.");

    Json(json)
}

pub async fn write_logs(){

    let file = File::open("logs/data.json").expect("Unable to open file!");
    let reader = BufReader::new(file);
    let mut json: Value = serde_json::from_reader(reader).expect("Failed to pars.");

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


    if !json.is_array() {
        json = Value::Array(vec![]);
    }

    let mut sys = System::new_all();
    sys.refresh_all();

    let mut map = serde_json::Map::new();

    map.insert(
        "Time".to_string(),
        Value::String(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string()),
    );
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

    if let Some(array) = json.as_array_mut() {
        array.push(Value::Object(map));
        
    } 

    let file = File::create("logs/data.json").expect("Unabler to create file");
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &json).expect("Failed to write!");
    

    println!("Logs written!")
}