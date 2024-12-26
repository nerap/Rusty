import torch
from torch.utils.data import Dataset
import numpy as np
import pandas as pd
from sklearn.preprocessing import MinMaxScaler
from src.config.database import SessionLocal
from datetime import datetime


class MarketDataset(Dataset):
    def __init__(self, timeframe_id: str, start_time: datetime, end_time: datetime):
        self.session = SessionLocal()
        self.data = self._fetch_market_data(timeframe_id, start_time, end_time)
        self.sequence_length = 100
        self.scaler = MinMaxScaler()
        self._preprocess_data()

    def _fetch_market_data(self, timeframe_id: str, start_time: datetime, end_time: datetime):
        query = """
            SELECT
                open, close, high, low, volume, trades,
                rsi_14, macd_line, macd_signal, macd_histogram,
                bb_upper, bb_middle, bb_lower, atr_14,
                volatility_1h, volatility_24h,
                price_change_1h, price_change_24h,
                volume_change_1h, volume_change_24h
            FROM MarketData
            WHERE timeframe_id = %(timeframe_id)s
            AND open_time BETWEEN %(start_time)s AND %(end_time)s
            AND usable_by_model = true
            ORDER BY open_time ASC
        """

        df = pd.read_sql(
            query,
            self.session.bind,
            params={
                "timeframe_id": timeframe_id,
                "start_time": start_time,
                "end_time": end_time
            }
        )
        return df

    def _preprocess_data(self):
        print("Raw data shape:", self.data.shape)
        print("Missing values:\n", self.data.isna().sum())
        print("Data sample:\n", self.data.head())

        # Fill missing values with forward fill, then backward fill
        self.data = self.data.ffill().bfill()

        # Convert to numpy array
        self.data = self.data.values

        # Scale the data
        self.data = self.scaler.fit_transform(self.data)

    def __len__(self):
        return len(self.data) - self.sequence_length

    def __getitem__(self, idx):
        sequence = self.data[idx:idx + self.sequence_length]
        target = self.data[idx + self.sequence_length, 0]  # Next close price

        return (
            torch.FloatTensor(sequence),
            torch.FloatTensor([target])
        )

    def __del__(self):
        self.session.close()
