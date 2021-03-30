use bevy::prelude::*;
use bird::Bird;
use knarkzel::prelude::Random;
use neuralnetwork::NeuralNetwork;

pub mod bird;
pub mod pipe;

pub const BIRDS: usize = 1000;
pub const STRUCTURE: &[usize] = &[3, 10, 3];

pub const PIPE_WIDTH: f32 = 64.0 * 2.0;

pub const SIZE_DELTA: f32 = 0.2;

#[derive(Default)]
pub struct Timer(pub f32);

pub fn spawn_bird(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
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
            material: materials.add(Color::rgb(random.rand_f32(), random.rand_f32(), random.rand_f32()).into()),
            transform: Transform::from_xyz(random.rand_range_f32(-windows.0 / 2.0..0.0), 0.0, 0.0),
            sprite: Sprite::new(Vec2::new(64.0, 64.0) * Vec2::new(size, size)),
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
