# 🚀 ZYO: Autonomous Agentic OS Scheduler & Ring-0 Firewall
> A sovereign, self-healing Linux kernel intelligence grid that dynamically rewrites CPU scheduling and vaporizes network threats in real-time.

[![License](https://img.shields.io/badge/License-TheSNMC-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Linux%206.12%2B%20(Bare%20Metal)-lightgrey)]()
[![Architecture](https://img.shields.io/badge/Architecture-Offline--First-success)]()

---

## 📖 Overview
Modern cloud infrastructure and operating systems are fundamentally bottlenecked by legacy, static design. Traditional Linux CPU schedulers (like CFS/EEVDF) rely on generalized heuristics that cannot adapt to massive, erratic workloads. Simultaneously, reactive firewalls (like `iptables`) process malicious packets *after* they enter the OS network stack, consuming critical CPU cycles and allowing datacenters to be overwhelmed by DDoS floods.

ZYO replaces this static paradigm with an Agentic Operating System Architecture. By decoupling the execution plane (Kernel-space eBPF) from the reasoning plane (User-space AI), ZYO creates a localized, multi-brain intelligence grid. It operates a fast-loop Reinforcement Learning neural network to optimize CPU time-slices natively in micro-seconds, while a localized 7-Billion parameter LLM acts as an asynchronous watchdog, capable of rewriting, recompiling, and hot-swapping core kernel C code on the fly if the system destabilizes.

**The Core Mandate:** Absolute data sovereignty and privacy-by-design. The entire architecture—from Ring-0 telemetry ingestion to Deep Learning tensor math and LLM code synthesis—executes 100% offline on local bare-metal hardware. Zero telemetry leaves the motherboard.

## ✨ Key Features
* **Multi-Dimensional Telemetry (3D Tensor):** The AI processes a mathematically normalized 3D tensor of `[Latency, Interrupts, Memory Pressure]`, scheduling tasks based on the actual physical stress of the entire motherboard rather than just CPU context switches.
* **XDP Network Shield (Ring-0 Packet Vaporizer):** Utilizes eBPF `XDP` to attach directly to the Network Interface Card (NIC). When the AI detects a bandwidth anomaly (>1MB/s), it dynamically injects the attacker's IP into a Ring-0 drop list, vaporizing malicious packets at the driver layer before Linux allocates memory for them.
* **The "Time Dilation" Fix (Asynchronous Orchestration):** When CPU latency breaches safety thresholds, an instant "Panic Button" hot-swaps a hardcoded SafeMode scheduler in nanoseconds, preventing system freeze while the LLM thinks in a background Tokio thread.
* **Automated Program Repair (Zero-Crash Guarantee):** If the LLM hallucinates flawed C code, the strict Linux eBPF verifier rejects it. Zyo captures the Clang compiler errors, feeds them back into the LLM context, and executes a 3-strike circuit breaker loop to autonomously repair the syntax.
* **Anti-Thrashing Momentum (EMA):** The RL agent features a `0.2` Exponential Moving Average damping factor and a `10000.0` hard-safety floor, preventing violent CPU weight swings that would otherwise destroy the L3 cache.
* **Lockless Memory Bridge:** Utilizes `BPF_MAP_TYPE_PERCPU_ARRAY`. The Rust orchestrator converts PyTorch weight calculations into Little-Endian hex bytes and injects them directly into Ring-0, allowing each CPU core to read isolated memory without spinlock contention.
* **Continuous Offline Memory Bank:** Logs exact hardware states and AI reward/punishment actions to a local CSV every 500ms, creating an offline dataset for future neural network training.

## 🛠️ Tech Stack
* **Language:** Rust (Orchestrator), C (eBPF Data Plane), Python (Model Export)
* **Framework:** `sched_ext` (scx), `XDP` (eXpress Data Path), `Tokio` (Async Rust)
* **Environment:** Linux Kernel 6.12+, Clang/LLVM 16+
* **Key Libraries/APIs:** LibTorch (PyTorch C++ Engine v2.2.0), Ollama API (Qwen 2.5 Coder 7B), `bpftool`, `libbpf-dev`

## ⚙️ Architecture & Data Flow
The system bridges Deep Learning, Rust, and eBPF across a dual-plane architecture:

* **Input (Telemetry Gather):** The Rust daemon polls `/proc/stat`, `/proc/meminfo`, and NIC driver `RX_BYTES` every 500ms. The data is normalized and clamped into a rigid 1D tensor shape.
* **Processing (The Dual Brain):** * *Fast Brain:* The LibTorch C++ engine processes the tensor through a Reinforcement Learning model, calculating optimal CPU weights using a latency-delta reward function.
    * *Slow Brain:* If hardware stress crosses critical thresholds, the `reqwest` client pings the local Qwen 2.5 LLM with the failing C code to synthesize new logic.
* **Output (Ring-0 Injection):** Rust executes JIT compilation via `clang -target bpf`. Hexadecimal bytes and IPv4 arrays are injected directly into the kernel's `struct_ops` and `HASH` maps via `bpftool`, altering system routing and scheduling instantly.

## 🔒 Privacy & Data Sovereignty
* **Data Collection:** Zero external collection. All telemetry (latency, network logs, memory pressure) is appended strictly to a local `zyo_training_data_v3.csv` file on the host machine.
* **Permissions Required:** `sudo` (Root) is strictly required to mount eBPF filesystems (`/sys/fs/bpf/`), register `sched_ext` schedulers, and attach XDP objects to network interfaces.
* **Cloud Connectivity:** Completely disabled/Not required. The LLM runs via a local Ollama bridge (`10.0.2.2`), and the PyTorch engine is a statically linked C++ binary.

## 🚀 Getting Started

### Prerequisites
* **OS:** Linux Kernel **6.12+** compiled with `CONFIG_SCHED_CLASS_EXT=y`, `CONFIG_BPF=y`, `CONFIG_DEBUG_INFO_BTF=y`.
* **Environment:** Clang & LLVM (>=16), `bpftool`, `libbpf-dev`, Rust Toolchain (Cargo).
* **AI Stack:** Ollama installed locally. 

### Installation

1. **Clone the repository:**
   ```bash
   git clone [https://github.com/thesnmc/ZYO.git](https://github.com/thesnmc/ZYO.git)
   cd ZYO
   ```

2. **Prepare the Local LLM:**
   Ensure Ollama is bound to `0.0.0.0` (if bridging from a host) and pull the coder model:
   ```bash
   ollama pull qwen2.5-coder:7b
   ```

3. **Export the PyTorch Fast Brain:**
   Compile the baseline Python RL network into a C++ TorchScript file:
   ```bash
   cd zyo_agent_rs
   pip3 install torch --index-url [https://download.pytorch.org/whl/cpu](https://download.pytorch.org/whl/cpu) --break-system-packages
   python3 export_v3_brain.py
   ```

4. **Install LibTorch (C++ Backend):**
   ```bash
   wget [https://download.pytorch.org/libtorch/cpu/libtorch-cxx11-abi-shared-with-deps-2.2.0%2Bcpu.zip](https://download.pytorch.org/libtorch/cpu/libtorch-cxx11-abi-shared-with-deps-2.2.0%2Bcpu.zip)
   unzip libtorch-cxx11-abi-shared-with-deps-2.2.0+cpu.zip
   ```

5. **Compile the Ring-0 eBPF Objects:**
   ```bash
   sudo rm -rf /sys/fs/bpf/zyo_*
   sudo clang -O2 -g -target bpf -I . -c zyo_sched.bpf.c -o zyo_sched.bpf.o
   sudo clang -O2 -g -target bpf -I/usr/include/x86_64-linux-gnu -c zyo_shield.bpf.c -o zyo_shield.bpf.o
   sudo bpftool struct_ops register zyo_sched.bpf.o /sys/fs/bpf/zyo_sched
   ```

6. **Ignite the Grid:**
   Bypass ABI version checks, link the C++ library, and run the Rust orchestrator as root:
   ```bash
   export LIBTORCH=$(pwd)/libtorch
   export LIBTORCH_BYPASS_VERSION_CHECK=1
   cargo build
   sudo LD_LIBRARY_PATH=${LIBTORCH}/lib ./target/debug/zyo_agent_rs
   ```

## 🤝 Contributing
Contributions, issues, and feature requests are welcome. Feel free to check the issues page if you want to contribute to the Sovereign Grid.

## 📄 License
This project is licensed under the TheSNMC License - see the LICENSE file for details.  
Built by an independent developer in Chennai, India.