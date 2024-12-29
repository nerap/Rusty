# Cryptocurrency Futures Trading System

A high-performance, neural network-based trading system for cryptocurrency futures markets. The system utilizes an ensemble of LSTM, CNN, and DNN models with dual timeframe analysis (15m/1h) for optimal trade execution.

## Features

- **Ensemble Neural Network Architecture**
  - LSTM for sequence learning and temporal patterns
  - CNN for pattern recognition and visual features
  - DNN for technical indicator processing
  - Advanced feature fusion for final decision making

- **Dual Timeframe Analysis**
  - 15-minute timeframe for tactical entries
  - 1-hour timeframe for strategic direction
  - Cross-timeframe confirmation logic

- **Technical Analysis Suite**
  - RSI, MACD, Bollinger Bands, ATR
  - Volatility metrics (1h and 24h windows)
  - Volume profiling and depth imbalance analysis
  - Market regime identification
  - Support/resistance detection

- **High-Performance Database**
  - TimescaleDB for time-series optimization
  - Hypertables for efficient data management
  - Optimized indices for HFT operations
  - Real-time market data processing

## Prerequisites

- Rust (latest stable version)
- Docker and Docker Compose
- TimescaleDB
- CUDA-compatible GPU (recommended for model training)

## Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/crypto-trading-system.git
cd crypto-trading-system
```

2. Set up the environment:
```bash
cp .env.example .env
# Edit .env with your configuration
```

3. Start the database:
```bash
docker-compose up -d timescaledb
```

4. Initialize the database schema:
```bash
psql -h localhost -U admin -d rusty -f init_schema.sql
```

5. Build and run the system:
```bash
docker-compose up -d analyzer
```

## Configuration

### Basic Configuration (configuration.yaml)
```yaml
data:
  lookback_days: 3000  # Historical data for training
  pairs:
    - symbol: "BTCUSDT"
      contract_type: "PERPETUAL"
      timeframes:
        - interval: "15m"
        - interval: "1h"
```

### Model Configuration
- Training parameters in `config/model_config.yaml`
- Risk management settings in `config/risk_config.yaml`
- Position sizing rules in `config/position_config.yaml`

## System Architecture

### Data Pipeline
1. Market Data Collection
2. Feature Engineering
3. Model Prediction
4. Signal Generation
5. Position Management

### Neural Network Ensemble
```
Input Data → [LSTM Branch]     → Feature   → Ensemble → Position
          → [CNN Branch]      → Fusion    → Decision → Management
          → [DNN Branch]      → Layer     → Layer   → System
```

### Database Schema
- MarketData: Raw market data and indicators
- ModelPredictions: Neural network outputs
- TradingSignals: Generated trading signals
- Positions: Active and historical positions
- PositionRisk: Risk management metrics
- ModelPerformance: Performance tracking

## Usage

### Starting the System
```bash
cargo run -- --config configuration.yaml --init true
```

### Monitoring
- Trading dashboard: http://localhost:3000
- Metrics dashboard: http://localhost:9090
- Log monitoring: http://localhost:5601

### Backesting
```bash
cargo run --bin backtester -- --config backtest_config.yaml
```

## Model Training

### Data Preparation
```bash
cargo run --bin prepare_data -- --timeframe 15m,1h --symbols BTCUSDT
```

### Training the Model
```bash
cargo run --bin train_model -- --model-config config/model_config.yaml
```

### Hyperparameter Optimization
```bash
cargo run --bin optimize_hyperparams -- --config config/optimization.yaml
```

## Performance Metrics

The system tracks various performance metrics:

### Trading Metrics
- Sharpe Ratio
- Maximum Drawdown
- Win Rate
- Profit Factor
- Average Holding Time

### Model Metrics
- Classification Accuracy
- Precision and Recall
- F1 Score
- Confusion Matrix

## Risk Management

The system implements multiple layers of risk management:

1. **Position Level**
   - Dynamic position sizing
   - Adaptive stop-loss
   - Take-profit optimization
   - Maximum position duration

2. **Portfolio Level**
   - Maximum drawdown protection
   - Exposure limits
   - Correlation management
   - Daily loss limits

3. **System Level**
   - Model drift detection
   - Market regime detection
   - Emergency shutdown protocols
   - Performance monitoring

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

## Acknowledgments

- TimescaleDB team for the high-performance database
- Binance for the market data API
- Rust community for excellent tools and libraries

## Disclaimer

This software is for educational purposes only. Cryptocurrency trading carries significant risks. Use at your own risk.
