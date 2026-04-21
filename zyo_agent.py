import gymnasium as gym
from gymnasium import spaces
import numpy as np
import time
from bcc import libbcc
import ctypes
import torch
import torch.nn as nn
import torch.optim as optim
from torch.distributions import Normal

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

class ZyoSchedEnv(gym.Env):
    def __init__(self):
        super(ZyoSchedEnv, self).__init__()
        self.action_space = spaces.Box(low=-1.0, high=1.0, shape=(1,), dtype=np.float32)
        self.observation_space = spaces.Box(low=0, high=np.inf, shape=(1,), dtype=np.float32)
        
        self.tuning_fd = libbcc.lib.bpf_obj_get(b"/sys/fs/bpf/zyo_tuning_map")
        if self.tuning_fd < 0:
            raise Exception("Failed to open tuning map.")
        
        self.last_total_switches = 0

    def read_os_switches(self):
        # Reads the absolute, unbreakable hardware context switch counter from the OS
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
            
        # High OS throughput = CPU is chewing through the Swarm = Good (Low Latency)
        simulated_latency = 50000.0 / diff
        return simulated_latency

    def step(self, action):
        safe_action = float(np.clip(action[0], -1.0, 1.0))
        
        kernel_weight = int((safe_action + 1.0) * 1500000) + 1000000 
        
        key = ctypes.c_uint32(0)
        val = ctypes.c_uint64(kernel_weight) 
        
        libbcc.lib.bpf_update_elem(self.tuning_fd, ctypes.byref(key), ctypes.byref(val), 0)
        
        time.sleep(0.5) 
        
        latency_us = self.read_latency()
        reward = -(latency_us / 100.0) 
        
        print(f" -> [AI WEIGHT] {safe_action:.3f} (NS: {kernel_weight}) | [SYS LATENCY] {latency_us:.2f} µs | [REWARD] {reward:.2f}")
        return np.array([latency_us], dtype=np.float32), reward, False, False, {}

    def reset(self, seed=None):
        super().reset(seed=seed)
        self.last_total_switches = 0
        return np.array([self.read_latency()], dtype=np.float32), {}

if __name__ == "__main__":
    env = ZyoSchedEnv()
    obs, _ = env.reset()
    policy = PolicyNetwork()
    optimizer = optim.Adam(policy.parameters(), lr=0.01) 
    
    print("\n[*] PyTorch AI Initialized. Bridged to Pure OS Telemetry.")
    print("[*] ACTION REQUIRED: Open a new terminal tab and run: python3 safe_load.py")
    print("[*] Watch the AI try to optimize the load. Press Ctrl+C to stop.\n")
    
    try:
        while True:
            state_tensor = torch.FloatTensor(obs)
            mean, std = policy(state_tensor)
            dist = Normal(mean, std)
            action = dist.sample()
            
            obs, reward, _, _, _ = env.step(action.detach().numpy())
            
            log_prob = dist.log_prob(action)
            loss = -log_prob * reward 
            
            optimizer.zero_grad()
            loss.backward()
            optimizer.step()
                
    except KeyboardInterrupt:
        print("\n[*] Halting training...")
        print("[*] SAVING NEURAL NETWORK STATE TO 'zyo_brain.pth'...")
        torch.save(policy.state_dict(), "zyo_brain.pth")
        print("[*] Brain saved! You can load this later without starting from scratch.")
        print("[*] PyTorch Brain detaching. Loop halted.")