# ZYO: Autonomous Agentic OS Scheduler

**Zyo** is a sovereign, self-healing Linux CPU scheduler. It replaces static, human-coded kernel policies with an autonomous, dual-brain AI agent that continuously analyzes running applications and synthesizes custom eBPF scheduling policies on the fly to optimize for throughput and latency.

Built with an absolute focus on **privacy-by-design, data sovereignty, and offline-first computing**, the entire architecture—from the Ring-0 telemetry bridge to the 7-billion parameter code-synthesizing LLM—runs locally on host hardware. Zero telemetry leaves the motherboard. It is a completely self-reliant intelligence.

---
## 🚀 Architectural Breakthroughs (The "PhD-Level" Features)
Zyo was engineered to solve the four major bottlenecks of AI-kernel integration:

1. **The "Time Dilation" Fix (Asynchronous Orchestration):** Heavy AI reasoning happens in user-space via a multi-threaded Rust `Tokio` runtime. When CPU latency breaches safety thresholds, an instant "Panic Button" hot-swaps a hardcoded SafeMode scheduler in nanoseconds, preventing system freezing while the LLM thinks in the background.
2. **Automated Program Repair (Zero-Crash Guarantee):** If the LLM hallucinates bad C code, the Linux eBPF verifier rejects it. Zyo captures the Clang compiler errors, feeds them back into the LLM's context window, and executes a 3-strike circuit breaker loop to autonomously repair the syntax before giving up.
3. **Anti-Thrashing Momentum (Exponential Moving Average):** The Reinforcement Learning agent features a `0.2` EMA damping factor. This mathematical shock absorber prevents the AI from violently swinging CPU weights, protecting the L3 cache from micro-stuttering.
4. **Lockless Memory Bridge:** Zyo utilizes `BPF_MAP_TYPE_PERCPU_ARRAY`. The user-space Rust orchestrator converts PyTorch weight calculations into Little-Endian hex bytes and injects them directly into Ring-0 memory, allowing each CPU core to read its own isolated memory address without lock contention.

---
## 🧠 The Dual-Brain Architecture

### 1. The Data Plane (eBPF / C)
A dynamic CPU scheduler injected directly into the Linux kernel via the `sched_ext` framework (`scx`). It uses eBPF PERCPU array maps to allow the user-space AI to manipulate scheduling weights and time-slices in real-time.

### 2. The Fast Brain (PyTorch C++ Engine via Rust)
A Reinforcement Learning neural network operating in continuous 500ms loops. It reads raw hardware context switches from `/proc/stat`. Using a strict reward function based on decreasing tail-latency, it calculates and applies the optimal CPU time-slice allocation to maximize throughput.

### 3. The Slow Brain (Local LLM Watchdog)
A localized LLM (Qwen 2.5 7B via Ollama). If the Fast Brain encounters a workload it cannot stabilize, the Slow Brain is triggered. It reads the failing eBPF C code, synthesizes a new scheduling logic string, invokes Clang to recompile, and seamlessly hot-swaps the new maps into Ring-0 without dropping SSH connections.

---
## ⚙️ System Requirements

To run this architecture, you need a highly specific Linux environment.

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
## 🛠️ Step-by-Step: Try It Yourself

### 1. Prepare the AI Server (Ollama)
Ensure Ollama is running and accessible. Download the C-coding model:
```bash
ollama pull qwen2.5-coder:7b
```
(Note: If running Linux via WSL2 and Ollama on your Windows host, you must set the environment variable OLLAMA_HOST="0.0.0.0" before running Ollama so it accepts the Linux bridge connection).

### 2. Export the PyTorch Brain
Rust requires a C++ compatible TorchScript file. Compile the baseline Python neural network:
```bash
pip3 install torch --index-url [https://download.pytorch.org/whl/cpu](https://download.pytorch.org/whl/cpu) --break-system-packages
python3 export_brain.py
mv zyo_brain.pt zyo_agent_rs/
```

### 3. Download the C++ PyTorch Engine
The Rust tch crate requires the native LibTorch C++ backend to execute the tensor math in user-space:
```bash
cd zyo_agent_rs
wget [https://download.pytorch.org/libtorch/cpu/libtorch-cxx11-abi-shared-with-deps-2.2.0%2Bcpu.zip](https://download.pytorch.org/libtorch/cpu/libtorch-cxx11-abi-shared-with-deps-2.2.0%2Bcpu.zip)
unzip libtorch-cxx11-abi-shared-with-deps-2.2.0+cpu.zip
```

### 4. Compile the Baseline Kernel
Clear any existing maps, compile the baseline C code, and register the struct operations into Ring-0:
```bash
sudo rm -rf /sys/fs/bpf/zyo_*
sudo clang -O2 -g -target bpf -I . -c zyo_sched.bpf.c -o zyo_sched.bpf.o
sudo bpftool struct_ops register zyo_sched.bpf.o /sys/fs/bpf/zyo_sched
```

---
## ⚡ Execution & Testing
To observe the Agentic OS in action, you need two terminal windows.

Terminal 1: Trigger the Swarm Workload
Simulate a massive, multi-core traffic flood to stress the CPU queue and force the Fast Brain to react.
```bash
python3 safe_load.py
```

Terminal 2: Ignite the Agentic OS
Bind the C++ library path and launch the Rust Dual-Brain orchestrator:
```bash
cd zyo_agent_rs
export LIBTORCH_BYPASS_VERSION_CHECK=1
export LIBTORCH=$(pwd)/libtorch

cargo build
sudo LD_LIBRARY_PATH=${LIBTORCH}/lib ./target/debug/zyo_agent_rs
```

What to expect during the test: 
1. You will watch the RL Agent print its real-time telemetry, dynamically tuning the CPU weight map based on the microsecond latency and issuing Rewards/Punishments.
2. When the Swarm workload intentionally pushes the latency beyond 800.0 µs, the system will slam the "RED ALERT".
3. The Rust orchestrator will instantly load the SafeMode scheduler, package the C code, and ping the local LLM asynchronously.
4. You will see a brief pause as the LLM synthesizes the new scheduler logic.
5. The terminal will output [*] HOT-SWAP SUCCESSFUL, surgically replacing its own brain while keeping your system online.

---
## 🔧 Troubleshooting
* **libtorch_cpu.so: cannot open shared object file:** Sudo strips environment variables. You must pass the library path directly in the execution command: `sudo LD_LIBRARY_PATH=$(pwd)/libtorch/lib ./target/debug/zyo_agent_rs`.
* **Connection Refused on Port 11434:** If Rust cannot reach the LLM, you are likely hitting a WSL2 boundary. Find your Windows host IP from Linux by running `ip route | grep default | awk '{print $3}'` (usually 10.0.2.2 or 172.x.x.x). Update the URL in `src/main.rs` with this IP.
* **Rust tch crate compiler errors:** Ensure you have exported `LIBTORCH_BYPASS_VERSION_CHECK=1` to allow tch v0.15.0 to link with PyTorch 2.2.0.

---
## 📜 License & Sovereignty
( TheSNMC )