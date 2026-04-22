# ZYO: Autonomous Agentic OS Scheduler & Ring-0 Firewall

**Zyo** is a sovereign, self-healing Linux CPU scheduler and autonomous network firewall. It replaces static, human-coded kernel policies with a dynamic AI agent that continuously analyzes running applications, hardware stress, and network traffic. It synthesizes custom eBPF scheduling policies and XDP packet-filtering maps on the fly to optimize for throughput, latency, and real-time threat defense.

Built with an absolute focus on **privacy-by-design, data sovereignty, and offline-first computing**, the entire architecture—from the Ring-0 telemetry bridge to the 7-billion parameter code-synthesizing LLM—runs locally on bare-metal hardware. Zero telemetry leaves the motherboard. It is a completely self-reliant intelligence grid.

---

## 🌎 The Problem & The Market Opportunity

Modern cloud infrastructure and operating systems are bottlenecked by legacy design:

1. **Static Kernels:** Linux relies on generic, one-size-fits-all scheduling heuristics designed decades ago. They cannot rapidly adapt to dynamic AI, High-Frequency Trading (HFT), or sudden swarm workloads.
2. **Reactive Security:** Traditional firewalls (like `iptables` or `ufw`) process malicious packets *after* they enter the operating system's network stack, consuming massive CPU cycles and allowing datacenters to be overwhelmed by DDoS attacks.
3. **Loss of Sovereignty:** Relying on external cloud scrubbers (like Cloudflare or AWS Shield) means sacrificing data sovereignty, exposing telemetry to third parties, and paying exorbitant enterprise SaaS fees.

**The Zyo Solution:** An Agentic OS that manages itself. By decoupling the control plane (AI reasoning) from the data plane (eBPF execution), Zyo brings autonomous, mathematically optimized infrastructure directly to the edge.

---

## 🚀 Architectural Breakthroughs (The "PhD-Level" Tech)

Zyo was engineered to solve the major bottlenecks of AI-kernel integration, successfully bridging Deep Learning, Rust, and eBPF:

1. **Multi-Dimensional Telemetry (The 3D Tensor):** The AI doesn't just "feel" CPU latency. It processes a mathematically normalized 3D tensor of `[Latency, Interrupts, Memory Pressure]`. This allows the agent to schedule tasks based on the actual physical hardware stress of the entire motherboard.
2. **XDP Network Shield (Ring-0 Packet Vaporizer):** Zyo utilizes eBPF `XDP` (eXpress Data Path) to attach directly to the Network Interface Card (NIC). When the AI detects a DDoS-level bandwidth anomaly (>1MB/s), it dynamically translates the attacker's IP to hex and injects it into a Ring-0 drop list, vaporizing malicious packets at the driver layer before they reach Linux.
3. **The "Time Dilation" Fix (Asynchronous Orchestration):** Heavy AI reasoning happens in user-space via a multi-threaded Rust `Tokio` runtime. When CPU latency breaches safety thresholds, an instant "Panic Button" hot-swaps a hardcoded SafeMode scheduler in nanoseconds, preventing system freeze while the LLM synthesizes code in the background.
4. **Automated Program Repair (Zero-Crash Guarantee):** If the LLM hallucinates flawed C code, the strict Linux eBPF verifier rejects it. Zyo captures the Clang compiler errors, feeds them back into the LLM's context window, and executes a 3-strike circuit breaker loop to autonomously repair the syntax before reverting to failsafes.
5. **Continuous Telemetry (The Memory Bank):** Every 500ms, the system logs its exact hardware state and AI reward/punishment actions `(timestamp, latency_us, interrupts, mem_pressure, safe_weight, reward)` to a CSV memory bank for offline dreaming and training of future neural networks.
6. **Anti-Thrashing Momentum (Exponential Moving Average):** The Reinforcement Learning agent features a `0.2` EMA damping factor and a `10000.0` hard-safety floor. This mathematical shock absorber prevents the AI from violently swinging CPU weights or completely starving the kernel, protecting the L3 cache from micro-stuttering.
7. **Lockless Memory Bridge:** Zyo utilizes `BPF_MAP_TYPE_PERCPU_ARRAY`. The user-space Rust orchestrator converts PyTorch weight calculations into Little-Endian hex bytes and injects them directly into Ring-0 memory, allowing each CPU core to read its own isolated memory address without locking up the kernel.

