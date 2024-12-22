use chrono::{DateTime, Datelike, Timelike, Utc};
use rand::Rng;
use std::f64;

#[derive(Debug, Clone)]
struct Position {
    position_type: PositionType,
    stop_loss: f64,
    take_profit: f64,
    entry_price: f64,
    entry_time: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
enum PositionType {
    Long,
    Short,
    None,
}

// Input data structure
#[derive(Debug)]
struct InputData {
    timestamp: DateTime<Utc>,
    price: f64,
    active_position: Option<Position>,
}

impl InputData {
    fn to_normalized_vec(&self) -> Vec<f64> {
        let timestamp_features = self.extract_time_features();
        let position_features = match &self.active_position {
            Some(pos) => {
                let position_type_value = match pos.position_type {
                    PositionType::Long => 1.0,
                    PositionType::Short => -1.0,
                    PositionType::None => 0.0,
                };
                let price_diff = (self.price - pos.entry_price) / pos.entry_price;
                let time_diff =
                    (self.timestamp - pos.entry_time).num_minutes() as f64 / (24.0 * 60.0); // Normalized to days
                vec![1.0, position_type_value, price_diff, time_diff]
            }
            None => vec![0.0, 0.0, 0.0, 0.0],
        };

        [timestamp_features, vec![self.price], position_features].concat()
    }

    fn extract_time_features(&self) -> Vec<f64> {
        let hour = self.timestamp.hour() as f64 / 24.0;
        let minute = self.timestamp.minute() as f64 / 60.0;
        let day_of_week = self.timestamp.weekday().num_days_from_monday() as f64 / 7.0;
        let day_of_month = self.timestamp.day() as f64 / 31.0;
        let month = self.timestamp.month() as f64 / 12.0;

        vec![hour, minute, day_of_week, day_of_month, month]
    }
}

struct NeuralNetwork {
    input_size: usize,
    hidden_size: usize,
    output_size: usize,
    weights1: Vec<Vec<f64>>,
    weights2: Vec<Vec<f64>>,
    bias1: Vec<f64>,
    bias2: Vec<f64>,
}

impl NeuralNetwork {
    fn new(hidden_size: usize) -> Self {
        let input_size = 10; // 5 time features + price + 4 position features
        let output_size = 3; // position type, stop loss, take profit
        let mut rng = rand::thread_rng();

        let weights1 = (0..hidden_size)
            .map(|_| (0..input_size).map(|_| rng.gen_range(-1.0..1.0)).collect())
            .collect();

        let weights2 = (0..output_size)
            .map(|_| (0..hidden_size).map(|_| rng.gen_range(-1.0..1.0)).collect())
            .collect();

        let bias1 = (0..hidden_size).map(|_| rng.gen_range(-1.0..1.0)).collect();

        let bias2 = (0..output_size).map(|_| rng.gen_range(-1.0..1.0)).collect();

        NeuralNetwork {
            input_size,
            hidden_size,
            output_size,
            weights1,
            weights2,
            bias1,
            bias2,
        }
    }

    fn sigmoid(x: f64) -> f64 {
        1.0 / (1.0 + (-x).exp())
    }

    fn sigmoid_derivative(x: f64) -> f64 {
        let sx = Self::sigmoid(x);
        sx * (1.0 - sx)
    }

    fn forward(&self, input: &[f64]) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let mut hidden = vec![0.0; self.hidden_size];
        for i in 0..self.hidden_size {
            let mut sum = self.bias1[i];
            for j in 0..self.input_size {
                sum += input[j] * self.weights1[i][j];
            }
            hidden[i] = Self::sigmoid(sum);
        }

        let mut output = vec![0.0; self.output_size];
        for i in 0..self.output_size {
            let mut sum = self.bias2[i];
            for j in 0..self.hidden_size {
                sum += hidden[j] * self.weights2[i][j];
            }
            output[i] = Self::sigmoid(sum);
        }

        (input.to_vec(), hidden, output)
    }

    fn backprop(
        &mut self,
        input: &[f64],
        hidden: &[f64],
        output: &[f64],
        target: &[f64],
        learning_rate: f64,
    ) {
        // Output layer error
        let mut output_delta = vec![0.0; self.output_size];
        for i in 0..self.output_size {
            let error = target[i] - output[i];
            output_delta[i] = error * Self::sigmoid_derivative(output[i]);
        }

        // Hidden layer error
        let mut hidden_delta = vec![0.0; self.hidden_size];
        for i in 0..self.hidden_size {
            let mut error = 0.0;
            for j in 0..self.output_size {
                error += output_delta[j] * self.weights2[j][i];
            }
            hidden_delta[i] = error * Self::sigmoid_derivative(hidden[i]);
        }

        // Update weights and biases
        for i in 0..self.output_size {
            for j in 0..self.hidden_size {
                self.weights2[i][j] += learning_rate * output_delta[i] * hidden[j];
            }
            self.bias2[i] += learning_rate * output_delta[i];
        }

        for i in 0..self.hidden_size {
            for j in 0..self.input_size {
                self.weights1[i][j] += learning_rate * hidden_delta[i] * input[j];
            }
            self.bias1[i] += learning_rate * hidden_delta[i];
        }
    }

