import torch
import torch.nn as nn

class PolicyNetwork(nn.Module):
    def __init__(self):
        super(PolicyNetwork, self).__init__()
        self.fc1 = nn.Linear(1, 16)
        self.fc_mean = nn.Linear(16, 1)

    def forward(self, x):
        x = torch.relu(self.fc1(x))
        return torch.tanh(self.fc_mean(x))

model = PolicyNetwork()
model.load_state_dict(torch.load("zyo_brain.pth"), strict=False)
model.eval()

example_input = torch.tensor([[160.0]])
traced_script_module = torch.jit.trace(model, example_input)
traced_script_module.save("zyo_brain.pt")
print("[*] Brain successfully compiled to zyo_brain.pt!")
