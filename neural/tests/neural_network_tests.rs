#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neural_network_prediction() {
        let nn = NeuralNetwork::new(4);
        let prediction = nn.predict(100.0);
        assert!(prediction >= -1.0 && prediction <= 1.0);
    }

    #[test]
    fn test_trading_bot_initial_position() {
        let bot = TradingBot::new(4);
        assert_eq!(*bot.get_position(), Position::Neutral);
    }

    #[test]
    fn test_trading_bot_decision() {
        let mut bot = TradingBot::new(4);
        let position = bot.make_decision(100.0);
        assert!(matches!(position, Position::Long | Position::Short | Position::Neutral));
    }
}
