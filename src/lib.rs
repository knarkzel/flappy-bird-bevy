use bevy::prelude::*;
use bird::Bird;

pub mod bird;
pub mod pipe;
pub mod knarkzel;
pub mod neuralnetwork;

use knarkzel::prelude::Random;
use neuralnetwork::NeuralNetwork;

pub const BIRDS: usize = 1000;
pub const STRUCTURE: &[usize] = &[3, 10, 3];

pub const PIPE_WIDTH: f32 = 64.0 * 2.0;

pub const SIZE_DELTA: f32 = 0.2;

#[derive(Default, Component)]
pub struct Timer(pub f32);

pub fn spawn_bird(
    commands: &mut Commands,
    windows: (f32, f32),
    random: &mut Random,
    neural_network: NeuralNetwork,
    size: f32,
) {
    let fitness = 0.0;
    let multiplier = 0.0;
    let velocity = Vec2::default();

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(random.rand_f32(), random.rand_f32(), random.rand_f32()),
                custom_size: Some(Vec2::new(64.0, 64.0) * Vec2::new(size, size)),
                ..Default::default()
            },
            transform: Transform::from_xyz(random.rand_range_f32(-windows.0 / 2.0..0.0), 0.0, 0.0),
            ..Default::default()
        })
        .insert(Bird {
            velocity,
            multiplier,
            size,
            neural_network,
            fitness,
        });
}
