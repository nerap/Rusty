CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
SET TIME ZONE 'UTC';

CREATE TYPE ContractType AS ENUM ('perpetual', 'current_quarter', 'next_quarter');

CREATE TABLE Timeframes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    symbol VARCHAR(20) NOT NULL,
    contract_type ContractType NOT NULL,
    interval_minutes INTEGER NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,

    UNIQUE (symbol, contract_type, interval_minutes)
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
    rsi_14 DECIMAL(20,8),
    macd_line DECIMAL(20,8),
    macd_signal DECIMAL(20,8),
    macd_histogram DECIMAL(20,8),
    bb_upper DECIMAL(24,8),
    bb_middle DECIMAL(24,8),
    bb_lower DECIMAL(24,8),
    atr_14 DECIMAL(20,8),

    -- Market microstructure
    depth_imbalance DECIMAL(20,8),

    -- Volatility metrics
    volatility_1h DECIMAL(20,8),
    volatility_24h DECIMAL(20,8),

    -- Price changes
    price_change_1h DECIMAL(20,8),
    price_change_24h DECIMAL(20,8),

    -- Trading volume changes
    volume_change_1h DECIMAL(20,8),
    volume_change_24h DECIMAL(20,8),

    -- Analyzed
    analyzed BOOLEAN DEFAULT FALSE,

    -- Usable
    usable_by_model BOOLEAN DEFAULT FALSE,

    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,

    UNIQUE (open_time, timeframe_id)
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
