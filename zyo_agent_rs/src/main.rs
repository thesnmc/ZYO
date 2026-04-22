/*
 * ZYO Agentic OS - Core Scheduler Engine
 * Copyright (C) 2026
 * * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published
 * by the Free Software Foundation, version 3.
 */

use anyhow::{Result, bail};
use reqwest::Client;
use serde_json::json;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tch::{Tensor, CModule};

// --- SYSTEM STATES ---
#[derive(Debug, PartialEq, Copy, Clone)]
enum SystemState {
    Running,
    SafeMode, 
}

// --- RL FAST BRAIN (Neural Network + Momentum) ---
struct FastBrain {
    model: CModule, 
    last_weight: f64,
    damping_factor: f64,
    previous_latency: f64, 
}

impl FastBrain {
    fn new() -> Self {
        let model = tch::CModule::load("zyo_brain.pt")
            .expect("[!] FATAL: Could not load zyo_brain.pt. Is it in the directory?");
            
        Self {
            model,
            last_weight: 1_000_000.0,
            damping_factor: 0.2, 
            previous_latency: 100.0, 
        }
    }

    fn calculate_reward(&mut self, current_latency: f64) -> f64 {
        let latency_improvement = self.previous_latency - current_latency;
        let reward = latency_improvement * 0.1; 
        self.previous_latency = current_latency;
        reward
    }

    fn predict_action(&self, current_latency: f64) -> f64 {
        let state_tensor = Tensor::from_slice(&[current_latency as f32]).view((1, 1));
        let action_tensor = self.model.forward_ts(&[state_tensor]).unwrap();
        f32::try_from(action_tensor).unwrap() as f64
    }

