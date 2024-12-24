CREATE EXTENSION IF NOT EXISTS timescaledb;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE Timeframes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(10) NOT NULL,
    interval_minutes INTEGER NOT NULL,
    is_enabled BOOLEAN DEFAULT true,
    weight DECIMAL(4,3) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,

    UNIQUE (interval_minutes, weight)
);

CREATE TABLE MarketData (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    timeframe_id UUID NOT NULL REFERENCES Timeframes(id),
    symbol VARCHAR(20) NOT NULL,
    contract_type VARCHAR(10) NOT NULL,
    open_time TIMESTAMPTZ NOT NULL,
    close_time TIMESTAMPTZ NOT NULL,
    open DECIMAL(20,8) NOT NULL,
    close DECIMAL(20,8) NOT NULL,
    high DECIMAL(20,8) NOT NULL,
    low DECIMAL(20,8) NOT NULL,
    volume DECIMAL(20,8) NOT NULL,
    trades BIGINT NOT NULL,

    -- Technical indicators
    rsi_14 DECIMAL(10,4),
    macd_line DECIMAL(10,4),
    macd_signal DECIMAL(10,4),
    macd_histogram DECIMAL(10,4),
    bb_upper DECIMAL(24,8),
    bb_middle DECIMAL(24,8),
    bb_lower DECIMAL(24,8),
    atr_14 DECIMAL(10,4),

    -- Market microstructure
    bid_ask_spread DECIMAL(24,8),
    depth_imbalance DECIMAL(10,4),
    funding_rate DECIMAL(10,4),
    open_interest DECIMAL(24,8),
    long_short_ratio DECIMAL(10,4),

    -- Volatility metrics
    volatility_1h DECIMAL(10,4),
    volatility_24h DECIMAL(10,4),

    -- Price changes
    price_change_1h DECIMAL(10,4),
    price_change_24h DECIMAL(10,4),

    -- Trading volume changes
    volume_change_1h DECIMAL(10,4),
    volume_change_24h DECIMAL(10,4),

    -- Analyzed
    analyzed BOOLEAN DEFAULT FALSE,

    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,

    UNIQUE (open_time, timeframe_id, symbol, contract_type)
);

CREATE TABLE Positions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    market_data_id UUID REFERENCES MarketData(id),
    symbol VARCHAR(20) NOT NULL,
    contract_type VARCHAR(10) NOT NULL,
    side VARCHAR(5) NOT NULL,
    size DECIMAL(20,8) NOT NULL,
    entry_price DECIMAL(20,8) NOT NULL,
    take_profit DECIMAL(20,8),
    stop_loss DECIMAL(20,8),
    entry_time TIMESTAMPTZ NOT NULL,
    exit_time TIMESTAMPTZ,
    exit_price DECIMAL(20,8),
    pnl DECIMAL(20,8),
    status VARCHAR(10) NOT NULL DEFAULT 'open',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE ModelPredictions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    market_data_id UUID REFERENCES MarketData(id),
    timeframe_id UUID REFERENCES Timeframes(id),
    lstm_pred DECIMAL(10,4) NOT NULL,
    cnn_pred DECIMAL(10,4) NOT NULL,
    dnn_pred DECIMAL(10,4) NOT NULL,
    ensemble_pred DECIMAL(10,4) NOT NULL,
    confidence DECIMAL(5,4) NOT NULL,
    prediction_time TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- -- Create hypertables
-- SELECT create_hypertable('MarketData', 'open_time');
-- SELECT create_hypertable('ModelPredictions', 'prediction_time');

-- Create indexes with open_time as first column for hypertable compatibility
CREATE UNIQUE INDEX idx_market_data_unique ON MarketData (open_time, symbol, contract_type, timeframe_id);
CREATE INDEX idx_market_data_symbol ON MarketData (open_time DESC, symbol, contract_type);
CREATE INDEX idx_market_data_timeframe ON MarketData (open_time DESC, timeframe_id);
CREATE INDEX idx_market_data_analyzed ON MarketData (analyzed, timeframe_id);
CREATE INDEX idx_positions_symbol ON Positions (symbol, contract_type, status);
CREATE INDEX idx_model_predictions_market ON ModelPredictions (market_data_id, prediction_time DESC);

-- Insert timeframe seeds
INSERT INTO Timeframes (name, interval_minutes, weight) VALUES
    ('15m', 15, 0.4),
    ('1h', 60, 0.6);
