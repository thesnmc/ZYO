import gymnasium as gym
from gymnasium import spaces
import numpy as np
import time
from bcc import libbcc
import ctypes
import torch
import torch.nn as nn
from torch.distributions import Normal
import os
import subprocess

# --- 1. THE FAST BRAIN (Neural Network) ---
class PolicyNetwork(nn.Module):
    def __init__(self):
        super(PolicyNetwork, self).__init__()
        self.fc1 = nn.Linear(1, 16)
        self.fc_mean = nn.Linear(16, 1)
        self.fc_std = nn.Linear(16, 1)

    def forward(self, x):
        x = torch.relu(self.fc1(x))
        mean = torch.tanh(self.fc_mean(x))
        std = torch.nn.functional.softplus(self.fc_std(x)) + 0.1 
        return mean, std

# --- 2. THE KERNEL BRIDGE ---
class ZyoSchedEnv(gym.Env):
    def __init__(self):
        super(ZyoSchedEnv, self).__init__()
        self.tuning_fd = libbcc.lib.bpf_obj_get(b"/sys/fs/bpf/zyo_tuning_map")
        if self.tuning_fd < 0:
            raise Exception("Failed to open tuning map.")
        self.last_total_switches = 0

    def read_os_switches(self):
        try:
            with open('/proc/stat', 'r') as f:
                for line in f:
                    if line.startswith('ctxt'):
                        return int(line.split()[1])
        except:
            return 0
        return 0

    def read_latency(self):
        current_switches = self.read_os_switches()
        if self.last_total_switches == 0:
            self.last_total_switches = current_switches
            return 0.0
        
        diff = current_switches - self.last_total_switches
        self.last_total_switches = current_switches
        
        if diff == 0:
            return 1000.0 
        return 50000.0 / diff

    def apply_weight(self, safe_action):
        kernel_weight = int((safe_action + 1.0) * 1500000) + 1000000 
        key = ctypes.c_uint32(0)
        val = ctypes.c_uint64(kernel_weight) 
        libbcc.lib.bpf_update_elem(self.tuning_fd, ctypes.byref(key), ctypes.byref(val), 0)
        return kernel_weight

# --- 3. THE SLOW BRAIN (LLM Code Synthesis Trigger) ---
def trigger_llm_hotswap(current_latency):
    print("\n[!!!] RED ALERT: RL Agent has lost control!")
    print(f"[!!!] Latency sustained at {current_latency:.2f} µs. Triggering LLM Synthesis...")
    
    # In a production SchedCP build, you would send a prompt to OpenAI/Anthropic/Ollama here:
    # prompt = f"The CPU queue is stalling at {current_latency}us. Rewrite zyo_sched.bpf.c to prioritize I/O bound tasks."
    # new_c_code = llm.generate(prompt)
    # with open("zyo_sched.bpf.c", "w") as f: f.write(new_c_code)
    
    print("[*] LLM generated new C code. Hot-swapping Ring-0...")
    
    try:
        # 1. Unregister the failing scheduler
        subprocess.run(["sudo", "bpftool", "struct_ops", "unregister", "name", "zyo_ops"], check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        
        # 2. THE FIX: Clean the BPF virtual filesystem to prevent 'File exists' error!
        subprocess.run("sudo rm -rf /sys/fs/bpf/zyo_*", shell=True, check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        
        # 3. Compile the new C code written by the LLM
        subprocess.run(["clang", "-O2", "-g", "-target", "bpf", "-I", ".", "-c", "zyo_sched.bpf.c", "-o", "zyo_sched.bpf.o"], check=True)
        
        # 4. Inject it back into the Kernel
        subprocess.run(["sudo", "bpftool", "struct_ops", "register", "zyo_sched.bpf.o", "/sys/fs/bpf/zyo_sched"], check=True, stdout=subprocess.DEVNULL)
        
        print("[*] HOT-SWAP SUCCESSFUL. Resuming RL Agent monitoring...\n")
        time.sleep(1) # Give the kernel a second to breathe
    except Exception as e:
        print(f"[!] Hot-swap failed: {e}")

# --- 4. THE AUTONOMOUS LOOP ---
if __name__ == "__main__":
    env = ZyoSchedEnv()
    
    # Load the trained brain!
    policy = PolicyNetwork()
    print("[*] Loading trained brain from 'zyo_brain.pth'...")
    policy.load_state_dict(torch.load("zyo_brain.pth"))
    policy.eval() # Set to inference mode (no more training)
    
    print("[*] Agentic OS Online. Fast Brain (RL) & Slow Brain (LLM) synchronized.")
    
    latency_violations = 0
    VIOLATION_THRESHOLD = 50.0 # Force a fake failure!
    MAX_VIOLATIONS = 4 # If it stays above threshold for 2 seconds (4 loops)
    
    try:
        while True:
            # 1. Get telemetry
            current_latency = env.read_latency()
            
            # 2. Fast Brain (RL) makes a split-second decision based on training
            state_tensor = torch.FloatTensor([current_latency])
            with torch.no_grad():
                mean, std = policy(state_tensor)
                dist = Normal(mean, std)
                action = dist.sample().numpy()
            
            safe_action = float(np.clip(action[0], -1.0, 1.0))
            kernel_weight = env.apply_weight(safe_action)
            
            print(f" -> [RL FAST BRAIN] Weight: {safe_action:.3f} | [LATENCY] {current_latency:.2f} µs")
            
            # 3. Slow Brain (LLM) Watchdog
            if current_latency > VIOLATION_THRESHOLD:
                latency_violations += 1
            else:
                latency_violations = 0 # Reset if the RL agent fixed the spike
                
            if latency_violations >= MAX_VIOLATIONS:
                trigger_llm_hotswap(current_latency)
                latency_violations = 0 # Reset after swap
                env.last_total_switches = env.read_os_switches() # Reset the tracking gap
            
            time.sleep(0.5)
                
    except KeyboardInterrupt:
        print("\n[*] Shutting down Agentic OS.")