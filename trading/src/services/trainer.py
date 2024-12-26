import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import DataLoader
import os
from typing import Tuple
from src.models.ensemble import TimeseriesEnsemble


class DualTimeframeTrainer:
    def __init__(
        self,
        model_15m: TimeseriesEnsemble,
        model_1h: TimeseriesEnsemble,
        learning_rate: float = 1e-4
    ):
        self.model_15m = model_15m
        self.model_1h = model_1h
        self.device = torch.device(
            'cuda' if torch.cuda.is_available() else 'cpu')

        self.model_15m.to(self.device)
        self.model_1h.to(self.device)

        self.criterion = nn.MSELoss()
        self.optimizer_15m = optim.Adam(
            self.model_15m.parameters(), lr=learning_rate)
        self.optimizer_1h = optim.Adam(
            self.model_1h.parameters(), lr=learning_rate)

    def train_epoch(
        self,
        dataloader_15m: DataLoader,
        dataloader_1h: DataLoader
    ) -> float:
        self.model_15m.train()
        self.model_1h.train()
        total_loss = 0

        for (batch_15m, target_15m), (batch_1h, target_1h) in zip(dataloader_15m, dataloader_1h):
            batch_15m, target_15m = batch_15m.to(
                self.device), target_15m.to(self.device)
            batch_1h, target_1h = batch_1h.to(
                self.device), target_1h.to(self.device)

            # Train 15m model
            self.optimizer_15m.zero_grad()
            pred_15m = self.model_15m(batch_15m)
            loss_15m = self.criterion(pred_15m, target_15m)

            # Train 1h model
            self.optimizer_1h.zero_grad()
            pred_1h = self.model_1h(batch_1h)
            loss_1h = self.criterion(pred_1h, target_1h)

            # Combined loss with higher weight for 1h timeframe
            loss = 0.4 * loss_15m + 0.6 * loss_1h
            loss.backward()

            # Add gradient clipping
            torch.nn.utils.clip_grad_norm_(self.model_15m.parameters(), 1.0)
            torch.nn.utils.clip_grad_norm_(self.model_1h.parameters(), 1.0)

            self.optimizer_15m.step()
            self.optimizer_1h.step()

            total_loss += loss.item()

        return total_loss / len(dataloader_15m)

    def validate(
        self,
        dataloader_15m: DataLoader,
        dataloader_1h: DataLoader
    ) -> float:
        self.model_15m.eval()
        self.model_1h.eval()
        total_loss = 0

        with torch.no_grad():
            for (batch_15m, target_15m), (batch_1h, target_1h) in zip(dataloader_15m, dataloader_1h):
                batch_15m, target_15m = batch_15m.to(
                    self.device), target_15m.to(self.device)
                batch_1h, target_1h = batch_1h.to(
                    self.device), target_1h.to(self.device)

                pred_15m = self.model_15m(batch_15m)
                loss_15m = self.criterion(pred_15m, target_15m)

                pred_1h = self.model_1h(batch_1h)
                loss_1h = self.criterion(pred_1h, target_1h)

                loss = 0.4 * loss_15m + 0.6 * loss_1h
                total_loss += loss.item()

        return total_loss / len(dataloader_15m)

    def save_models(self, path: str, prefix: str):
        if not os.path.exists(path):
            os.makedirs(path)

        torch.save(
            self.model_15m.state_dict(),
            os.path.join(path, f"{prefix}_15m.pth")
        )
        torch.save(
            self.model_1h.state_dict(),
            os.path.join(path, f"{prefix}_1h.pth")
        )

    def load_models(self, path: str, prefix: str):
        self.model_15m.load_state_dict(
            torch.load(os.path.join(path, f"{prefix}_15m.pth"))
        )
        self.model_1h.load_state_dict(
            torch.load(os.path.join(path, f"{prefix}_1h.pth"))
        )
