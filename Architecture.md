# 🏗️ Architecture & Design Document: ZYO Agentic OS & Ring-0 Firewall
**Version:** 3.0.0 | **Date:** 2026-04-22 | **Author:** Sujay

---

## 1. Executive Summary
This document outlines the architecture for ZYO, a sovereign, self-healing Linux intelligence grid. ZYO replaces static, human-coded kernel scheduling heuristics with an autonomous, dual-brain AI agent. It is designed to continuously analyze hardware stress via a 3D Deep Learning Tensor and dynamically synthesize custom eBPF scheduling policies. Simultaneously, it operates a Ring-0 XDP network shield to vaporize DDoS threats at the driver level. The system operates entirely offline, adhering to a strict data sovereignty mandate, ensuring zero telemetry ever leaves the host motherboard.

## 2. Architectural Drivers
* **Primary Goals:** Autonomous infrastructure self-healing, real-time workload optimization, and zero-latency network defense without human intervention.
* **Technical Constraints:** Must execute entirely on local bare-metal hardware. Must interface natively with Linux Kernel 6.12+ without requiring customized kernel recompilations (utilizing standard `sched_ext` and `XDP` hooks).
* **Non-Functional Requirements (NFRs):** * **Security/Privacy:** 100% data sovereignty. Air-gapped AI reasoning. No cloud API dependencies for LLM code synthesis.
    * **Reliability:** Zero-Crash Guarantee. The system must mathematically survive LLM hallucinations and compilation failures.
    * **Performance:** The user-space orchestrator must consume < 1% CPU overhead to prevent observing the system from impacting the system (avoiding the Heisenberg effect). Lockless kernel memory propagation.

## 3. System Architecture (The 10,000-Foot View)

ZYO decouples the reasoning engine from the execution engine, bridging deep learning and kernel operations via a lockless memory architecture.

```text
+-------------------------------------------------------------+
|                     USER-SPACE (CONTROL PLANE)              |
|                                                             |
|  [Slow Brain: Qwen 2.5 7B]    [Fast Brain: PyTorch C++]     |
|       (Code Synthesis)             (RL Tensor Math)         |
|              ^                             ^                |
|              | (Async Repair)              | (500ms Tick)   |
|  [Rust Tokio Orchestrator & Circuit Breakers]               |
+----------------------|----------------------|---------------+
                       | bpftool HEX Inject   | Telemetry Poll
+----------------------v----------------------v---------------+
|                     RING-0 (DATA PLANE)                     |
|                                                             |
|  [BPF_MAP_TYPE_PERCPU_ARRAY]  [BPF_MAP_TYPE_HASH]           |
|                                                             |
|  [sched_ext (scx) Scheduler]  [XDP Network Firewall]        |
|  (CPU Time-Slice Control)     (Packet Vaporization)         |
+-------------------------------------------------------------+
|                     HARDWARE (BARE METAL)                   |
+-------------------------------------------------------------+
```

* **Control Plane (User-Space):** Built with Rust and the Tokio asynchronous runtime. Houses the Fast Brain (PyTorch C++ Engine) for microsecond RL weight adjustments, and the Slow Brain (Ollama/Qwen) for complex eBPF C code generation.
* **Data Plane (Ring-0):** Pure eBPF C code injected into the kernel. The CPU scheduler dictates process queues, while the XDP firewall sits directly on the Network Interface Card (NIC) to drop malicious packets before OS allocation.
* **Memory Bank (Local Data):** A continuous, offline CSV append-log capturing the 3D tensor state and reward outcomes for offline model training.

## 4. Design Decisions & Trade-Offs (The "Why")

* **Decision 1: Utilizing eBPF over Loadable Kernel Modules (LKMs)**
    * **Rationale:** LKMs can catastrophically panic the kernel if they contain a single memory bug. eBPF programs must pass a strict mathematical verifier before loading, guaranteeing system stability.
    * **Trade-off:** eBPF restricts the use of unbounded loops (e.g., `while(true)`) and limits instruction counts, forcing us to move complex logic into user-space and keep the kernel code strictly O(1).