    fn calculate_damped_weight(&mut self, raw_action: f64, reward: f64) -> u64 {
        let target_weight = ((raw_action + (reward * 0.01)) * 1_500_000.0) + 1_000_000.0;
        self.last_weight = (self.damping_factor * target_weight) 
                         + ((1.0 - self.damping_factor) * self.last_weight);
        self.last_weight as u64
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("[*] ZYO Agentic OS Online (Rust/Tokio Edition)");
    
    let state = Arc::new(Mutex::new(SystemState::Running));
    let mut brain = FastBrain::new();
    let client = Client::new();
    
    let mut latency_violations = 0;
    let violation_threshold = 800.0; 
    let max_violations = 4;
    
    let mut last_switches = read_os_switches();

    loop {
        let current_state = *state.lock().await;
        if current_state == SystemState::SafeMode {
            println!("[!] System in Safe Mode. Waiting for LLM background synthesis...");
            sleep(Duration::from_secs(2)).await;
            continue;
        }

        let current_switches = read_os_switches();
        let diff = current_switches.saturating_sub(last_switches);
        last_switches = current_switches;

        let current_latency = if diff == 0 {
            100.0 
        } else {
            50000.0 / (diff as f64)
        };
        
        let reward = brain.calculate_reward(current_latency);
        let raw_action = brain.predict_action(current_latency);
        let safe_weight = brain.calculate_damped_weight(raw_action, reward);
        
        println!(" -> [RL FAST BRAIN] Weight: {} | Latency: {:.2} µs | Reward: {:.2}", safe_weight, current_latency, reward);

        // --- THE MEMORY BRIDGE ---
        // Inject the PyTorch weight directly into the live Linux kernel
        inject_weight_to_kernel(safe_weight);

        if current_latency > violation_threshold {
            latency_violations += 1;
        } else {
            latency_violations = 0;
        }

        if latency_violations >= max_violations {
            println!("\n[!!!] RED ALERT: RL Agent lost control.");
            
            *state.lock().await = SystemState::SafeMode;
            println!("[*] PANIC BUTTON HIT: Loading hardcoded SafeMode Scheduler instantly.");
            load_safe_scheduler();

            let state_clone = Arc::clone(&state);
            let client_clone = client.clone();
            
            tokio::spawn(async move {
                if let Err(e) = trigger_llm_repair(client_clone, current_latency).await {
                    println!("[!] FATAL: LLM Repair failed completely. System locked in SafeMode. Error: {}", e);
                } else {
                    println!("[*] Repair successful. Handing control back to Fast Brain.");
                    *state_clone.lock().await = SystemState::Running;
                }
            });

            latency_violations = 0;
        }

        sleep(Duration::from_millis(500)).await;
    }
}

async fn trigger_llm_repair(client: Client, latency: f64) -> Result<()> {
    println!("[*] Background Thread: Contacting Local Qwen 2.5...");
    
    let baseline_code = std::fs::read_to_string("zyo_sched.bpf.c")
        .unwrap_or_else(|_| "// Error reading baseline code".to_string());
    
    let mut retry_count = 0;
    let max_retries = 3; 
    
    let mut current_prompt = format!(
        "The Linux CPU queue is stalling at {} microseconds. \
        Rewrite the following eBPF C code. Modify the default 'slice' value inside zyo_enqueue to prioritize throughput (try setting it to 500000). \
        Output ONLY the valid C code. Do not include markdown formatting or explanations.\n\nCurrent Code:\n{}", 
        latency, baseline_code
    );

    while retry_count < max_retries {
        let res = client.post("http://10.0.2.2:11434/api/generate")
            .json(&json!({
                "model": "qwen2.5-coder:7b",
                "prompt": &current_prompt,
                "stream": false
            }))
            .send()
            .await?;
            
        let json_resp: serde_json::Value = res.json().await?;
        let new_code = extract_c_code(json_resp["response"].as_str().unwrap_or(""));
        
        std::fs::write("zyo_sched.bpf.c", &new_code)?;
        
        Command::new("sudo").args(["bpftool", "struct_ops", "unregister", "name", "zyo_ops"]).output().ok();
        Command::new("sudo").args(["sh", "-c", "rm -rf /sys/fs/bpf/zyo_*"]).output().ok();

        let compile_result = Command::new("sudo")
            .args(["clang", "-O2", "-g", "-target", "bpf", "-I", ".", "-c", "zyo_sched.bpf.c", "-o", "zyo_sched.bpf.o"])
            .output()?;

        if compile_result.status.success() {
            let load_result = Command::new("sudo")
                .args(["bpftool", "struct_ops", "register", "zyo_sched.bpf.o", "/sys/fs/bpf/zyo_sched"])
                .output()?;
                
            if load_result.status.success() {
                println!("\n[*] HOT-SWAP SUCCESSFUL on attempt {}.", retry_count + 1);
                return Ok(());
            }
        }
        
        let error_log = String::from_utf8_lossy(&compile_result.stderr);
        println!("\n--- WHAT QWEN WROTE ---\n{}\n-----------------------", new_code);
        println!("--- COMPILER ERROR ---\n{}\n-----------------------\n", error_log);
        println!("[!] Verifier rejected the code. Retry {}/{}", retry_count + 1, max_retries);
        
        current_prompt = format!(
            "Your previous attempt to fix the eBPF code failed to compile. \n\n\
            Original Working Code:\n{}\n\n\
            Your Failed Code:\n{}\n\n\
            Clang Compiler Error:\n{}\n\n\
            Fix the exact error in your failed code. Output ONLY the complete, valid C code without markdown or explanations.", 
            baseline_code, new_code, error_log
        );
        
        retry_count += 1;
    }

    bail!("LLM failed the verifier 3 times. Circuit breaker tripped.");
}

// --- HELPER FUNCTIONS ---
fn load_safe_scheduler() {
    Command::new("sudo").args(["bpftool", "struct_ops", "unregister", "name", "zyo_ops"]).output().ok();
    Command::new("sudo").args(["bpftool", "struct_ops", "register", "zyo_safe_fallback.bpf.o"]).output().ok();
}

fn extract_c_code(raw: &str) -> String {
    if raw.contains("```c") {
        raw.split("```c").nth(1).unwrap_or("").split("```").next().unwrap_or("").trim().to_string()
    } else {
        raw.to_string()
    }
}

fn read_os_switches() -> u64 {
    if let Ok(file) = std::fs::File::open("/proc/stat") {
        let reader = std::io::BufReader::new(file);
        for line in std::io::BufRead::lines(reader).map_while(Result::ok) {
            if line.starts_with("ctxt") {
                if let Some(val) = line.split_whitespace().nth(1) {
                    return val.parse().unwrap_or(0);
                }
            }
        }
    }
    0
}

// --- NEW: THE EBPF MEMORY BRIDGE ---
fn inject_weight_to_kernel(weight: u64) {
    // 1. Break the u64 into 8 raw bytes (Little-Endian for x86 CPUs)
    let b = weight.to_le_bytes();
    
    // 2. Format them into hex so bpftool can read them
    let val_hex = format!("{:#04x} {:#04x} {:#04x} {:#04x} {:#04x} {:#04x} {:#04x} {:#04x}", 
                          b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]);
                          
    // Key 0 corresponds to the single element in our eBPF PERCPU_ARRAY map
    let key_hex = "0x00 0x00 0x00 0x00";
    
    // 3. Command the kernel to overwrite the map memory
    let cmd = format!("bpftool map update name zyo_tuning_map key {} value {}", key_hex, val_hex);
    Command::new("sudo").args(["sh", "-c", &cmd]).output().ok();
}