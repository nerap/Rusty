use rand::Rng;

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

fn sigmoid_derivative(x: f64) -> f64 {
    let sx = sigmoid(x);
    sx * (1.0 - sx)
}


#[derive(Debug)]
pub struct Neuron {
    pub weights: Vec<f64>,
    pub bias: f64,
    pub last_activation: f64,
    pub last_input: Vec<f64>,
}

impl Neuron {
    pub fn new(num_inputs: usize) -> Self {
        let mut rng = rand::thread_rng();
        let weights: Vec<f64> = (0..num_inputs)
            .map(|_| rng.gen_range(-1.0..1.0))
            .collect();

        Neuron {
            weights,
            bias: rng.gen_range(-1.0..1.0),
            last_activation: 0.0,
            last_input: vec![0.0; num_inputs],
        }
    }

    pub fn forward(&mut self, inputs: &[f64]) -> f64 {
        assert_eq!(inputs.len(), self.weights.len());
        self.last_input = inputs.to_vec();

        let sum: f64 = inputs
            .iter()
            .zip(self.weights.iter())
            .map(|(input, weight)| input * weight)
            .sum();

        self.last_activation = sigmoid(sum + self.bias);
        self.last_activation
    }
}

#[derive(Debug)]
pub struct Layer {
    pub neurons: Vec<Neuron>,
}

impl Layer {
    fn new(num_neurons: usize, num_inputs: usize) -> Self {
        let neurons = (0..num_neurons)
            .map(|_| Neuron::new(num_inputs))
            .collect();

        Layer { neurons }
    }

    fn forward(&mut self, inputs: &[f64]) -> Vec<f64> {
        self.neurons
            .iter_mut()
            .map(|neuron| neuron.forward(inputs))
            .collect()
    }
}

pub struct NeuralNetwork {
    layers: Vec<Layer>,
    learning_rate: f64,
}

impl NeuralNetwork {
    fn new(layer_sizes: &[usize], learning_rate: f64) -> Self {
        assert!(layer_sizes.len() >= 2);

        let mut layers = Vec::new();
        for i in 0..layer_sizes.len() - 1 {
            layers.push(Layer::new(layer_sizes[i + 1], layer_sizes[i]));
        }

        NeuralNetwork {
            layers,
            learning_rate,
        }
    }

    fn forward(&mut self, inputs: &[f64]) -> Vec<f64> {
        let mut current_outputs = inputs.to_vec();

        for layer in &mut self.layers {
            current_outputs = layer.forward(&current_outputs);
        }

        current_outputs
    }

    // Simple backpropagation implementation
    fn train(&mut self, inputs: &[f64], targets: &[f64]) {
        // Forward pass
        let outputs = self.forward(inputs);

        // Calculate output layer error
        let mut layer_errors = vec![
            outputs
                .iter()
                .zip(targets.iter())
                .map(|(output, target)| (target - output) * output * (1.0 - output))
                .collect::<Vec<f64>>()
        ];

        // Backpropagate error
        for layer_idx in (0..self.layers.len()).rev() {
            let layer = &mut self.layers[layer_idx];
            let errors = &layer_errors[0];

            // Update weights and biases
            for (neuron_idx, neuron) in layer.neurons.iter_mut().enumerate() {
                let error = errors[neuron_idx];

                // Update weights
                for (weight_idx, weight) in neuron.weights.iter_mut().enumerate() {
                    let input = neuron.last_input[weight_idx];
                    *weight += self.learning_rate * error * input;
                }

                // Update bias
                neuron.bias += self.learning_rate * error;
            }

            // Calculate error for next layer back if not at input layer
            if layer_idx > 0 {
                let prev_layer = &self.layers[layer_idx - 1];
                let mut next_errors = vec![0.0; prev_layer.neurons.len()];

                for (neuron_idx, neuron) in layer.neurons.iter().enumerate() {
                    let error = errors[neuron_idx];
                    for (weight_idx, weight) in neuron.weights.iter().enumerate() {
                        next_errors[weight_idx] += error * weight;
                    }
                }

                layer_errors[0] = next_errors;
            }
        }
    }
}
