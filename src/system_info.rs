use std::collections::VecDeque;
use std::process::Command;

use sysinfo::System;

#[derive(Clone, Copy, Debug)]
pub struct SystemSample {
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub gpu_usage: f32,
    pub vram_usage: u64,
}

#[derive(Debug)]
pub struct MinuteRingBuffer {
    capacity: usize,
    samples: VecDeque<SystemSample>,
    cpu_sum: f32,
    memory_sum: u64,
    gpu_sum: f32,
    vram_sum: u64,
}

impl MinuteRingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            samples: VecDeque::with_capacity(capacity),
            cpu_sum: 0.0,
            memory_sum: 0,
            gpu_sum: 0.0,
            vram_sum: 0,
        }
    }

    pub fn push(&mut self, sample: SystemSample) {
        if self.samples.len() == self.capacity {
            if let Some(oldest) = self.samples.pop_front() {
                self.cpu_sum -= oldest.cpu_usage;
                self.memory_sum -= oldest.memory_usage;
                self.gpu_sum -= oldest.gpu_usage;
                self.vram_sum -= oldest.vram_usage;
            }
        }

        self.cpu_sum += sample.cpu_usage;
        self.memory_sum += sample.memory_usage;
        self.gpu_sum += sample.gpu_usage;
        self.vram_sum += sample.vram_usage;
        self.samples.push_back(sample);
    }

    pub fn is_full(&self) -> bool {
        self.samples.len() == self.capacity
    }

    pub fn average(&self) -> Option<SystemSample> {
        if !self.is_full() {
            return None;
        }

        let len = self.samples.len() as f32;

        Some(SystemSample {
            cpu_usage: self.cpu_sum / len,
            memory_usage: (self.memory_sum as f64 / len as f64).round() as u64,
            gpu_usage: self.gpu_sum / len,
            vram_usage: (self.vram_sum as f64 / len as f64).round() as u64,
        })
    }
}

pub struct SysInfo{
    pub sys: System,
}

impl SysInfo{

    pub fn new() -> Self{
        let mut sys = System::new_all();
        sys.refresh_all();

        Self {
            sys,
        }
    }

    pub fn cpu_usage(&self) -> String{
        
        self.sys.global_cpu_usage().to_string()
    }

    pub fn memory_usage(&self) -> String{
        
        self.sys.used_memory().to_string()
    }

    pub fn gpu_usage(&self) -> String{
        let gpu_usage = Command::new("nvidia-smi")
            .arg("--query-gpu=utilization.gpu")
            .arg("--format=csv,noheader,nounits")
            .output()
            .expect("failed to run");

        String::from_utf8_lossy(&gpu_usage.stdout)
            .trim()
            .to_string()
        
    }

    pub fn vram_usage(&self) -> String{
        let vram_usage = Command::new("nvidia-smi")
            .arg("--query-gpu=memory.used")
            .arg("--format=csv,noheader,nounits")
            .output()
            .expect("failed to run");

        String::from_utf8_lossy(&vram_usage.stdout)
            .trim()
            .to_string()
    }

    
}

pub fn collect_system_sample() -> SystemSample {
    let mut sys = System::new_all();
    sys.refresh_all();
    read_system_sample(&mut sys)
}

fn read_system_sample(sys: &mut System) -> SystemSample {
    SystemSample {
        cpu_usage: sys.global_cpu_usage(),
        memory_usage: sys.used_memory(),
        gpu_usage: read_nvidia_value("utilization.gpu") as f32,
        vram_usage: read_nvidia_value("memory.used"),
    }
}

fn read_nvidia_value(query: &str) -> u64 {
    let output = Command::new("nvidia-smi")
        .arg(format!("--query-gpu={query}"))
        .arg("--format=csv,noheader,nounits")
        .output()
        .expect("failed to run");

    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .expect("nvidia-smi returned a non-numeric value")
}
