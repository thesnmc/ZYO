# ZYO: Autonomous Agentic OS Scheduler

**Zyo** is a proof-of-concept **Agentic Operating System** architecture. It replaces static, human-coded Linux CPU scheduling rules with an autonomous, dual-brain AI agent that continuously analyzes running applications and writes custom eBPF scheduling policies on the fly to optimize for throughput and latency.

Built with an absolute focus on **privacy-by-design, data sovereignty, and offline-first computing**, the entire architecture—from the Ring-0 telemetry bridge to the 7-billion parameter code-synthesizing LLM—runs locally on host hardware. Zero telemetry leaves the motherboard. It is a completely self-reliant intelligence.

---

## 🚀 Core Features
* **Decoupled Agentic Control Plane:** Heavy AI reasoning (Reinforcement Learning and LLM generation) happens safely in user-space, while the actual task scheduling executes in kernel-space via eBPF. The kernel remains blisteringly fast.
* **Microsecond Dynamic Tuning:** A custom PyTorch RL Agent continuously hunts for the CPU "Goldilocks Zone" by dynamically updating eBPF map weights based on real-time context-switch latency.
* **Real-Time Code Synthesis:** When extreme workloads trigger latency thresholds, the system packages its own failing C code, prompts a local LLM to fix the logic, and receives a newly compiled kernel module.
* **Zero-Downtime Hot-Swapping:** New kernel operations are seamlessly linked (and old memory maps garbage-collected) using `sched_ext` without dropping SSH connections or stalling active processes.
* **Zero-Crash Guarantee:** Mathematically protected by the Linux eBPF verifier. If the AI hallucinates an infinite loop or unsafe memory access, the verifier rejects the injection, ensuring the host OS never panics.
* **Cross-OS Telemetry Bridge:** Engineered to allow a Linux Kernel (WSL2/VM) to route its raw Ring-0 telemetry to a localized AI server running on a separate host OS (Windows), bridging environments seamlessly.

---

## 🧠 The Dual-Brain Architecture

### 1. The Data Plane (eBPF / C)
A dynamic CPU scheduler injected directly into the Linux kernel via the `sched_ext` framework (`scx`). It uses eBPF array maps to allow the user-space AI to manipulate scheduling weights and time-slices in real-time.

### 2. The Fast Brain (PyTorch RL Agent)
A Reinforcement Learning neural network operating in continuous microsecond loops. It reads raw hardware context switches from `/proc/stat`. Using a strict reward function based on decreasing tail-latency, it calculates and applies the optimal CPU time-slice allocation to maximize throughput.

### 3. The Slow Brain (Local LLM Watchdog)
A localized LLM (Qwen 2.5 7B via Ollama). If the Fast Brain encounters a workload it cannot stabilize (e.g., persistent latency spikes), the Slow Brain is triggered. It reads the failing eBPF C code, synthesizes a new scheduling logic string, invokes Clang to recompile, and seamlessly hot-swaps the new maps into Ring-0.

---

## ⚙️ Prerequisites & System Requirements

To run this architecture, you need a highly specific Linux environment.

**Kernel & Toolchain:**
* Linux Kernel **6.12+** (Must be compiled with `CONFIG_SCHED_CLASS_EXT=y`, `CONFIG_BPF=y`, `CONFIG_DEBUG_INFO_BTF=y`)
* Clang & LLVM (`>=16`)
* `bpftool` and `libbpf-dev`

**AI & Python Environment:**
* Python 3.10+ 
* Python Libraries: `torch`, `gymnasium`, `bcc`, `requests`, `numpy`
* **Ollama** (Running locally with `qwen2.5-coder:7b` pulled)

---

## 🛠️ Installation & Setup

### 1. Generate Hardware-Specific Headers
You cannot use a pre-compiled `vmlinux.h` as it must map exactly to your machine's kernel structure. Generate it locally in your project root:
```bash
bpftool btf dump file /sys/kernel/btf/vmlinux format c > vmlinux.h
```

### 2. Prepare the AI Server (Ollama)
Ensure Ollama is running and accessible. Download the C-coding model:
```bash
ollama pull qwen2.5-coder:7b
```
*(Note: If running Linux via WSL2 and Ollama on your Windows host, you must set `$env:OLLAMA_HOST="0.0.0.0"` in your Windows PowerShell before running `ollama serve` so it accepts the Linux bridge connection).*

### 3. Setup the Python Environment
Create a virtual environment and install the required neural network and bridging libraries:
```bash
python3 -m venv zyo-env
source zyo-env/bin/activate
pip install torch gymnasium bcc requests numpy
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

**Terminal 1: Trigger the Swarm Workload**
Simulate a massive, multi-core traffic flood to stress the CPU queue and force the Fast Brain to react.
```bash
python3 safe_load.py
```

**Terminal 2: Ignite the Agentic OS**
Launch the Dual-Brain orchestrator (ensure you run this with the Python executable from your virtual environment):
```bash
sudo ./zyo-env/bin/python zyo_agent_v3.py
```

**What to expect during the test:** 1. You will watch the RL Agent print its real-time telemetry, dynamically tuning the CPU weight map based on the microsecond latency.
2. When the Swarm workload intentionally pushes the latency beyond the hardcoded `VIOLATION_THRESHOLD`, the system will slam the "RED ALERT".
3. The RL agent will halt, package the C code, and ping the local LLM.
4. You will see a brief pause as the LLM synthesizes the new scheduler logic.
5. The terminal will output `[*] HOT-SWAP SUCCESSFUL`, dynamically incrementing the map and link IDs as the kernel surgically replaces its own brain.

---

## 🔧 Troubleshooting
* **Connection Refused on Port 11434:** If your Python script cannot reach the LLM, you are likely hitting a WSL2 boundary. Find your Windows host IP from Linux by running `ip route | grep default | awk '{print $3}'` (usually `10.0.2.2` or `172.x.x.x`). Update the `requests.post` URL in `zyo_agent_v3.py` with this IP.
* **Operation Not Permitted (Clang):** Ensure you are running the compile commands with `sudo` and that previous `/sys/fs/bpf/zyo_*` maps have been completely removed.
* **Missing bcc module:** `bcc` bindings for Python often require system-level installation depending on your distro (`sudo apt install python3-bpfcc`).

*( TheSNMC )*