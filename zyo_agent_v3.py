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
import requests # NEW: For talking to the local LLM

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

# --- 3. THE SLOW BRAIN (Local LLM Synthesis) ---
def trigger_llm_hotswap(current_latency):
    print("\n[!!!] RED ALERT: RL Agent has lost control!")
    print(f"[!!!] Latency sustained at {current_latency:.2f} µs. Triggering Local LLM Synthesis...")
    
    # Read the current failing C code
    with open("zyo_sched.bpf.c", "r") as f:
        current_code = f.read()

    # The Prompt for the local AI
    prompt = f"""
The Linux CPU queue is stalling at {current_latency} microseconds. 
The current eBPF scheduler is failing to keep up with the Swarm workers.
Rewrite the following eBPF C code. Modify the default 'slice' value inside zyo_enqueue to prioritize throughput (try setting it to 500000).
Output ONLY the valid C code. Do not include markdown formatting or explanations.

Current Code:
{current_code}
    """
    
    try:
        print("[*] Contacting Local AI (Qwen 2.5)...")
        # Call the local free LLM API
        response = requests.post("http://localhost:11434/api/generate", json={
            "model": "qwen2.5-coder:7b",
            "prompt": prompt,
            "stream": False
        })
        
        new_c_code = response.json()['response']
        
        # Clean up any markdown the AI accidentally outputs
        if "```c" in new_c_code:
            new_c_code = new_c_code.split("```c")[1].split("```")[0].strip()
        elif "```" in new_c_code:
            new_c_code = new_c_code.split("```")[1].split("```")[0].strip()
            
        # Overwrite the file with the AI's new code
        with open("zyo_sched.bpf.c", "w") as f:
            f.write(new_c_code)
            
        print("[*] AI generated new C code. Hot-swapping Ring-0...")
        
        # Unhook, Wipe, Compile, Inject
        subprocess.run(["sudo", "bpftool", "struct_ops", "unregister", "name", "zyo_ops"], check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        subprocess.run("sudo rm -rf /sys/fs/bpf/zyo_*", shell=True, check=False, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        subprocess.run(["sudo", "clang", "-O2", "-g", "-target", "bpf", "-I", ".", "-c", "zyo_sched.bpf.c", "-o", "zyo_sched.bpf.o"], check=True)
        subprocess.run(["sudo", "bpftool", "struct_ops", "register", "zyo_sched.bpf.o", "/sys/fs/bpf/zyo_sched"], check=True, stdout=subprocess.DEVNULL)
        
        print("[*] HOT-SWAP SUCCESSFUL. Resuming RL Agent monitoring...\n")
        time.sleep(1) 
    except Exception as e:
        print(f"[!] Hot-swap or AI Generation failed: {e}")

# --- 4. THE AUTONOMOUS LOOP ---
if __name__ == "__main__":
    env = ZyoSchedEnv()
    
    policy = PolicyNetwork()
    print("[*] Loading trained brain from 'zyo_brain.pth'...")
    policy.load_state_dict(torch.load("zyo_brain.pth"))
    policy.eval() 
    
    print("[*] Agentic OS Online. Fast Brain (RL) & Slow Brain (Local LLM) synchronized.")
    
    latency_violations = 0
    VIOLATION_THRESHOLD = 150.0 # Keeping it low to force the trigger for testing
    MAX_VIOLATIONS = 4 
    
    try:
        while True:
            current_latency = env.read_latency()
            
            state_tensor = torch.FloatTensor([current_latency])
            with torch.no_grad():
                mean, std = policy(state_tensor)
                dist = Normal(mean, std)
                action = dist.sample().numpy()
            
            safe_action = float(np.clip(action[0], -1.0, 1.0))
            kernel_weight = env.apply_weight(safe_action)
            
            print(f" -> [RL FAST BRAIN] Weight: {safe_action:.3f} | [LATENCY] {current_latency:.2f} µs")
            
            if current_latency > VIOLATION_THRESHOLD:
                latency_violations += 1
            else:
                latency_violations = 0 
                
            if latency_violations >= MAX_VIOLATIONS:
                trigger_llm_hotswap(current_latency)
                latency_violations = 0 
                env.last_total_switches = env.read_os_switches() 
            
            time.sleep(0.5)
                
    except KeyboardInterrupt:
        print("\n[*] Shutting down Agentic OS.")