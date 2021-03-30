use bevy::prelude::*;
use knarkzel::prelude::*;
use neuralnetwork::*;

use crate::pipe::*;
use crate::{Timer, *};

use bevy::sprite::collide_aabb::collide;

pub struct BirdPlugin;

impl Plugin for BirdPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(bird_movement.system())
            .add_system(bird_collision.system())
            .add_system(bird_fitness.system())
            .add_system(bird_process.system())
            .add_system(check_dead_birds.system());
    }
}

#[derive(Debug, Clone)]
pub struct Bird {
    pub fitness: f32,
    pub multiplier: f32,
    pub velocity: Vec2,
    pub size: f32,
    pub neural_network: NeuralNetwork,
}

impl Bird {
    fn get_speed(&self) -> f32 {
        2.0 - self.size
    }
}

#[derive(Default)]
pub struct DeadBirds(pub Vec<Bird>);

impl DeadBirds {
    fn sort(&mut self) {
        self.0
            .sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
    }
}

fn bird_fitness(mut birds: Query<&mut Bird>, time: Res<Time>) {
    for mut bird in birds.iter_mut() {
        bird.fitness += time.delta_seconds() * bird.multiplier;
    }
}

fn bird_movement(mut birds: Query<(&mut Bird, &mut Transform)>, time: Res<Time>, windows: Res<Windows>) {
    let delta = time.delta_seconds() * 60.0;
    let window = windows.get_primary().unwrap();
    let (width, _height) = (window.width(), window.height());

    for (mut bird, mut transform) in birds.iter_mut() {
        let deltaspeed = delta * bird.get_speed();
        let bird_pos = transform.translation;

        bird.velocity.y -= deltaspeed;
        transform.translation.y += bird.velocity.y * deltaspeed;

        bird.multiplier = (1.0 - (bird_pos.x / width).abs()) * 5.0;

        let output = bird.neural_network.output();
        if output[0] > 0.6 {
            bird.velocity.y = 15.0 * deltaspeed;
        }
        if output[1] > 0.6 {
            transform.translation.x += deltaspeed;
        }
        if output[2] > 0.6 {
            transform.translation.x -= deltaspeed;
        }
    }
}

fn bird_collision(
    mut commands: Commands,
    mut deadbirds: Query<&mut DeadBirds>,
    birds: Query<(&Bird, &Transform, &Sprite, Entity)>,
    pipes: Query<(&Pipe, &Transform, &Sprite)>,
    windows: Res<Windows>,
) {
    let window = windows.get_primary().unwrap();
    let (width, height) = (window.width(), window.height());

    for (bird, bird_transform, bird_sprite, bird_entity) in birds.iter() {
        let bird_pos = bird_transform.translation;
        let bird_size = bird_sprite.size;

        for (_, pipe_transform, pipe_sprite) in pipes.iter() {
            let pipe_pos = pipe_transform.translation;
            let pipe_size = pipe_sprite.size;

            let bounds_x = bird_pos.y > height / 2.0 || bird_pos.y < -height / 2.0;
            let bounds_y = bird_pos.x > width / 2.0 || bird_pos.x < -width / 2.0;

            if collide(bird_pos, bird_size, pipe_pos, pipe_size).is_some() || bounds_x || bounds_y {
                if let Ok(mut deadbirds) = deadbirds.single_mut() {
                    commands.entity(bird_entity).despawn();
                    deadbirds.0.push(bird.clone());
                    break;
                }
            }
        }
    }
}

fn bird_process(
    mut birds: Query<(&mut Bird, &Transform)>,
    pipes: Query<(&Pipe, &Transform, &Sprite)>,
    windows: Res<Windows>,
) {
    let window = windows.get_primary().unwrap();
    let (_width, height) = (window.width(), window.height());

    // get pipe stats
    let mut data = pipes.iter().map(|(pipe, transform, sprite)| {
        let pipe_size = sprite.size;
        let pipe_pos = transform.translation;
        match pipe.0 {
            PipeType::Top => (pipe_pos.x, pipe_pos.y - pipe_size.y),
            PipeType::Bottom => (pipe_pos.x, pipe_pos.y + pipe_size.y),
        }
    }).collect_vec();
    data.sort_by(|(a, _), (b, _)| a.partial_cmp(&b).unwrap());
    
    for (mut bird, transform) in birds.iter_mut() {
        let bird_pos = transform.translation;
        let y = bird_pos.y / height;

        let valid_pipes = data.iter().filter(|(x, _)| *x + PIPE_WIDTH > bird_pos.x).collect_vec();
        let (top, bottom) = if valid_pipes.len() >= 2 {
            (valid_pipes[0].1 / height, valid_pipes[1].1 / height)
        } else {
            (0.0, 0.0)
        };

        bird.neural_network.process(&[y, top, bottom]);
    }
}

fn check_dead_birds(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut timer: Query<&mut Timer>,
    mut deadbirds: Query<&mut DeadBirds>,
    mut random: Query<&mut Random>,
    pipes: Query<(Entity, &Pipe)>,
    windows: Res<Windows>,
) {
    // SPAWN GOOD BIRDS
    if let Ok(mut deadbirds) = deadbirds.single_mut() {
        if deadbirds.0.len() >= BIRDS {
            let mut random = random.single_mut().expect("No randomizer found!");

            // get rid of PIPES
            pipes
                .iter()
                .for_each(|(entity, _)| commands.entity(entity).despawn());

            // reset TIMER
            if let Ok(mut timer) = timer.single_mut() {
                timer.0 = 2.5;
            }

            deadbirds.sort();

            let mut best_birds = deadbirds.0.clone().into_iter().take(10).collect_vec();
            let mut other_birds = (0..50).map(|_| best_birds.clone()).flatten().collect_vec();
            for bird in other_birds.iter_mut() {
                let random_bird = random.rand_range(0..3) as usize;
                bird.size = (best_birds[random_bird].size + bird.size) / 2.0;

                let mut new_network = bird.neural_network.crossover(&best_birds[random_bird].neural_network);
                new_network.mutate();
                bird.neural_network = new_network;
            }

            // NEW BIRDS
            let new_birds = (0..1000 - (best_birds.len() + other_birds.len()))
                .map(|_| {
                    let velocity = Vec2::default();
                    let size = random.rand_range_f32(1.0 - SIZE_DELTA..1.0 + SIZE_DELTA);
                    let neural_network = NeuralNetwork::new(STRUCTURE, &mut random);
                    Bird {
                        velocity,
                        neural_network,
                        size,
                        multiplier: 0.0,
                        fitness: 0.0,
                    }
                })
                .collect_vec();

            best_birds.extend(other_birds);
            best_birds.extend(new_birds);
            deadbirds.0.clear();

            // dbg!(&best_birds.len());
            let window = windows.get_primary().unwrap();
            let (width, height) = (window.width(), window.height());

            best_birds.into_iter().for_each(|bird| {
                spawn_bird(&mut commands, &mut materials, (width, height), &mut random, bird.neural_network, bird.size);
            })
        }
    }
}
