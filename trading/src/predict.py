from src.models.ensemble import TimeseriesEnsemble
import torch
import numpy as np

last_15m_candle = {
    'open': 20450.50,
    'high': 8890.00,
    'low': 8000.25,
    'close': 8750.75,
    'volume': 11250.45,
    'trades': 3567,
    'rsi_14': 58.43,
    'macd_line': 145.32,
    'macd_signal': 132.45,
    'macd_histogram': 12.87,
    'bb_upper': 93100.25,
    'bb_middle': 92500.50,
    'bb_lower': 91900.75,
    'atr_14': 285.50,
    'volatility_1h': 0.85,
    'volatility_24h': 1.25,
    'price_change_1h': 0.45,
    'price_change_24h': 1.85,
    'volume_change_1h': 5.25,
    'volume_change_24h': 12.75
}

last_1h_candle = {
    'open': 20450.50,
    'high': 20450.50,
    'low': 20450.50,
    'close': 20450.50,
    'volume': 4850.78,
    'trades': 15234,
    'rsi_14': 62.15,
    'macd_line': 235.45,
    'macd_signal': 210.32,
    'macd_histogram': 25.13,
    'bb_upper': 93500.25,
    'bb_middle': 92600.50,
    'bb_lower': 91700.75,
    'atr_14': 450.25,
    'volatility_1h': 0.95,
    'volatility_24h': 1.45,
    'price_change_1h': 0.85,
    'price_change_24h': 2.15,
    'volume_change_1h': 8.45,
    'volume_change_24h': 15.85
}

def predict_with_sample():
    # Convert dictionaries to numpy arrays
    data_15m = np.array([
        last_15m_candle['open'], last_15m_candle['close'],
        last_15m_candle['high'], last_15m_candle['low'],
        last_15m_candle['volume'], last_15m_candle['trades'],
        last_15m_candle['rsi_14'], last_15m_candle['macd_line'],
        last_15m_candle['macd_signal'], last_15m_candle['macd_histogram'],
        last_15m_candle['bb_upper'], last_15m_candle['bb_middle'],
        last_15m_candle['bb_lower'], last_15m_candle['atr_14'],
        last_15m_candle['volatility_1h'], last_15m_candle['volatility_24h'],
        last_15m_candle['price_change_1h'], last_15m_candle['price_change_24h'],
        last_15m_candle['volume_change_1h'], last_15m_candle['volume_change_24h']
    ]).reshape(1, 20)  # Reshape to match model input

    data_1h = np.array([
        last_1h_candle['open'], last_1h_candle['close'],
        last_1h_candle['high'], last_1h_candle['low'],
        last_1h_candle['volume'], last_1h_candle['trades'],
        last_1h_candle['rsi_14'], last_1h_candle['macd_line'],
        last_1h_candle['macd_signal'], last_1h_candle['macd_histogram'],
        last_1h_candle['bb_upper'], last_1h_candle['bb_middle'],
        last_1h_candle['bb_lower'], last_1h_candle['atr_14'],
        last_1h_candle['volatility_1h'], last_1h_candle['volatility_24h'],
        last_1h_candle['price_change_1h'], last_1h_candle['price_change_24h'],
        last_1h_candle['volume_change_1h'], last_1h_candle['volume_change_24h']
    ]).reshape(1, 20)

    # Convert to PyTorch tensors
    input_15m = torch.FloatTensor(data_15m)
    input_1h = torch.FloatTensor(data_1h)

    # Load models
    model_15m = TimeseriesEnsemble(input_size=20, hidden_size=128)
    model_1h = TimeseriesEnsemble(input_size=20, hidden_size=128)

    model_15m.load_state_dict(torch.load('models/saved/epoch_0_15m.pth'))
    model_1h.load_state_dict(torch.load('models/saved/epoch_0_1h.pth'))

    model_15m.eval()
    model_1h.eval()

    with torch.no_grad():
        prob_15m = torch.softmax(model_15m(input_15m), dim=1)[0]
        prob_1h = torch.softmax(model_1h(input_1h), dim=1)[0]

        combined_probs = {
            'long': (0.4 * prob_15m[0] + 0.6 * prob_1h[0]).item(),
            'short': (0.4 * prob_15m[1] + 0.6 * prob_1h[1]).item(),
            'hold': (0.4 * prob_15m[2] + 0.6 * prob_1h[2]).item()
        }

        signal = 'BUY' if combined_probs['long'] > 0.6 else \
            'SELL' if combined_probs['short'] > 0.6 else 'HOLD'

        return {
            'signal': signal,
            'probabilities': combined_probs,
            'current_price': last_15m_candle['close']
        }


if __name__ == "__main__":
    result = predict_with_sample()
    print(f"\nPrediction Results:")
    print(f"Signal: {result['signal']}")
    print(f"Current Price: ${result['current_price']:.2f}")
    print(f"Probabilities:")
    print(f"  Long: {result['probabilities']['long']:.2%}")
    print(f"  Short: {result['probabilities']['short']:.2%}")
    print(f"  Hold: {result['probabilities']['hold']:.2%}")
