use crate::*;
use knarkzel::prelude::*;
use std::f32::consts::E;

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + E.powf(-x))
}

#[derive(Debug, Default, Clone)]
struct Node {
    data: f32,
    bias: f32,
    weights: Vec<f32>,
}

impl Node {
    fn weighted(&self, weight_index: usize) -> f32 {
        self.data * self.weights[weight_index] + self.bias
    }

    fn add(&self, other: &Self) -> Self {
        let data = 0.0;
        let bias = (self.bias + other.bias) / 2.0;

        let (first, second) = (self.weights.iter(), other.weights.iter());
        let weights = first.zip(second).map(|(x, y)| (x + y) / 2.0).collect_vec();

        Self {
            data,
            bias,
            weights,
        }
    }
}

#[derive(Debug, Clone)]
struct Layer {
    nodes: Vec<Node>,
}

#[derive(Default, Debug, Clone)]
pub struct NeuralNetwork {
    layers: Vec<Layer>,
}

impl NeuralNetwork {
    fn get(&mut self, layer: usize, node: usize) -> &mut Node {
        &mut self.layers[layer].nodes[node]
    }

    fn add_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    pub fn new(structure: &[usize], random: &mut Random) -> Self {
        let mut layers = vec![];
        for layer in structure.windows(2) {
            let amount_nodes = layer[0];
            let amount_weights = layer[1];

            let nodes = (0..amount_nodes)
                .map(|_| {
                    let data = 0.0;
                    let bias = random.rand_range_f32(-1.0..1.0);
                    let weights = (0..amount_weights)
                        .map(|_| random.rand_range_f32(-1.0..1.0))
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

    pub fn process(&mut self, input: &[f32]) {
        for (i, data) in input.into_iter().enumerate() {
            let node = self.layers[0].nodes.get_mut(i).expect("Too much input passed");
            node.data = *data;
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

    pub fn output(&self) -> Vec<f32> {
        self.layers
            .last()
            .unwrap()
            .nodes
            .iter()
            .map(|node| node.data)
            .collect_vec()
    }
}

pub trait GeneticAlgorithms {
    fn mutate(&mut self);
    fn crossover(&self, other: &Self) -> Self;
}

impl GeneticAlgorithms for NeuralNetwork {
    fn mutate(&mut self) {
        let mut random = Random::new();
        for layer in 0..self.layers.len() {
            for node in 0..self.layers[layer].nodes.len() {
                if random.rand_range(0..100) > 80 {
                    let node = self.get(layer, node);

                    node.bias *= random.rand_range_f32(0.5..1.0);
                    node.bias += random.rand_range_f32(0.0..0.1);

                    for weight in node.weights.iter_mut() {
                        *weight *= random.rand_range_f32(0.5..1.0);
                    }
                }
            }
        }
    }

    fn crossover(&self, other: &Self) -> Self {
        let mut new_network = NeuralNetwork::default();

        for i in 0..self.layers.len() {
            let first = self.layers[i].nodes.iter();
            let second = other.layers[i].nodes.iter();

            let nodes = first.zip(second).map(|(x, y)| x.add(y)).collect_vec();

            new_network.add_layer(Layer { nodes });
        }

        new_network
    }
}