    fn train(
        &mut self,
        training_data: &[InputData],
        targets: &[Vec<f64>],
        epochs: usize,
        learning_rate: f64,
    ) {
        for epoch in 0..epochs {
            let mut total_error = 0.0;

            for (input_data, target) in training_data.iter().zip(targets.iter()) {
                let normalized_input = input_data.to_normalized_vec();
                let (input_layer, hidden_layer, output_layer) = self.forward(&normalized_input);

                for (output, target) in output_layer.iter().zip(target.iter()) {
                    total_error += (target - output).powi(2);
                }

                self.backprop(
                    &input_layer,
                    &hidden_layer,
                    &output_layer,
                    target,
                    learning_rate,
                );
            }

            if epoch % 100 == 0 {
                println!(
                    "Epoch {}: Error = {}",
                    epoch,
                    total_error / training_data.len() as f64
                );
            }
        }
    }

    fn predict(&self, input_data: &InputData) -> Position {
        let normalized_input = input_data.to_normalized_vec();
        let (_, _, output) = self.forward(&normalized_input);

        let position_type = match output[0] {
            x if x > 0.66 => PositionType::Long,
            x if x < 0.33 => PositionType::Short,
            _ => PositionType::None,
        };

        let stop_loss = output[1] * 5.0; // Max 5% stop loss
        let take_profit = output[2] * 10.0; // Max 10% take profit

        Position {
            position_type,
            stop_loss,
            take_profit,
            entry_price: input_data.price,
            entry_time: input_data.timestamp,
        }
    }
    fn print_network_state(&self, input_data: &InputData) {
        let normalized_input = input_data.to_normalized_vec();
        let (_, hidden, output) = self.forward(&normalized_input);

        // Calculate the maximum width needed for numbers
        let max_width = 8;

        // Print header
        println!("\n{}", "=".repeat(80));
        println!("Neural Network State");
        println!("{}", "=".repeat(80));

        // Print input layer
        println!("\nInput Layer:");
        println!("{}", "-".repeat(80));
        println!("Time features:");
        println!("  Hour:        {:>8.3}", normalized_input[0]);
        println!("  Minute:      {:>8.3}", normalized_input[1]);
        println!("  Day of Week: {:>8.3}", normalized_input[2]);
        println!("  Day of Month:{:>8.3}", normalized_input[3]);
        println!("  Month:       {:>8.3}", normalized_input[4]);
        println!("Price:         {:>8.3}", normalized_input[5]);
        println!("Position features:");
        println!("  Active:      {:>8.3}", normalized_input[6]);
        println!("  Type:        {:>8.3}", normalized_input[7]);
        println!("  Price Diff:  {:>8.3}", normalized_input[8]);
        println!("  Time Diff:   {:>8.3}", normalized_input[9]);

        // Print weights and biases for first hidden layer
        println!("\nWeights (Input → Hidden) [showing first 3 neurons]:");
        println!("{}", "-".repeat(80));
        for i in 0..(self.hidden_size.min(3)) {
            print!("Neuron {}: ", i);
            for j in 0..self.input_size {
                print!("{:>8.3} ", self.weights1[i][j]);
            }
            println!("\n          Bias: {:>8.3}", self.bias1[i]);
        }
        if self.hidden_size > 3 {
            println!("... {} more neurons ...", self.hidden_size - 3);
        }

        // Print hidden layer values
        println!("\nHidden Layer Values [showing first 5 neurons]:");
        println!("{}", "-".repeat(80));
        print!("Values:  ");
        for i in 0..(self.hidden_size.min(3)) {
            print!("{:>8.3} ", hidden[i]);
        }
        if self.hidden_size > 5 {
            print!("...");
        }
        println!();

        // Print weights and biases for output layer
        println!("\nWeights (Hidden → Output):");
        println!("{}", "-".repeat(80));
        for i in 0..self.output_size {
            print!("Output {}: ", i);
            for j in 0..(self.hidden_size.min(5)) {
                print!("{:>8.3} ", self.weights2[i][j]);
            }
            if self.hidden_size > 5 {
                print!("...");
            }
            println!("\n          Bias: {:>8.3}", self.bias2[i]);
        }

        // Print output layer
        println!("\nOutput Layer:");
        println!("{}", "-".repeat(80));
        println!(
            "Position Type: {:>8.3} -> {}",
            output[0],
            match output[0] {
                x if x > 0.66 => "LONG",
                x if x < 0.33 => "SHORT",
                _ => "NONE",
            }
        );
        println!("Stop Loss:    {:>8.3}% of position", output[1] * 5.0);
        println!("Take Profit:  {:>8.3}% of position", output[2] * 10.0);

        // Print network architecture summary
        println!("\nNetwork Architecture:");
        println!("{}", "-".repeat(80));
        println!("Input Neurons:  {}", self.input_size);
        println!("Hidden Neurons: {}", self.hidden_size);
        println!("Output Neurons: {}", self.output_size);
        println!("{}", "=".repeat(80));
    }

