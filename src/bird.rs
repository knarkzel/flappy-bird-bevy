use bevy::prelude::*;
use knarkzel::prelude::*;
use neuralnetwork::*;

use crate::{*, Timer};
use crate::pipe::*;

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
    pub velocity: Vec2,
    pub neural_network: NeuralNetwork,
}

#[derive(Default)]
pub struct DeadBirds(pub Vec<Bird>);

fn bird_fitness(time: Res<Time>, mut query: Query<&mut Bird>) {
    for mut bird in query.iter_mut() {
        bird.fitness += time.delta_seconds();
    }
}


fn bird_movement(mut query: Query<(&mut Bird, &mut Transform)>) {
    for (mut bird, mut transform) in query.iter_mut() {
        bird.velocity.y -= 1.0;
        transform.translation.y += bird.velocity.y;
        let output = bird.neural_network.output();

        let speed = 1.0;

        if let Some(value) = output.get(0) {
            if value > &0.62 {
                bird.velocity.y = 15.0;
            }
        }

        if let Some(value) = output.get(1) {
            if value > &0.62 {
                transform.translation.x += speed;
            }
        }

        if let Some(value) = output.get(2) {
            if value > &0.62 {
                transform.translation.x -= speed;
            }
        }
    }
}

fn bird_collision(
    mut commands: Commands,
    query_bird: Query<(&Bird, &Transform, &Sprite, Entity)>,
    query_objects: Query<(&Pipe, &Transform, &Sprite)>,
    mut query_deadbirds: Query<&mut DeadBirds>,
) {
    for (bird, bird_transform, bird_sprite, bird_entity) in query_bird.iter() {
        let a_pos = bird_transform.translation;
        let a_size = bird_sprite.size;

        for (_, object_transform, object_sprite) in query_objects.iter() {
            let b_pos = object_transform.translation;
            let b_size = object_sprite.size;

            let bounds_x = a_pos.y > HEIGHT / 2.0 || a_pos.y < -HEIGHT / 2.0;
            let bounds_y = a_pos.x > WIDTH / 2.0 || a_pos.x < -WIDTH / 2.0;

            if collide(a_pos, a_size, b_pos, b_size).is_some() || bounds_x || bounds_y {
                if let Ok(mut deadbirds) = query_deadbirds.single_mut() {
                    commands.entity(bird_entity).despawn();
                    deadbirds.0.push(bird.clone());
                    break;
                }
            }
        }
    }
}

fn bird_process(
    mut query: Query<(&mut Bird, &Transform)>,
    pipes: Query<(&Pipe, &Transform)>,
) {
    for (mut bird, transform) in query.iter_mut() {
        let translation = transform.translation;
        let bird_x = translation.x;
        let mut closest_x = f32::MAX;
        let (mut stored_gap, mut pipe_x) = (0.0, 0.0);

        for (pipe, transform) in pipes.iter() {
            let position = transform.translation;
            let (x, _y) = (position.x, position.y);
            if x < closest_x && x > bird_x {
                closest_x = x;
                stored_gap = pipe.0 / HEIGHT;
                pipe_x = x;
            }
        }

        let position = (translation.y + HEIGHT / 2.0) / HEIGHT;
        let speed = bird.velocity.y / 15.0;
        let delta_x = (pipe_x - translation.x) / WIDTH;
        bird.neural_network.process(&[
            position,
            speed,
            stored_gap,
            delta_x,
        ]);
    }
}

fn check_dead_birds(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut timer: Query<&mut Timer>,
    mut deadbirds: Query<&mut DeadBirds>,
    mut random: Query<&mut Random>,
    pipes: Query<(Entity, &Pipe)>,
) {
    // SPAWN GOOD BIRDS
    if let Ok(mut deadbirds) = deadbirds.single_mut() {
        if deadbirds.0.len() >= BIRDS {
            let mut random = random.single_mut().expect("No randomizer found!");

            // get rid of PIPES
            pipes.iter().for_each(|(entity, _)| commands.entity(entity).despawn());


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
                let mut new_network = bird.neural_network.crossover(&best_birds[0].neural_network);
                new_network.mutate();
                bird.neural_network = new_network;
            }

            // NEW BIRDS
            let new_birds = (0..500)
                .map(|_| {
                    let velocity = Vec2::default();
                    let neural_network = NeuralNetwork::new(&[3, 10, 1]);
                    Bird {
                        velocity,
                        neural_network,
                        fitness: 0.0,
                    }
                })
                .collect_vec();

            best_birds.extend(other_birds);
            best_birds.extend(new_birds);
            deadbirds.0.clear();

            for mut bird in best_birds.into_iter() {
                bird.velocity = Vec2::default();
                commands
                    .spawn_bundle(SpriteBundle {
                        material: materials.add(
                            Color::rgb(
                                random.rand_f32(),
                                random.rand_f32(),
                                random.rand_f32(),
                            )
                            .into(),
                        ),
                        transform: Transform::from_xyz(
                            random.rand_range_f32(-WIDTH / 2.0..0.0),
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
