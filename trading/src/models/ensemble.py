import torch
import torch.nn as nn
from src.models.base_model import BaseModel


class TimeseriesEnsemble(BaseModel):
    def __init__(self, input_size, hidden_size):
        super().__init__()

        self.lstm = nn.LSTM(
            input_size=input_size,
            hidden_size=hidden_size,
            num_layers=2,
            dropout=0.2,
            batch_first=True
        )

        self.cnn = nn.Sequential(
            nn.Linear(input_size, 64),
            nn.LeakyReLU(),
            nn.BatchNorm1d(64),
            nn.Linear(64, 64),
            nn.LeakyReLU(),
            nn.BatchNorm1d(64)
        )

        self.dnn = nn.Sequential(
            nn.Linear(hidden_size + 64, 128),
            nn.LeakyReLU(),
            nn.BatchNorm1d(128),
            nn.Dropout(0.3),
            nn.Linear(128, 64),
            nn.LeakyReLU(),
            nn.BatchNorm1d(64),
            nn.Linear(64, 3)
        )

        # Initialize weights
        self.apply(self._init_weights)

    def _init_weights(self, m):
        if isinstance(m, nn.Linear):
            nn.init.kaiming_normal_(m.weight)
            if m.bias is not None:
                nn.init.constant_(m.bias, 0)

    def forward(self, x):
        if len(x.size()) == 2:
            x = x.unsqueeze(1)

        x = torch.nan_to_num(x, 0.0)

        # LSTM branch
        lstm_out, _ = self.lstm(x)
        lstm_last = lstm_out[:, -1, :]

        # CNN branch
        cnn_out = self.cnn(x[:, -1, :])

        # Combine branches
        combined = torch.cat([lstm_last, cnn_out], dim=1)
        logits = self.dnn(combined)

        return logits