    // Helper function to create a visual representation of the network
    fn print_network_ascii(&self) {
        println!("\nNetwork Structure:");
        println!("{}", "=".repeat(80));

        // Calculate the maximum number of neurons in any layer
        let max_neurons = self.input_size.max(self.hidden_size.max(self.output_size));

        // Print each layer
        for row in 0..max_neurons {
            // Input layer
            if row < self.input_size {
                print!("(I{:2}) ", row);
            } else {
                print!("     ");
            }

            // Connections to hidden layer
            if row < self.input_size && row < self.hidden_size {
                print!("--→ ");
            } else {
                print!("    ");
            }

            // Hidden layer
            if row < self.hidden_size {
                print!("(H{:2}) ", row);
            } else {
                print!("     ");
            }

            // Connections to output layer
            if row < self.hidden_size && row < self.output_size {
                print!("--→ ");
            } else {
                print!("    ");
            }

            // Output layer
            if row < self.output_size {
                print!("(O{:2})", row);
            }

            println!();
        }
        println!("{}", "=".repeat(80));
        println!("Legend:");
        println!("I: Input Neuron   H: Hidden Neuron   O: Output Neuron");
        println!("O0: Position Type (Long/Short/None)");
        println!("O1: Stop Loss Percentage");
        println!("O2: Take Profit Percentage");
    }
}

fn main() {
    // Example usage
    let mut nn = NeuralNetwork::new(10); // 10 neurons in hidden layer

    // Example training data
    let training_data = vec![
        InputData {
            timestamp: Utc::now(),
            price: 100.0,
            active_position: None,
        },
        InputData {
            timestamp: Utc::now(),
            price: 105.0,
            active_position: None,
        },
        InputData {
            timestamp: Utc::now(),
            price: 20.0,
            active_position: None,
        },
        InputData {
            timestamp: Utc::now(),
            price: 50.0,
            active_position: None,
        },
        InputData {
            timestamp: Utc::now(),
            price: 20.0,
            active_position: None,
        },
        InputData {
            timestamp: Utc::now(),
            price: 50.0,
            active_position: None,
        },
        InputData {
            timestamp: Utc::now(),
            price: 120.0,
            active_position: None,
        },
        InputData {
            timestamp: Utc::now(),
            price: 180.0,
            active_position: None,
        },
        InputData {
            timestamp: Utc::now(),
            price: 180.0,
            active_position: None,
        },
        InputData {
            timestamp: Utc::now(),
            price: 20.0,
            active_position: None,
        },
        InputData {
            timestamp: Utc::now(),
            price: 78.0,
            active_position: None,
        },
        InputData {
            timestamp: Utc::now(),
            price: 80.0,
            active_position: None,
        },
    ];

    let training_targets = vec![
        vec![0.5, 0.02, 0.05], // Example target
        vec![0.5, 0.03, 0.06], // Example target
        vec![1.0, 0.03, 0.06], // Example target
        vec![1.0, 0.03, 0.06], // Example target
        vec![1.0, 0.03, 0.06], // Example target
        vec![1.0, 0.03, 0.06], // Example target
        vec![0.0, 0.03, 0.06], // Example target
        vec![0.0, 0.03, 0.06], // Example target
        vec![0.0, 0.03, 0.06], // Example target
        vec![1.0, 0.03, 0.06], // Example target
        vec![1.0, 0.03, 0.06], // Example target
        vec![1.0, 0.03, 0.06], // Example target
    ];

    nn.print_network_ascii();
    // Train the network
    nn.train(&training_data, &training_targets, 1000, 0.1);

    // Make a prediction
    let input = InputData {
        timestamp: Utc::now(),
        price: 20.0,
        active_position: None,
    };

    nn.print_network_state(&input);

    let prediction = nn.predict(&input);
    println!("Prediction: {:?}", prediction);
}
