use axum::{Json, response::IntoResponse};
use serde_json::{Map, Value};
use std::fs::{File, OpenOptions};
use std::io::BufReader;
use std::io::{Read, Seek, SeekFrom, Write};
use std::process::Command;
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

    append_log_entry("logs/data.json", &Value::Object(map));

    println!("Logs written!")
}

fn append_log_entry(path: &str, entry: &Value) {
    let formatted_entry = indent_json_block(
        &serde_json::to_string_pretty(entry).expect("Failed to serialize log entry"),
    );

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .expect("Unable to open log file");

    let file_len = file.metadata().expect("Unable to read log metadata").len();

    if file_len == 0 {
        write!(file, "[\n{formatted_entry}\n]\n").expect("Failed to initialize log file");
        return;
    }

    let closing_bracket_pos = find_last_non_whitespace(&mut file)
        .expect("Log file is empty or contains only whitespace");

    file.seek(SeekFrom::Start(closing_bracket_pos))
        .expect("Failed to seek to insertion point");

    let mut marker: [u8; 1] = [0; 1];
    file.read_exact(&mut marker)
        .expect("Failed to read log terminator");

    file.seek(SeekFrom::Start(closing_bracket_pos))
        .expect("Failed to seek to insertion point");

    match marker[0] {
        b'[' => {
            write!(file, "[\n{formatted_entry}\n]\n").expect("Failed to append first log entry");
        }
        b']' => {
            file.set_len(closing_bracket_pos)
                .expect("Failed to truncate trailing bracket");
            file.seek(SeekFrom::Start(closing_bracket_pos))
                .expect("Failed to seek after truncation");
            write!(file, ",\n{formatted_entry}\n]\n").expect("Failed to append log entry");
        }
        _ => panic!("Log file is not a JSON array"),
    }
}

fn find_last_non_whitespace(file: &mut File) -> Option<u64> {
    let len = file.metadata().ok()?.len();

    for pos in (0..len).rev() {
        file.seek(SeekFrom::Start(pos)).ok()?;

        let mut byte: [u8; 1] = [0; 1];
        file.read_exact(&mut byte).ok()?;

        if !byte[0].is_ascii_whitespace() {
            return Some(pos);
        }
    }

    None
}

fn indent_json_block(json: &str) -> String {
    json.lines()
        .map(|line| format!("  {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}
