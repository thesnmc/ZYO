import torch
import torch.nn as nn

class PolicyNetwork3D(nn.Module):
    def __init__(self):
        super(PolicyNetwork3D, self).__init__()
        # NEW: The input layer now accepts 3 dimensions!
        self.fc1 = nn.Linear(3, 16)
        self.fc_mean = nn.Linear(16, 1)

    def forward(self, x):
        x = torch.relu(self.fc1(x))
        return torch.tanh(self.fc_mean(x))

print("[*] Synthesizing 3D Neural Network...")
model = PolicyNetwork3D()
model.eval()

# Example input tensor: [Latency, Interrupts, MemPressure]
example_input = torch.tensor([[150.0, 5000.0, 45.2]])
traced_script_module = torch.jit.trace(model, example_input)
traced_script_module.save("zyo_brain_v3.pt")
print("[*] V3 Brain successfully exported as zyo_brain_v3.pt!")
