from bcc import BPF
import time

# 1. The Kernel-Space code (written in C)
# This runs inside the kernel every time a new process starts (execve)
kernel_code = """
BPF_HASH(counter_map, u32, u64);

int count_exec(void *ctx) {
    u32 key = 0;
    u64 *val, initial_val = 1;

    val = counter_map.lookup(&key);
    if (val) {
        *val += 1;
    } else {
        counter_map.update(&key, &initial_val);
    }
    return 0;
}
"""

# 2. Load the code into the Kernel
print("[*] Project Zyo: Injecting connectivity probe into Kernel...")
b = BPF(text=kernel_code)
execve_fn = b.get_syscall_fnname("execve")
b.attach_kprobe(event=execve_fn, fn_name="count_exec")

# 3. The User-Space loop (Control Plane)
print("[*] Successfully bridged. Open another terminal and run some commands (ls, clear, etc.)")
print("[*] Press Ctrl+C to stop.")

try:
    while True:
        time.sleep(1)
        for key, val in b["counter_map"].items():
            print(f"Total processes launched since start: {val.value}")
except KeyboardInterrupt:
    print("\n[*] Probe removed. Sandbox clean.")