-- Enable TimescaleDB extension if not already enabled
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Create sequence for Kline id
CREATE SEQUENCE IF NOT EXISTS kline_id_seq;

-- Enhanced Kline table optimized for TimescaleDB
CREATE TABLE IF NOT EXISTS Kline (
    id BIGINT NOT NULL DEFAULT nextval('kline_id_seq'),
    symbol VARCHAR(20),  -- Fixed length for common trading pairs
    contract_type VARCHAR(10),
    open_time TIMESTAMPTZ NOT NULL,  -- Better for time-series than DECIMAL
    close_time TIMESTAMPTZ NOT NULL,  -- Better for time-series than DECIMAL
    open_price NUMERIC(20,8) NOT NULL,  -- Precise decimal for prices
    high_price NUMERIC(20,8) NOT NULL,
    low_price NUMERIC(20,8) NOT NULL,
    close_price NUMERIC(20,8) NOT NULL,
    volume NUMERIC(30,8) NOT NULL,  -- Large numbers for volume
    base_asset_volume NUMERIC(30,8) NOT NULL,
    number_of_trades BIGINT NOT NULL,  -- Integer instead of DECIMAL
    taker_buy_volume NUMERIC(30,8) NOT NULL,
    taker_buy_base_asset_volume NUMERIC(30,8) NOT NULL,

    -- Price Action Indicators (all DOUBLE PRECISION for faster calculations)
    rsi_14 DOUBLE PRECISION,
    rsi_4 DOUBLE PRECISION,
    stoch_k_14 DOUBLE PRECISION,
    stoch_d_14 DOUBLE PRECISION,
    cci_20 DOUBLE PRECISION,

    -- Trend Indicators
    macd_12_26 DOUBLE PRECISION,
    macd_signal_9 DOUBLE PRECISION,
    macd_histogram DOUBLE PRECISION,
    ema_9 DOUBLE PRECISION,
    ema_20 DOUBLE PRECISION,
    ema_50 DOUBLE PRECISION,
    ema_200 DOUBLE PRECISION,

    -- Volatility Indicators
    bollinger_upper DOUBLE PRECISION,
    bollinger_middle DOUBLE PRECISION,
    bollinger_lower DOUBLE PRECISION,
    atr_14 DOUBLE PRECISION,
    keltner_upper DOUBLE PRECISION,
    keltner_middle DOUBLE PRECISION,
    keltner_lower DOUBLE PRECISION,

    -- Volume Indicators
    obv NUMERIC(30,8),
    mfi_14 DOUBLE PRECISION,
    vwap NUMERIC(20,8),
    cmf_20 DOUBLE PRECISION,

    -- Market Context
    funding_rate NUMERIC(10,8),
    open_interest NUMERIC(30,8),
    long_short_ratio DOUBLE PRECISION,
    cvd NUMERIC(30,8),

    -- Position Management
    current_position VARCHAR(5) CHECK (current_position IN ('LONG', 'SHORT', NULL)),
    position_entry_price NUMERIC(20,8),
    position_size NUMERIC(20,8),
    position_pnl NUMERIC(20,8),
    position_entry_time TIMESTAMPTZ,

    -- Prediction and Performance
    predicted_position VARCHAR(5) CHECK (predicted_position IN ('LONG', 'SHORT', 'HOLD', NULL)),
    prediction_confidence DOUBLE PRECISION CHECK (prediction_confidence >= 0 AND prediction_confidence <= 1),
    actual_profit_loss NUMERIC(20,8),
    trade_executed BOOLEAN DEFAULT FALSE,

    -- Analyzed
    analyzed BOOLEAN DEFAULT FALSE,

    -- Predicted
    predicted BOOLEAN DEFAULT FALSE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Composite primary key including the partitioning column
    CONSTRAINT kline_pkey PRIMARY KEY (id, open_time)
);

-- Set the ownership of the sequence
ALTER SEQUENCE kline_id_seq OWNED BY kline.id;

-- Create hypertable
SELECT create_hypertable('kline', 'open_time',
    chunk_time_interval => INTERVAL '1 day');

-- Create a unique index on symbol and open_time to prevent duplicates
CREATE UNIQUE INDEX idx_kline_unique ON kline(id, symbol, open_time);

-- Optional: Cache sequence values for better insert performance
ALTER SEQUENCE kline_id_seq CACHE 1000;

-- Position tracking table
CREATE TABLE IF NOT EXISTS Positions (
    id BIGSERIAL PRIMARY KEY,
    symbol VARCHAR(20) NOT NULL,
    position_type VARCHAR(5) NOT NULL CHECK (position_type IN ('LONG', 'SHORT')),
    entry_price NUMERIC(20,8) NOT NULL,
    entry_time TIMESTAMPTZ NOT NULL,
    size NUMERIC(20,8) NOT NULL,
    take_profit NUMERIC(20,8),
    stop_loss NUMERIC(20,8),
    exit_price NUMERIC(20,8),
    exit_time TIMESTAMPTZ,
    pnl NUMERIC(20,8),
    status VARCHAR(6) NOT NULL CHECK (status IN ('OPEN', 'CLOSED')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Performance metrics table
CREATE TABLE IF NOT EXISTS ModelPerformance (
    id BIGSERIAL PRIMARY KEY,
    model_version VARCHAR(50) NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    timeframe VARCHAR(10) NOT NULL,
    total_trades INTEGER,
    winning_trades INTEGER,
    losing_trades INTEGER,
    win_rate NUMERIC(5,2),  -- Percentage with 2 decimal places
    avg_profit NUMERIC(10,2),
    max_drawdown NUMERIC(5,2),
    sharpe_ratio NUMERIC(5,2),
    measurement_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Convert ModelPerformance to hypertable for time-series analytics
-- SELECT create_hypertable('modelperformance', 'measurement_time');

-- Optimized indexes for common queries
CREATE INDEX idx_kline_id ON kline(id, open_time DESC);
CREATE INDEX idx_kline_analyzed ON kline(analyzed);
CREATE INDEX idx_kline_predicted ON kline(predicted);
CREATE INDEX idx_kline_symbol_open_time ON kline(symbol, open_time DESC);
CREATE INDEX idx_kline_symbol_open_time_and_close_time ON kline(symbol, open_time DESC,close_time DESC);
CREATE INDEX idx_kline_prediction ON kline(predicted_position, prediction_confidence) WHERE predicted_position IS NOT NULL;
CREATE INDEX idx_positions_symbol_status ON positions(symbol, status, entry_time DESC);
CREATE INDEX idx_positions_pnl ON positions(pnl) WHERE status = 'CLOSED';

-- -- Compression policy for old data (optional)
-- SELECT add_compression_policy('kline', INTERVAL '7 days');
--
-- -- Retention policy (optional, adjust based on needs)
-- SELECT add_retention_policy('kline', INTERVAL '1 year');
