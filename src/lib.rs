use bevy::prelude::*;
use knarkzel::prelude::Random;
use neuralnetwork::NeuralNetwork;
use bird::Bird;

pub mod bird;
pub mod pipe;

pub const WIDTH: f32 = 1280.0;
pub const HEIGHT: f32 = 960.0;
pub const BIRDS: usize = 1000;
pub const STRUCTURE: &[usize] = &[5, 10, 2];

pub const PIPE_WIDTH: f32 = 64.0 * 2.0;

#[derive(Default)]
pub struct Timer(pub f32);

pub fn spawn_bird(commands: &mut Commands, materials: &mut ResMut<Assets<ColorMaterial>>, random: &mut Random, neural_network: NeuralNetwork) {
    let fitness = 0.0;
    let multiplier = 0.0;
    let velocity = Vec2::default();

    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(
                Color::rgb(random.rand_f32(), random.rand_f32(), random.rand_f32()).into(),
            ),
            transform: Transform::from_xyz(random.rand_range_f32(-WIDTH / 2.0..0.0), 0.0, 0.0),
            sprite: Sprite::new(Vec2::new(64.0, 64.0)),
            ..Default::default()
        })
        .insert(Bird {
            velocity,
            multiplier,
            neural_network,
            fitness,
        });
}
