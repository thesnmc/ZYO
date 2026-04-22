import torch
import torch.nn as nn
import torch.optim as optim
import numpy as np

# 1. Define the exact same brain architecture
class PolicyNetwork(nn.Module):
    def __init__(self):
        super(PolicyNetwork, self).__init__()
        self.fc1 = nn.Linear(1, 16)
        self.fc_mean = nn.Linear(16, 1)

    def forward(self, x):
        x = torch.relu(self.fc1(x))
        return torch.tanh(self.fc_mean(x))

print("[*] Waking up offline training protocol...")

# 2. Load the Memory Bank
try:
    # Columns: timestamp, latency_us, safe_weight, reward
    data = np.genfromtxt("zyo_training_data.csv", delimiter=',', skip_header=1)
except Exception as e:
    print(f"[!] Error loading CSV: {e}")
    exit()

# 3. Filter for positive reinforcement (only learn from good moves)
good_memories = data[data[:, 3] > 0]
print(f"[*] Found {len(data)} total memories. Filtered down to {len(good_memories)} successful actions.")

if len(good_memories) < 10:
    print("[!] Not enough successful memories to train yet. Let the OS run longer to gather data!")
    exit()

# 4. Prepare the Training Data
# Input X: The CPU Latency
X = torch.tensor(good_memories[:, 1], dtype=torch.float32).unsqueeze(1)

# Output Y: Reverse-engineer the raw neural network float from the safe_weight
raw_action_targets = (good_memories[:, 2] - 1000000.0) / 1500000.0
Y = torch.tensor(raw_action_targets, dtype=torch.float32).unsqueeze(1)

# 5. Train the Brain (Offline Dreaming)
model = PolicyNetwork()
optimizer = optim.Adam(model.parameters(), lr=0.01)
criterion = nn.MSELoss()

print("[*] Initiating REM Sleep (Training V2 Brain)...")
epochs = 500
for epoch in range(epochs):
    optimizer.zero_grad()
    predictions = model(X)
    loss = criterion(predictions, Y)
    loss.backward()
    optimizer.step()
    
    if epoch % 100 == 0:
        print(f" -> Epoch {epoch}/{epochs} | Loss: {loss.item():.4f}")

# 6. Export the V2 Brain
model.eval()
example_input = torch.tensor([[160.0]])
traced_script_module = torch.jit.trace(model, example_input)
traced_script_module.save("zyo_brain_v2.pt")

print("[*] Dream sequence complete. zyo_brain_v2.pt successfully synthesized!")