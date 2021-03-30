use knarkzel::prelude::*;
use bevy::{prelude::*, window::WindowMode};

use bird::*;
use game::{Timer, *};
use neuralnetwork::NeuralNetwork;
use pipe::*;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Flappy Bird".to_string(),
            resizable: true,
            mode: WindowMode::Fullscreen { use_size: false },
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(BirdPlugin)
        .add_plugin(PipePlugin)
        .add_startup_system(setup.system())
        .run();
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn().insert(Timer(2.5));
    commands.spawn().insert(DeadBirds::default());
    commands.spawn().insert(Random::default());

    let windows = (1920.0, 1080.0);
    let mut random = Random::new();
    (0..BIRDS).for_each(|_| {
        let neural_network = NeuralNetwork::new(STRUCTURE, &mut random);
        let size = random.rand_range_f32(1.0 - SIZE_DELTA..1.0 + SIZE_DELTA);
        spawn_bird(&mut commands, &mut materials, windows, &mut random, neural_network, size);
    });
}
