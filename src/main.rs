use bevy::prelude::*;
use knarkzel::prelude::*;

use game::{*, Timer};
use bird::*;
use pipe::*;
use neuralnetwork::NeuralNetwork;

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
        .add_plugin(BirdPlugin)
        .add_plugin(PipePlugin)
        .add_startup_system(setup.system())
        .run();
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    commands.spawn().insert(Timer(2.5));

    commands.spawn().insert(Randomizer::default());

    commands.spawn().insert(DeadBirds::default());

    let mut random = Random::new();

    for _ in 0..BIRDS {
        let velocity = Vec2::default();
        let neural_network = NeuralNetwork::new(&[4, 15, 3]);
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
            .insert(Bird {
                velocity,
                neural_network,
                fitness: 0.0,
            });
    }
}
