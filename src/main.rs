use bevy::prelude::*;
use knarkzel::prelude::*;

use bevy::sprite::collide_aabb::collide;

use std::f64::consts::E;

const WIDTH: f32 = 1280.0;
const HEIGHT: f32 = 960.0;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Flappy Bird".to_string(),
            width: WIDTH,
            height: HEIGHT,
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(player_movement.system())
        .add_system(player_collision.system())
        .add_system(spawn_pipes.system())
        .add_system(move_pipes.system())
        .add_system(despawn_pipes.system())
        .run();
}

struct Player {
    velocity: Vec2,
    neural_network: NeuralNetwork,
}

#[derive(Default)]
struct Timer(f32, Random);

struct Pipe;

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands.spawn().insert(Timer::default());

    let velocity = Vec2::default();
    let neural_network = NeuralNetwork::new(&[2, 10, 1]);

    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(0.26, 0.53, 0.96).into()),
            transform: Transform::from_xyz(0., 300., 0.),
            sprite: Sprite::new(Vec2::new(64., 64.)),
            ..Default::default()
        })
        .insert(Player { velocity, neural_network });
}

fn spawn_pipes(
    time: Res<Time>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<&mut Timer>,
) {
    if let Ok(mut timer) = query.single_mut() {
        timer.0 += time.delta_seconds();
        if timer.0 > 2.5 {
            timer.0 = 0.0;

            let width = 64.0 * 2.0;
            let height = 64.0 * 7.5;
            let color = Color::rgb(0.44, 0.81, 0.42);

            let difficulty = 5;
            let random = timer.1.rand_range(0..difficulty) as f32;
            let gap_top = 64.0 * random;
            let gap_bottom = 64.0 * (difficulty as f32 - random);

            let (x1, y1) = ((WIDTH + width) / 2.0, (-HEIGHT + height) / 2.0 - gap_bottom);
            let (x2, y2) = ((WIDTH + width) / 2.0, (HEIGHT - height) / 2.0 + gap_top);

            let pipe_bottom = SpriteBundle {
                material: materials.add(color.into()),
                transform: Transform::from_xyz(x1, y1, 0.),
                sprite: Sprite::new(Vec2::new(width, height)),
                ..Default::default()
            };

            let pipe_top = SpriteBundle {
                material: materials.add(color.into()),
                transform: Transform::from_xyz(x2, y2, 0.),
                sprite: Sprite::new(Vec2::new(width, height)),
                ..Default::default()
            };

            commands.spawn_bundle(pipe_top).insert(Pipe);
            commands.spawn_bundle(pipe_bottom).insert(Pipe);
        }
    }
}

fn move_pipes(mut query: Query<(&Pipe, &mut Transform)>) {
    for (_, mut transform) in query.iter_mut() {
        transform.translation.x -= 3.0;
    }
}

fn despawn_pipes(mut commands: Commands, query: Query<(Entity, &Transform, &Pipe)>) {
    for (entity, transform, _) in query.iter() {
        if transform.translation.x < (-WIDTH - 128.0) / 2.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Player, &mut Transform)>,
) {
    if let Ok((mut player, mut transform)) = query.single_mut() {
        player.velocity.y -= 1.0;
        transform.translation.y += player.velocity.y;

        if keyboard_input.pressed(KeyCode::Space) {
            player.velocity.y = 15.0;
        }
    }
}

fn player_collision(
    mut commands: Commands,
    query_player: Query<(&Player, &Transform, &Sprite, Entity)>,
    query_objects: Query<(&Pipe, &Transform, &Sprite)>,
) {
    if let Ok((_, player_transform, player_sprite, player_entity)) = query_player.single() {
        let a_pos = player_transform.translation;
        let a_size = player_sprite.size;

        for (_, object_transform, object_sprite) in query_objects.iter() {
            let b_pos = object_transform.translation;
            let b_size = object_sprite.size;

            if collide(a_pos, a_size, b_pos, b_size).is_some() {
                commands.entity(player_entity).despawn();
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
struct Node {
    data: f64,
    bias: f64,
    weights: Vec<f64>,
}

impl Node {
    fn weighted(&self, weight_index: usize) -> f64 {
        self.data * self.weights[weight_index] + self.bias
    }
}

#[derive(Debug)]
struct Layer {
    nodes: Vec<Node>,
}

#[derive(Debug)]
struct NeuralNetwork {
    layers: Vec<Layer>,
}

impl NeuralNetwork {
    fn new(shape: &[usize]) -> Self {
        let mut layers = vec![];
        let mut random = Random::new();

        for layer in shape.windows(2) {
            let amount_nodes = layer[0];
            let amount_weights = layer[1];

            let nodes = (0..amount_nodes)
                .map(|_| {
                    let data = 0.0;
                    let bias = -1.0 + random.rand_float() / f64::MAX * 2.0;
                    let weights = (0..amount_weights)
                        .map(|_| -1.0 + random.rand_float() / f64::MAX * 2.0)
                        .collect_vec();

                    Node { data, bias, weights }
                })
                .collect_vec();

            layers.push(Layer { nodes });
        }

        let nodes = vec![Node::default(); *shape.last().unwrap()];
        let output_layer = Layer { nodes };
        layers.push(output_layer);

        NeuralNetwork { layers }
    }

    fn process(&mut self, values: &[f64]) {
        for (i, data) in values.into_iter().enumerate() {
            if let Some(node) = self.layers[0].nodes.get_mut(i) {
                node.data = *data;
            }
        }

        for i in 1..self.layers.len() {
            for j in 0..self.layers[i].nodes.len() {
                let sum = self.layers[i - 1].nodes.iter().map(|node| node.weighted(j)).sum();
                self.layers[i].nodes[j].data = sigmoid(sum);
            }
        }
    }

    fn output(&self) -> Vec<f64> {
        self.layers.last().unwrap().nodes.iter().map(|node| node.data).collect_vec()
    }
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + E.powf(-x))
}
