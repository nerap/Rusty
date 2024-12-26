import os
from datetime import datetime, timedelta
import torch
from torch.utils.data import DataLoader
from dotenv import load_dotenv

from src.models.ensemble import TimeseriesEnsemble
from src.data.market_dataset import MarketDataset
from src.services.trainer import DualTimeframeTrainer
from src.config.database import SessionLocal
from src.utils.logger import setup_logger

load_dotenv()
logger = setup_logger()


def main():
    # Initialize models
    model_15m = TimeseriesEnsemble(input_size=20, hidden_size=128)
    model_1h = TimeseriesEnsemble(input_size=20, hidden_size=128)

    # Time ranges for training
    end_time = datetime.utcnow()
    start_time = end_time - timedelta(days=30)

    # Initialize datasets
    dataset_15m = MarketDataset(
        timeframe_id=os.getenv('TIMEFRAME_15M_ID'),
        start_time=start_time,
        end_time=end_time
    )

    dataset_1h = MarketDataset(
        timeframe_id=os.getenv('TIMEFRAME_1H_ID'),
        start_time=start_time,
        end_time=end_time
    )

    # Create data loaders
    train_loader_15m = DataLoader(
        dataset_15m,
        batch_size=32,
        shuffle=True
    )

    train_loader_1h = DataLoader(
        dataset_1h,
        batch_size=32,
        shuffle=True
    )

    # Initialize trainer
    trainer = DualTimeframeTrainer(
        model_15m=model_15m,
        model_1h=model_1h,
        learning_rate=1e-4
    )

    # Training loop
    NUM_EPOCHS = 10
    for epoch in range(NUM_EPOCHS):
        train_loss = trainer.train_epoch(train_loader_15m, train_loader_1h)
        val_loss = trainer.validate(train_loader_15m, train_loader_1h)

        logger.info(f"Epoch {epoch}: Train Loss = {
                    train_loss:.4f}, Val Loss = {val_loss:.4f}")

        # Save models periodically
        if epoch % 10 == 0:
            trainer.save_models(
                os.getenv('MODEL_SAVE_PATH'),
                f"epoch_{epoch}"
            )


if __name__ == "__main__":
    main()
