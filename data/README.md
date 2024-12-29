# Binance Futures Market Data Service

A high-performance market data collection and analysis service for Binance Futures, built with Rust and TimescaleDB.

## Features

- **Efficient Data Collection**: Built with Rust for reliable data fetching
- **Multi-Timeframe Support**: Configurable timeframe collection (e.g., 15m, 1h)
- **Technical Analysis**: Comprehensive suite of technical indicators
- **TimescaleDB Integration**: Optimized time-series data storage
- **Concurrent Processing**: Efficient handling of multiple symbol/timeframe combinations

## System Architecture

### Data Layer
- **TimescaleDB**: Optimized for time-series data with hypertables
- **Schema Design**: Efficient storage of market data and technical indicators
- **Indices**: Optimized for high-frequency querying

### Market Data Processing
- **Data Fetching**: Continuous market data collection from Binance Futures
- **Technical Indicators**: Calculation of common market indicators:
  - RSI, MACD, Bollinger Bands
  - ATR, Volatility metrics
  - Volume analysis
  - Market microstructure imbalance

## Getting Started

### Prerequisites
- Rust (latest stable version)
- Docker and Docker Compose
- TimescaleDB

### Environment Setup

1. Clone the repository

2. Set up environment variables:
```bash
DB_HOST=timescaledb
DB_USER=admin
DB_PASSWORD=admin
DB_NAME=rusty
DB_PORT=5432
```

3. Start the services:
```bash
docker-compose up -d
```

### Configuration

Configuration is managed through `configuration.yaml`:

```yaml
data:
  lookback_days: 3000  # Historical data fetch period
  pairs:
    - symbol: "BTCUSDT"
      contract_type: "PERPETUAL"
      timeframes:
        - interval: "15m"
        - interval: "1h"
```

## Database Schema

### Tables
- `Timeframes`: Manages different data collection intervals
- `MarketData`: Stores OHLCV and calculated indicators

### Features
- Hypertables for efficient time-series operations
- Optimized indices for high-frequency queries
- Built-in technical analysis storage

## Technical Analysis

The system calculates and stores various technical indicators:

- **Core Indicators**
  - RSI (14 periods)
  - MACD (12, 26, 9)
  - Bollinger Bands (20, 2)

- **Volatility Metrics**
  - ATR (14 periods)
  - 1h/24h volatility

- **Volume Analysis**
  - Volume changes
  - Market depth imbalance

## Performance Optimization

- Concurrent task handling with semaphore-based rate limiting
- Efficient data fetching with retry mechanisms
- Optimized database queries with proper indexing
- Memory-efficient data processing

## Error Handling

The system implements comprehensive error handling:
- Rate limiting management
- Network retry logic
- Database transaction management
- Graceful shutdown handling

## Monitoring and Logging

- Structured logging with tracing
- Database health monitoring
- API rate limit tracking
- Performance metrics collection

## Key Components

### Market Data Fetcher
- Handles data retrieval from Binance Futures API
- Manages API rate limits
- Implements retry mechanisms
- Supports historical and real-time data collection

### Market Data Analyzer
- Processes raw market data
- Calculates technical indicators
- Updates database with analyzed data

### Configuration Service
- Manages system configuration
- Validates input parameters
- Handles timeframe settings

## Contributing

Please read CONTRIBUTING.md for details on our code of conduct and the process for submitting pull requests.

## License

This project is licensed under the MIT License - see the LICENSE.md file for details

## Acknowledgments

- Binance Futures API
- TimescaleDB Team
- Rust Community
