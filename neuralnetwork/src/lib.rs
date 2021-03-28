use std::ops::Add;
use std::f64::consts::E;
use knarkzel::prelude::*;

#[derive(Debug, Default, Clone)]
struct Node {
    data: f64,
    bias: f64,
    weights: Vec<f64>,
}

impl Add for Node {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            data: 0.0,
            bias: (self.bias + other.bias) / 2.0,
            weights: self.weights,
        }
    }
}

impl Node {
    fn weighted(&self, weight_index: usize) -> f64 {
        self.data * self.weights[weight_index] + self.bias
    }
}

#[derive(Debug, Clone)]
struct Layer {
    nodes: Vec<Node>,
}

#[derive(Debug, Clone)]
pub struct NeuralNetwork {
    layers: Vec<Layer>,
}

impl NeuralNetwork {
    fn get(&mut self, layer: usize, node: usize) -> &mut Node {
        &mut self.layers[layer].nodes[node]
    }

    pub fn new(structure: &[usize]) -> Self {
        let mut layers = vec![];
        let mut random = Random::new();

        for layer in structure.windows(2) {
            let amount_nodes = layer[0];
            let amount_weights = layer[1];

            let nodes = (0..amount_nodes)
                .map(|_| {
                    let data = 0.0;
                    let bias = random.rand_range_float(-1.0..1.0);
                    let weights = (0..amount_weights)
                        .map(|_| random.rand_range_float(-1.0..1.0))
                        .collect_vec();

                    Node {
                        data,
                        bias,
                        weights,
                    }
                })
                .collect_vec();

            layers.push(Layer { nodes });
        }

        let nodes = vec![Node::default(); *structure.last().unwrap()];
        let output_layer = Layer { nodes };
        layers.push(output_layer);

        NeuralNetwork { layers }
    }

    pub fn process(&mut self, input: &[f64]) {
        for (i, data) in input.into_iter().enumerate() {
            if let Some(node) = self.layers[0].nodes.get_mut(i) {
                node.data = *data;
            }
        }

        // i -> current layer
        for i in 1..self.layers.len() {
            // j -> current node of current layer
            for j in 0..self.layers[i].nodes.len() {
                let sum = self.layers[i - 1]
                    .nodes
                    .iter()
                    .map(|node| node.weighted(j))
                    .sum();
                self.layers[i].nodes[j].data = sigmoid(sum);
            }
        }
    }

    pub fn output(&self) -> Vec<f64> {
        self.layers
            .last()
            .unwrap()
            .nodes
            .iter()
            .map(|node| node.data)
            .collect_vec()
    }

    pub fn mutate(&mut self) {
        let mut random = Random::new();
        for layer in 0..self.layers.len() {
            for node in 0..self.layers[layer].nodes.len() {
                if random.rand_range(0..100) > 80 {
                    let node = self.get(layer, node);
                    node.bias *= random.rand_range_float(0.5..1.0);
                    node.bias += random.rand_range_float(0.0..0.1);
                    for weight in node.weights.iter_mut() {
                        *weight *= random.rand_range_float(0.5..1.0);
                    }
                }
            }
        }
    }

    pub fn crossover(&self, other: &Self) -> Self {
        let mut new_network = NeuralNetwork { layers: vec![] };
        for i in 0..self.layers.len() {
            let first = self.layers[i].nodes.iter();
            let second = other.layers[i].nodes.iter();
            let mut nodes = vec![];
            for (x, y) in first.zip(second) {
                let node = x.clone() + y.clone();
                nodes.push(node)
            }
            new_network.layers.push(Layer { nodes });
        }
        new_network
    }
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + E.powf(-x))
}