* **Decision 2: Rust (Tokio) over Python for Orchestration**
    * **Rationale:** Python’s Global Interpreter Lock (GIL) and garbage collection pauses create fatal latency spikes in kernel time. Rust guarantees deterministic memory safety and zero-cost abstractions.
    * **Trade-off:** We had to abandon native PyTorch Python bindings and engineer a complex bridge using LibTorch C++ and the Rust `tch` crate, increasing build complexity.

* **Decision 3: `BPF_MAP_TYPE_PERCPU_ARRAY` for State Management**
    * **Rationale:** Prevents CPU lock contention.
    * **Trade-off:** Consumes slightly more RAM by duplicating the array for every logical CPU core, but guarantees that the kernel never hangs waiting for the Rust AI to finish a write operation.

* **Decision 4: The "Panic Button" (Asynchronous Hot-Swapping)**
    * **Rationale:** LLMs take 3-10 seconds to write code. The OS cannot freeze during this time.
    * **Trade-off:** Requires maintaining a pre-compiled, static `zyo_safe_fallback.o` module that is instantly swapped in on the main thread while the LLM repairs the primary scheduler in the background.

## 5. Data Flow & Lifecycle

* **Ingestion (The 3D Tensor):** Every 500ms, the Rust daemon extracts raw CPU latency, hardware interrupts, and memory pressure. It normalizes this data (val / threshold) into a rigid [1, 3] PyTorch tensor. Simultaneously, the XDP map tracks `RX_BYTES` passing through the NIC.
* **Processing (RL & Anomaly Detection):** * **Standard:** The Fast Brain calculates a delta-reward based on previous latency, applying an Exponential Moving Average (EMA damping = 0.2) to output a smooth target weight.
    * **Anomaly:** If bandwidth crosses 1MB/s, the XDP extraction logic flags the IP. If CPU latency breaches 800µs, the 3-strike Verifier Loop kicks in, feeding Clang compiler errors to the local Qwen LLM for autonomous repair.
* **Execution/Output:** Rust translates calculated weights into Little-Endian Hex and pushes them to Ring-0 via `bpftool`. The XDP map absorbs the malicious IP and drops all subsequent packets at the driver level. Hardware state is appended to the local CSV Memory Bank.

## 6. Security & Privacy Threat Model

* **Data at Rest:** Telemetry is stored exclusively in a localized CSV memory bank. No cloud-syncing or external database telemetry is enabled.
* **Data in Transit:** The reasoning engine is entirely air-gapped from cloud AI APIs (OpenAI/Anthropic). All model inference happens on the local GPU/CPU. The XDP firewall operates strictly internally, sending `XDP_DROP` instructions without transmitting outbound replies to attackers.
* **Mitigated Risks:** * **Malicious AI Code:** If the LLM hallucinates a virus or bad memory pointer, the Linux eBPF verifier intercepts and destroys the code before it reaches Ring-0.
    * **Infinite Hallucination:** A strict `max_retries = 3` circuit breaker prevents the system from burning CPU cycles if the LLM cannot fix a compiler error, failing over to SafeMode.

## 7. Future Architecture Roadmap

* **Distributed Orchestration:** Scaling the single-node orchestrator to support a swarm architecture, allowing one primary ZYO Fast Brain to coordinate eBPF map weights across 100+ bare-metal servers via secure local subnets.
* **Native Libbpf-rs Integration:** Transitioning away from `bpftool` shell commands to native Rust C-bindings for map updates, shaving additional microseconds off the IPC bridge.
* **Sovereign Code Synthesis Model:** Moving off generalized LLMs (like Qwen) and training a hyper-specific, highly quantized Small Language Model (SLM) strictly on Linux kernel C code to achieve sub-second autonomous recompilation.
