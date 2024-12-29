# Rusy

A cryptocurrency futures trading system, featuring dual timeframe analysis, ensemble machine learning models, and HFT-optimized data storage.

## System Architecture

The system is split into two main services:

- [Data Service](https://github.com/yourusername/crypto-trading/tree/main/data-service) - Market data collection, processing, and storage
- [Trading Service](https://github.com/yourusername/crypto-trading/tree/main/trading-service) - Trading strategy execution and position management

### Data Service Features

- Real-time market data collection from Binance Futures
- Dual timeframe analysis (15m/1h)
- Comprehensive technical analysis suite
- HFT-optimized TimescaleDB storage
- Efficient data preprocessing pipeline
- Rate limiting and error handling
- Automatic data recovery and validation

### Trading Service Features

- Ensemble machine learning model (LSTM + CNN + DNN)
- Position management and risk controls
- Real-time signal generation
- Advanced order execution
- Performance monitoring
- Automated trading strategies

## Technical Stack

- **Backend:** Rust
- **Database:** TimescaleDB (PostgreSQL)
- **Machine Learning:** TensorFlow/PyTorch
- **Infrastructure:** Docker, Docker Compose
- **API Integration:** Binance Futures API

## Prerequisites

- Rust (latest stable version)
- Docker and Docker Compose
- TimescaleDB
- PostgreSQL client

## Quick Start

1. Clone the repository:
```bash
git clone https://github.com/yourusername/rusty.git
cd rusty/data
```

2. Set up environment variables:
```bash
cp .env.example .env
# Edit .env with your configuration
```

3. Start the services:
```bash
docker-compose up -d --build
```

## Configuration

The system can be configured through `configuration.yaml`:

```yaml
data:
  lookback_days: 3000 // The max is 2019-06 so it's to fetch the oldest possible
  pairs:
    - symbol: "BTCUSDT"
      contract_type: "PERPETUAL"
      timeframes:
        - interval: "15m"
        - interval: "1h"
```

## Database Schema

The system uses a highly optimized TimescaleDB schema designed for HFT operations:

- Hypertables for time-series data
- Optimized indices for quick lookups
- Efficient data partitioning
- Automated data retention policies

Key tables:
- MarketData (OHLCV data)
- Timeframes (Time interval configurations)
- Positions (Trade management)
- ModelPredictions (ML model outputs)

## Neural Network Architecture

The trading system employs an ensemble approach combining:

1. LSTM Network:
   - Input: Technical indicators, price action
   - Architecture: 3 LSTM layers with dropout
   - Output: Price direction probability

2. CNN Network:
   - Input: Price action patterns
   - Architecture: 2D convolutions for pattern recognition
   - Output: Pattern classification

3. DNN Network:
   - Input: Market microstructure features
   - Architecture: Dense layers with batch normalization
   - Output: Trade signal strength

The ensemble combines these outputs using a weighted voting system.

## Development

### Building from Source

```bash
# Build data service
cd data-service
cargo build --release

# Build trading service
cd ../trading-service
cargo build --release
```

### Running Tests

```bash
cargo test --all-features
```

## Deployment

The system is containerized and can be deployed using Docker Compose:

```bash
docker-compose -f docker-compose.prod.yml up -d
```

## Monitoring

- Log monitoring through Docker logs
- Metrics collection via Prometheus
- Trading performance dashboard in Grafana
- Real-time alerts and notifications

## Contributing

1. Fork the repository
2. Create your feature branch
3. Commit your changes
4. Push to the branch
5. Create a new Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Disclaimer

This software is for educational purposes only. Cryptocurrency trading carries significant risks. Always perform your own research and risk assessment.

## Acknowledgments

- Binance for their robust API
- TimescaleDB for high-performance time-series database
- Rust community for excellent crates and support