---

## 🧠 The Sovereign Grid Architecture

The system operates across three distinct layers:

### 1. The Data Plane (eBPF / C in Ring-0)
* **CPU Scheduler (`zyo_sched.bpf.c`):** Injected directly into the Linux kernel via the `sched_ext` framework (`scx`). It exposes eBPF PERCPU maps to allow the user-space AI to manipulate scheduling weights and time-slices.
* **XDP Firewall (`zyo_shield.bpf.c`):** A high-speed packet filtering engine that uses `BPF_MAP_TYPE_HASH` to drop traffic natively on the network card.

### 2. The Fast Brain (PyTorch C++ Engine via Rust)
A Reinforcement Learning neural network operating in continuous 500ms loops. It reads raw hardware telemetry (`/proc/stat`, `/proc/meminfo`, and RX bytes). Using a strict reward function based on decreasing tail-latency, it calculates and applies the optimal CPU time-slice allocation to maximize throughput natively in user-space.

### 3. The Slow Brain (Local LLM Watchdog)
A localized LLM (Qwen 2.5 7B via Ollama). If the Fast Brain encounters a chaotic workload it cannot stabilize, the Slow Brain is triggered. It acts as the "Architect," reading the failing eBPF C code, synthesizing a new scheduling logic string, invoking Clang to recompile, and seamlessly hot-swapping the new maps into Ring-0 without dropping active connections.

---

## 📊 Benchmarks & Proof of Work

During live simulation testing, the ZYO grid successfully demonstrated autonomous self-defense:
* **The Attack:** A simulated DDoS payload flooded the host network interface at **8.4 MB/s** (a massive spike in `RX_BYTES` and hardware interrupts).
* **The Detection:** Within milliseconds, the Rust orchestrator detected the 3D tensor anomaly, crossing the 1,000,000 B/s threshold. 
* **The Killshot:** The system autonomously flagged the `[CRITICAL ANOMALY]`, extracted the attacking IP, and executed a Ring-0 map injection. The terminal confirmed `[🛡️ VAPORIZED]`, instantly dropping the connection at the XDP layer and returning the host to stable CPU latency.

---

## ⚙️ System Requirements

To deploy this architecture, you require a highly specific Linux environment.

**Kernel & Toolchain:**
* Linux Kernel **6.12+** (Compiled with `CONFIG_SCHED_CLASS_EXT=y`, `CONFIG_BPF=y`, `CONFIG_DEBUG_INFO_BTF=y`)
* Clang & LLVM (`>=16`)
* `bpftool` and `libbpf-dev`
* **Rust Toolchain** (Cargo, Rustc)

**AI Stack:**
* **Ollama** (Running locally with `qwen2.5-coder:7b` pulled)
* **LibTorch CPU:** PyTorch C++ Engine (v2.2.0)
* Python 3.10+ (Only required for initial model export and swarm load testing)

---

## 🛠️ Step-by-Step: Ignite the Grid

### 1. Prepare the AI Server (Ollama)
Ensure Ollama is running and accessible. Download the localized code-synthesis model:
```bash
ollama pull qwen2.5-coder:7b
```
*(Note: If running Linux via WSL2 and Ollama on your Windows host, you must set the Windows environment variable `OLLAMA_HOST="0.0.0.0"` before running Ollama so it accepts the Linux bridge connection from `10.0.2.2`).*

### 2. Export the V3 PyTorch Fast Brain
Rust requires a C++ compatible TorchScript file. Compile the baseline Python neural network:
```bash
pip3 install torch --index-url [https://download.pytorch.org/whl/cpu](https://download.pytorch.org/whl/cpu) --break-system-packages
python3 export_v3_brain.py
mv zyo_brain_v3.pt zyo_agent_rs/
```

### 3. Download the C++ PyTorch Engine
The Rust `tch` crate requires the native LibTorch C++ backend to execute the tensor math in user-space:
```bash
cd zyo_agent_rs
wget [https://download.pytorch.org/libtorch/cpu/libtorch-cxx11-abi-shared-with-deps-2.2.0%2Bcpu.zip](https://download.pytorch.org/libtorch/cpu/libtorch-cxx11-abi-shared-with-deps-2.2.0%2Bcpu.zip)
unzip libtorch-cxx11-abi-shared-with-deps-2.2.0+cpu.zip
```

