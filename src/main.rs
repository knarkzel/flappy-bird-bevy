use bevy::prelude::*;
use knarkzel::prelude::*;

use bevy::sprite::collide_aabb::collide;

use std::f64::consts::E;

const WIDTH: f32 = 1280.0;
const HEIGHT: f32 = 960.0;
const BIRDS: usize = 1000;

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
        .add_system(player_fitness.system())
        .add_system(player_process_data.system())
        .add_system(spawn_pipes.system())
        .add_system(move_pipes.system())
        .add_system(despawn_pipes.system())
        .add_system(check_dead_birds.system())
        .run();
}

#[derive(Debug, Clone)]
struct Player {
    velocity: Vec2,
    neural_network: NeuralNetwork,
    fitness: f32,
}

#[derive(Default)]
struct Timer(f32, Random);

#[derive(Default)]
struct DeadBirds(Vec<Player>);

struct Pipe(f32);

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands.spawn().insert(Timer(2.5, Default::default()));

    commands.spawn().insert(DeadBirds::default());

    let mut random = Random::new();

    for _ in 0..BIRDS {
        let velocity = Vec2::default();
        let neural_network = NeuralNetwork::new(&[4, 15, 1]);
        commands
            .spawn_bundle(SpriteBundle {
                material: materials.add(
                    Color::rgb(
                        random.rand_float() as f32,
                        random.rand_float() as f32,
                        random.rand_float() as f32,
                    )
                    .into(),
                ),
                // transform: Transform::from_xyz(0., 300., 0.),
                transform: Transform::from_xyz(
                    random.rand_range_float((-WIDTH / 2.0) as f64..0.0) as f32,
                    300.,
                    0.,
                ),
                sprite: Sprite::new(Vec2::new(64., 64.)),
                ..Default::default()
            })
            .insert(Player {
                velocity,
                neural_network,
                fitness: 0.0,
            });
    }
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

            let gap_y = y2 - (height + gap_top + gap_bottom) / 2.0;

            commands.spawn_bundle(pipe_top).insert(Pipe(gap_y));
            commands.spawn_bundle(pipe_bottom).insert(Pipe(gap_y));
        }
    }
}

fn move_pipes(mut query: Query<(&Pipe, &mut Transform)>) {
    for (_, mut transform) in query.iter_mut() {
        transform.translation.x -= 3.0;
    }
}

fn player_fitness(time: Res<Time>, mut query: Query<&mut Player>) {
    for mut player in query.iter_mut() {
        player.fitness += time.delta_seconds();
    }
}

