

use anyhow::{Result, bail};
use reqwest::Client;
use serde_json::json;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tch::{Tensor, CModule};

use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use std::net::Ipv4Addr; // NEW: Needed to parse IP addresses for the XDP Shield

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
        // LOAD THE NEW V3 BRAIN
        let model = tch::CModule::load("zyo_brain_v3.pt")
            .expect("[!] FATAL: Could not load zyo_brain_v3.pt. Is it in the directory?");
            
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

    // Accepts 3 Dimensions with Tensor Normalization
    fn predict_action(&self, latency: f64, interrupts: f64, mem_pressure: f64) -> f64 {
        // Normalize the inputs so the Neural Network doesn't overload
        let norm_lat = (latency / 1000.0) as f32;
        let norm_intr = (interrupts / 10000.0) as f32; // Scales down safely
        let norm_mem = (mem_pressure / 100.0) as f32;  // Scales 13% down to 0.13

        let state_tensor = Tensor::from_slice(&[norm_lat, norm_intr, norm_mem]).view((1, 3));
        
        let action_tensor = self.model.forward_ts(&[state_tensor]).unwrap();
        f32::try_from(action_tensor).unwrap() as f64
    }

    fn calculate_damped_weight(&mut self, raw_action: f64, reward: f64) -> u64 {
        let target_weight = ((raw_action + (reward * 0.01)) * 1_500_000.0) + 1_000_000.0;
        self.last_weight = (self.damping_factor * target_weight) 
                         + ((1.0 - self.damping_factor) * self.last_weight);
                         
        // THE HARD FLOOR: Never let the AI starve the CPU completely
        if self.last_weight < 10000.0 {
            self.last_weight = 10000.0;
        }
        
        self.last_weight as u64
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n=======================================================");
    println!("[*] ZYO AGENTIC OS ONLINE (v1.0 - The Sovereign Grid)");
    println!("=======================================================\n");
    
    // --- ARM THE XDP SHIELD ---
    init_network_shield();
    
    // --- CSV MEMORY BANK (Upgraded Headers) ---
    let csv_path = "zyo_training_data_v3.csv";
    let file_exists = std::path::Path::new(csv_path).exists();
    
    let mut csv_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(csv_path)?;

    if !file_exists {
        writeln!(csv_file, "timestamp,latency_us,interrupts,mem_pressure_pct,safe_weight,reward")?;
        println!("[*] Initialized new 3D telemetry bank: {}", csv_path);
    } else {
        println!("[*] Appending to existing 3D telemetry bank: {}", csv_path);
    }
    
    let state = Arc::new(Mutex::new(SystemState::Running));
    let mut brain = FastBrain::new();
    let client = Client::new();
    
    let mut latency_violations = 0;
    let violation_threshold = 800.0; 
    let max_violations = 4;
    
    let mut last_switches = read_os_metric("ctxt").unwrap_or(0);
    let mut last_interrupts = read_os_metric("intr").unwrap_or(0);
    let mut last_rx_bytes = read_rx_bytes(); // NEW: Network Bandwidth Telemetry

    loop {
        let current_state = *state.lock().await;
        if current_state == SystemState::SafeMode {
            println!("[!] System in Safe Mode. Waiting for LLM background synthesis...");
            sleep(Duration::from_secs(2)).await;
            continue;
        }

        // 1. Gather Telemetry (CPU, Mem, Network)
        let current_switches = read_os_metric("ctxt").unwrap_or(last_switches);
        let diff_switches = current_switches.saturating_sub(last_switches);
        last_switches = current_switches;

        let current_intr = read_os_metric("intr").unwrap_or(last_interrupts);
        let diff_intr = current_intr.saturating_sub(last_interrupts);
        last_interrupts = current_intr;
        
        // Read Network Delta
        let current_rx_bytes = read_rx_bytes();
        let diff_rx = current_rx_bytes.saturating_sub(last_rx_bytes);
        last_rx_bytes = current_rx_bytes;

        let mem_pressure = read_memory_pressure();

        let current_latency = if diff_switches == 0 {
            100.0 
        } else {
            50000.0 / (diff_switches as f64)
        };
        
        let reward = brain.calculate_reward(current_latency);
        
        // 2. Feed the Normalized 3D Tensor to the AI
        let raw_action = brain.predict_action(current_latency, diff_intr as f64, mem_pressure);
        let safe_weight = brain.calculate_damped_weight(raw_action, reward);
        
        println!(" -> [ZYO Core] W: {} | Lat: {:.2}µs | Intr: {} | Mem: {:.1}% | Net: {} B/s", 
                 safe_weight, current_latency, diff_intr, mem_pressure, diff_rx);

        // 3. Log to V3 Memory Bank
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
        if let Err(e) = writeln!(csv_file, "{:.4},{:.2},{},{:.2},{},{:.2}", timestamp, current_latency, diff_intr, mem_pressure, safe_weight, reward) {
            eprintln!("[!] WARNING: Failed to write to telemetry bank: {}", e);
        }

        // 4. Update the Kernel Map for CPU Scheduling
        inject_weight_to_kernel(safe_weight);

        // --- 5. THE XDP DDOS AI SENSOR ---
        // If bandwidth spikes over 1MB/sec instantly, the AI triggers a lockdown
        if diff_rx > 1_000_000 {
            println!("\n[🚨] CRITICAL ANOMALY: Massive Bandwidth Spike Detected ({} B/s)", diff_rx);
            println!("[🚨] XDP AI ENGAGED: Extracting Malicious IP...");
            // In a full production build, you'd extract the actual IP from the packet. 
            // For this architecture demo, we block a simulated attacker IP:
            block_ip("185.154.12.5"); 
            println!("-------------------------------------------------------\n");
        }

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

// --- NEW HELPER FUNCTIONS FOR XDP NETWORK SHIELD ---

fn init_network_shield() {
    println!("[*] Initializing XDP Network Shield on enp0s3...");
    // Clear old attachments, then mount the eBPF object directly to the network card using xdpgeneric
    Command::new("sudo").args(["ip", "link", "set", "dev", "enp0s3", "xdpgeneric", "off"]).output().ok();
    let out = Command::new("sudo").args(["ip", "link", "set", "dev", "enp0s3", "xdpgeneric", "obj", "zyo_shield.bpf.o", "sec", "xdp"]).output().unwrap();
    
    if out.status.success() {
        println!("[*] 🛡️  XDP SHIELD ARMED! Ring-0 Packet Vaporizer Online.\n");
    } else {
        println!("[!] Failed to attach XDP shield: {}\n", String::from_utf8_lossy(&out.stderr));
    }
}

fn block_ip(ip_str: &str) {
    if let Ok(ip) = ip_str.parse::<Ipv4Addr>() {
        let octets = ip.octets();
        // Convert the IP address into a 4-byte hex array for bpftool
        let key_hex = format!("{:02x} {:02x} {:02x} {:02x}", octets[0], octets[1], octets[2], octets[3]);
        
        // Inject the hex string into the Ring-0 Drop List memory map!
        let cmd = format!("bpftool map update name drop_list key hex {} value hex 01", key_hex);
        Command::new("sudo").args(["sh", "-c", &cmd]).output().ok();
        
        println!("[🛡️ ] VAPORIZED: IP {} successfully blacklisted in Ring-0 Memory.", ip_str);
    }
}

fn read_rx_bytes() -> u64 {
    if let Ok(file) = std::fs::read_to_string("/sys/class/net/enp0s3/statistics/rx_bytes") {
        return file.trim().parse().unwrap_or(0);
    }
    0
}

// --- STANDARD HELPER FUNCTIONS ---

async fn trigger_llm_repair(client: Client, latency: f64) -> Result<()> {
    println!("[*] Background Thread: Contacting Local Qwen 2.5...");
    let baseline_code = std::fs::read_to_string("zyo_sched.bpf.c").unwrap_or_else(|_| "// Error reading baseline code".to_string());
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
            .json(&json!({"model": "qwen2.5-coder:7b", "prompt": &current_prompt, "stream": false}))
            .send().await?;
            
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
        current_prompt = format!(
            "Your previous attempt to fix the eBPF code failed to compile. \nOriginal Code:\n{}\nFailed Code:\n{}\nClang Error:\n{}\nFix the exact error in your failed code. Output ONLY the valid C code.", 
            baseline_code, new_code, error_log
        );
        retry_count += 1;
    }
    bail!("LLM failed the verifier 3 times. Circuit breaker tripped.");
}

fn load_safe_scheduler() {
    Command::new("sudo").args(["bpftool", "struct_ops", "unregister", "name", "zyo_ops"]).output().ok();
    Command::new("sudo").args(["bpftool", "struct_ops", "register", "zyo_safe_fallback.bpf.o"]).output().ok();
}

fn extract_c_code(raw: &str) -> String {
    if raw.contains("```c") { raw.split("```c").nth(1).unwrap_or("").split("```").next().unwrap_or("").trim().to_string() } else { raw.to_string() }
}

// Universal /proc/stat metric parser
fn read_os_metric(target: &str) -> Option<u64> {
    if let Ok(file) = std::fs::File::open("/proc/stat") {
        let reader = std::io::BufReader::new(file);
        for line in std::io::BufRead::lines(reader).map_while(Result::ok) {
            if line.starts_with(target) {
                if let Some(val) = line.split_whitespace().nth(1) {
                    return val.parse().ok();
                }
            }
        }
    }
    None
}

// Memory Pressure Parser
fn read_memory_pressure() -> f64 {
    let mut mem_total = 1.0;
    let mut mem_available = 1.0;
    
    if let Ok(file) = std::fs::File::open("/proc/meminfo") {
        let reader = std::io::BufReader::new(file);
        for line in std::io::BufRead::lines(reader).map_while(Result::ok) {
            if line.starts_with("MemTotal:") {
                mem_total = line.split_whitespace().nth(1).unwrap_or("1").parse().unwrap_or(1.0);
            } else if line.starts_with("MemAvailable:") {
                mem_available = line.split_whitespace().nth(1).unwrap_or("1").parse().unwrap_or(1.0);
            }
        }
    }
    // Calculate percentage of memory USED (Pressure)
    ((mem_total - mem_available) / mem_total) * 100.0
}

fn inject_weight_to_kernel(weight: u64) {
    let b = weight.to_le_bytes();
    let val_hex = format!("{:#04x} {:#04x} {:#04x} {:#04x} {:#04x} {:#04x} {:#04x} {:#04x}", 
                          b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]);
    let cmd = format!("bpftool map update name zyo_tuning_map key 0x00 0x00 0x00 0x00 value {}", val_hex);
    Command::new("sudo").args(["sh", "-c", &cmd]).output().ok();
}