### 4. Compile the Ring-0 Core & Shield
Clear any existing maps, compile the baseline CPU scheduler, and cross-compile the XDP network firewall:
```bash
sudo rm -rf /sys/fs/bpf/zyo_*

# Compile CPU Scheduler
sudo clang -O2 -g -target bpf -I . -c zyo_sched.bpf.c -o zyo_sched.bpf.o
sudo bpftool struct_ops register zyo_sched.bpf.o /sys/fs/bpf/zyo_sched

# Compile XDP Network Shield
sudo clang -O2 -g -target bpf -I/usr/include/x86_64-linux-gnu -c zyo_shield.bpf.c -o zyo_shield.bpf.o
```

---

## ⚡ Execution & Live Testing

To observe the Agentic OS in action, you need three terminal windows to ignite the grid, trigger a CPU swarm, and simulate a DDoS attack.

**Terminal 1: Ignite the Agentic OS**
Bind the C++ library path and launch the Rust Dual-Brain orchestrator:
```bash
cd zyo_agent_rs
export LIBTORCH_BYPASS_VERSION_CHECK=1
export LIBTORCH=$(pwd)/libtorch
cargo build
sudo LD_LIBRARY_PATH=${LIBTORCH}/lib ./target/debug/zyo_agent_rs
```

**Terminal 2: Trigger the CPU Swarm Workload**
Simulate a massive, multi-core traffic flood to stress the CPU queue and force the Fast Brain to react:
```bash
python3 safe_load.py
```

**Terminal 3: Simulate the DDoS Attack**
Force a massive bandwidth payload into your network card to trigger the XDP firewall (bypassing SSL checks for testing):
```bash
wget --no-check-certificate -O /dev/null [https://dl.google.com/go/go1.22.2.linux-amd64.tar.gz](https://dl.google.com/go/go1.22.2.linux-amd64.tar.gz)
```

---

## 🎯 Expected Output & Behavior

* **Telemetry Stream:** You will watch the RL Agent print its real-time 3D telemetry (Lat, Intr, Mem, Net), dynamically tuning the `W:` (Weight) map based on the microsecond hardware stress.
* **Network Lockdown:** When the payload hits your network interface, bandwidth will spike. Zyo will instantly output `[🚨] XDP AI ENGAGED` and `[🛡️ ] VAPORIZED`, dropping the IP address directly in Ring-0.
* **LLM CPU Rescue:** When the Swarm workload pushes the CPU latency beyond the `800.0 µs` threshold, the system will trigger a **RED ALERT**. The Rust orchestrator will load the SafeMode scheduler, package the C code, and ping Qwen asynchronously. The terminal will output `[*] HOT-SWAP SUCCESSFUL`, surgically replacing its own brain without dropping host connectivity.

---

## 🔧 Troubleshooting & Edge Cases

* **`libtorch_cpu.so: cannot open shared object file`**: Sudo strips environment variables. You must pass the library path directly in the execution command: `sudo LD_LIBRARY_PATH=$(pwd)/libtorch/lib ./target/debug/zyo_agent_rs`.
* **Connection Refused on Port 11434**: If Rust cannot reach the LLM, you are hitting a VM/WSL2 boundary. Find your host IP from Linux by running `ip route | grep default | awk '{print $3}'`. Update the API URL in `src/main.rs` with this IP.
* **Rust `tch` crate compiler errors**: Ensure you have exported `export LIBTORCH_BYPASS_VERSION_CHECK=1` to allow `tch` v0.15.0 to link with PyTorch 2.2.0.
* **Cannot find device "eth0"**: Your network interface might have a different name (e.g., `enp0s3` or `wlan0`). Run `ip -br l` to find your active interface, and update the string references inside `src/main.rs`.

---

## 🗺️ Roadmap & Future Vision

* **Phase 1 (Completed):** Core multi-brain architecture, eBPF CPU scheduling, and XDP autonomous defense prototype.
* **Phase 2:** Enterprise dashboard UI, distributed cluster orchestration, and migrating from generalized LLMs to a deeply fine-tuned Sovereign Kernel Code Synthesis model.
* **Phase 3:** Full commercial deployment for local infrastructure providers and independent datacenters prioritizing data sovereignty.

---

## 👤 The Architect
**(TheSNMC)**
*Built for a Sovereign Future.*