fn despawn_pipes(mut commands: Commands, query: Query<(Entity, &Transform, &Pipe)>) {
    for (entity, transform, _) in query.iter() {
        if transform.translation.x < (-WIDTH - 128.0) / 2.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn player_movement(mut query: Query<(&mut Player, &mut Transform)>) {
    for (mut player, mut transform) in query.iter_mut() {
        player.velocity.y -= 1.0;
        transform.translation.y += player.velocity.y;
        let output = player.neural_network.output();

        if output[0] > 0.5 {
            player.velocity.y = 15.0;
        }
    }
}

fn player_collision(
    mut commands: Commands,
    query_player: Query<(&Player, &Transform, &Sprite, Entity)>,
    query_objects: Query<(&Pipe, &Transform, &Sprite)>,
    mut query_deadbirds: Query<&mut DeadBirds>,
) {
    for (player, player_transform, player_sprite, player_entity) in query_player.iter() {
        let a_pos = player_transform.translation;
        let a_size = player_sprite.size;

        for (_, object_transform, object_sprite) in query_objects.iter() {
            let b_pos = object_transform.translation;
            let b_size = object_sprite.size;

            let player_above = a_pos.y > HEIGHT / 2.0;
            let player_below = a_pos.y < -HEIGHT / 2.0;

            if collide(a_pos, a_size, b_pos, b_size).is_some() || player_above || player_below {
                if let Ok(mut deadbirds) = query_deadbirds.single_mut() {
                    commands.entity(player_entity).despawn();
                    deadbirds.0.push(player.clone());
                    dbg!(&deadbirds.0.len());
                    break;
                }
            }
        }
    }
}

fn player_process_data(
    mut query: Query<(&mut Player, &Transform)>,
    pipes: Query<(&Pipe, &Transform)>,
) {
    for (mut player, transform) in query.iter_mut() {
        let translation = transform.translation;
        let bird_x = translation.x;
        let mut closest_x = f32::MAX;
        let (mut stored_gap, mut pipe_x) = (0.0, 0.0);

        for (pipe, transform) in pipes.iter() {
            let position = transform.translation;
            let (x, _y) = (position.x, position.y);
            if x + 50.0 < closest_x && x > bird_x {
                closest_x = x;
                stored_gap = pipe.0;
                pipe_x = x;
            }
        }

        let position = (translation.y + HEIGHT / 2.0) / HEIGHT;
        let speed = player.velocity.y / 15.0;
        let delta_x = (pipe_x - translation.x) / WIDTH;
        player.neural_network.process(&[
            position as f64,
            speed as f64,
            (stored_gap / HEIGHT) as f64,
            delta_x as f64,
        ]);
    }
}

fn check_dead_birds(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut timer: Query<&mut Timer>,
    mut deadbirds: Query<&mut DeadBirds>,
    pipes: Query<(Entity, &Pipe)>,
    players: Query<&Entity>,
) {
    // SPAWN GOOD BIRDS
    if let Ok(mut deadbirds) = deadbirds.single_mut() {
        if deadbirds.0.len() >= BIRDS && players.iter().count() <= 0 {
            // get rid of PIPES
            for (entity, _) in pipes.iter() {
                commands.entity(entity).despawn();
            }

            // reset TIMER
            if let Ok(mut timer) = timer.single_mut() {
                timer.0 = 2.5;
            }

            deadbirds
                .0
                .sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
            let mut best_birds = deadbirds.0.clone().into_iter().take(50).collect_vec();
            let mut other_birds = (0..9).map(|_| best_birds.clone()).flatten().collect_vec();
            for bird in other_birds.iter_mut() {
                bird.neural_network.mutate();
            }

            // NEW BIRDS

            let new_birds = (0..500)
                .map(|_| {
                    let velocity = Vec2::default();
                    let neural_network = NeuralNetwork::new(&[3, 10, 1]);
                    Player {
                        velocity,
                        neural_network,
                        fitness: 0.0,
                    }
                })
                .collect_vec();

            best_birds.extend(other_birds);
            best_birds.extend(new_birds);
            deadbirds.0.clear();

            let mut random = Random::new();

            for mut bird in best_birds.into_iter() {
                bird.velocity = Vec2::default();
                commands
                    .spawn_bundle(SpriteBundle {
                        material: materials.add(
                            Color::rgb(
                                random.rand_float() as f32,
                                random.rand_float() as f32,
                                random.rand_float() as f32,
                            )
                            .into(),
                        ),
                        transform: Transform::from_xyz(
                            random.rand_range_float((-WIDTH / 2.0) as f64..0.0) as f32,
                            300.,
                            0.,
                        ),
                        sprite: Sprite::new(Vec2::new(64., 64.)),
                        ..Default::default()
                    })
                    .insert(bird);
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

#[derive(Debug, Clone)]
struct Layer {
    nodes: Vec<Node>,
}

#[derive(Debug, Clone)]
struct NeuralNetwork {
    layers: Vec<Layer>,
}

impl NeuralNetwork {
    fn new(structure: &[usize]) -> Self {
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

    fn process(&mut self, input: &[f64]) {
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

    fn output(&self) -> Vec<f64> {
        self.layers
            .last()
            .unwrap()
            .nodes
            .iter()
            .map(|node| node.data)
            .collect_vec()
    }

    fn get(&mut self, layer: usize, node: usize) -> &mut Node {
        &mut self.layers[layer].nodes[node]
    }

    fn mutate(&mut self) {
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
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + E.powf(-x))
}
