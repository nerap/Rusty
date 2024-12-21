#[cfg(test)]
mod tests {
    use neural_trading_bot::neural_network::{NeuralNetwork, Neuron};

    #[test]
    fn test_neuron_initialization() {
        let neuron = Neuron::new(3);
        assert_eq!(neuron.weights.len(), 3);
        assert!(neuron.bias >= -1.0 && neuron.bias <= 1.0);
    }

    #[test]
    fn test_forward_propagation() {
        let mut nn = NeuralNetwork::new(&[2, 2, 1], 0.1);
        let input = vec![0.5, 0.5];
        let output = nn.forward(&input);
        assert_eq!(output.len(), 1);
        assert!(output[0] >= 0.0 && output[0] <= 1.0);
    }

    #[test]
    fn test_training() {
        let mut nn = NeuralNetwork::new(&[2, 2, 1], 0.1);
        let input = vec![0.5, 0.5];
