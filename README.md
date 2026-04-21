# ZYO: Autonomous Agentic OS Scheduler (SchedCP / OS-R1 Architecture)

Zyo is a proof-of-concept **Agentic Operating System** architecture. It replaces static, human-coded Linux CPU scheduling rules with an autonomous, dual-brain AI agent that continuously analyzes running applications and writes custom eBPF scheduling policies on the fly to optimize for throughput and latency.

Built with an absolute focus on **privacy-by-design and offline-first computing**, the entire architecture—from the Ring-0 telemetry bridge to the 7-billion parameter code-synthesizing LLM—runs locally on host hardware. Zero telemetry leaves the motherboard.

## 🧠 The Dual-Brain Architecture

This system decouples the control plane (AI orchestration) from the data plane (eBPF kernel execution) to ensure the Linux kernel remains incredibly fast while benefiting from deep machine learning.

* **The Data Plane (eBPF / C):** A dynamic CPU scheduler injected directly into the Linux kernel via the `sched_ext` framework. It uses eBPF maps to allow user-space to manipulate scheduling weights in real-time.
* **The Fast Brain (PyTorch RL Agent):** A Reinforcement Learning neural network operating in microsecond loops. It reads raw hardware context switches from `/proc/stat` and calculates the optimal CPU time-slice allocation (the "Goldilocks Zone") to maximize throughput.
* **The Slow Brain (Local LLM Watchdog):** A localized LLM (Qwen 2.5 7B via Ollama). If the Fast Brain encounters a workload it cannot stabilize (e.g., latency spikes above 150µs), the Slow Brain is triggered. It reads the failing eBPF C code, synthesizes a new scheduling logic string, invokes Clang to recompile, and seamlessly hot-swaps the new maps into Ring-0 without dropping system connections.

## 🛡️ Zero-Crash Guarantee
Because this architecture utilizes the Linux eBPF verifier, the system is mathematically protected from AI hallucinations. If the Local LLM generates an infinite loop or unsafe memory pointer in the C code, the kernel verifier outright rejects the hot-swap, ensuring the host OS never panics or crashes.

---

## ⚙️ Prerequisites

To run this architecture, you need a highly specific Linux environment:
* **Linux Kernel 6.12+** (Compiled with `CONFIG_SCHED_CLASS_EXT=y`, `CONFIG_BPF=y`, `CONFIG_DEBUG_INFO_BTF=y`)
* **Clang & LLVM** (`>=16`)
* **bpftool**
* **Python 3.10+** (with `torch`, `gymnasium`, `bcc`, `requests`)
* **Ollama** (Running locally with `qwen2.5-coder:7b` pulled)

## 🚀 Getting Started

### 1. Generate Hardware-Specific Headers
You cannot use a pre-compiled `vmlinux.h` as it must map exactly to your machine's kernel structure. Generate it locally:
```bash
bpftool btf dump file /sys/kernel/btf/vmlinux format c > vmlinux.h
```

### 2. Prepare the AI Server (Ollama)
Ensure Ollama is running and accessible. If running Linux via WSL2 and Ollama on the Windows host, ensure OLLAMA_HOST="0.0.0.0" is set on the host, and update the target IP in zyo_agent_v3.py.
```bash
ollama pull qwen2.5-coder:7b
```

### 3. Compile and Inject the Baseline Kernel
Clear any existing maps, compile the baseline C code, and register the struct operations:
```bash
sudo rm -rf /sys/fs/bpf/zyo_*
sudo clang -O2 -g -target bpf -I . -c zyo_sched.bpf.c -o zyo_sched.bpf.o
sudo bpftool struct_ops register zyo_sched.bpf.o /sys/fs/bpf/zyo_sched
```

## ⚡ Execution

### Terminal 1: Trigger the Swarm
Simulate a massive multi-core workload to stress the CPU queue.
```bash
python3 safe_load.py
```

### Terminal 2: Ignite the Agentic OS
Launch the Dual-Brain orchestrator.
```bash
sudo python3 zyo_agent_v3.py
```

What to expect: You will watch the RL Agent dynamically tune the CPU weight map. When the workload intentionally exceeds the hardcoded VIOLATION_THRESHOLD, you will see the system halt, ping the local LLM, compile the newly generated C code, and live-swap the kernel scheduler map ids in real-time.

( TheSNMC )