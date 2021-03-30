use bevy::prelude::*;

use crate::Timer;
use crate::*;

pub struct PipePlugin;

impl Plugin for PipePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(spawn_pipes.system())
            .add_system(move_pipes.system())
            .add_system(despawn_pipes.system());
    }
}

#[derive(PartialEq)]
pub enum PipeType {
    Top,
    Bottom,
}

pub struct Pipe(pub PipeType);

fn spawn_pipes(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut timer: Query<&mut Timer>,
    mut random: Query<&mut Random>,
    time: Res<Time>,
    windows: Res<Windows>,
) {
    let window = windows.get_primary().unwrap();
    let (width, height) = (window.width(), window.height());

    if let Ok(mut timer) = timer.single_mut() {
        timer.0 += time.delta_seconds();
        if timer.0 > 2.5 {
            timer.0 = 0.0;

            let color = Color::rgb(0.44, 0.81, 0.42);

            let mut random = random.single_mut().expect("Failed to get random");

            let gap_size = 64.0 * 5.0;
            let random_value = random.rand_range_f32(0.0..height - gap_size);

            let x = (width + PIPE_WIDTH) / 2.0;
            let (x1, y1) = (x, -height + random_value);
            let (x2, y2) = (x, random_value + gap_size);

            let pipe_bottom = SpriteBundle {
                material: materials.add(color.into()),
                transform: Transform::from_xyz(x1, y1, 0.),
                sprite: Sprite::new(Vec2::new(PIPE_WIDTH, height)),
                ..Default::default()
            };

            let pipe_top = SpriteBundle {
                material: materials.add(color.into()),
                transform: Transform::from_xyz(x2, y2, 0.),
                sprite: Sprite::new(Vec2::new(PIPE_WIDTH, height)),
                ..Default::default()
            };

            commands.spawn_bundle(pipe_top).insert(Pipe(PipeType::Top));
            commands.spawn_bundle(pipe_bottom).insert(Pipe(PipeType::Bottom));
        }
    }
}

fn move_pipes(mut pipes: Query<(&Pipe, &mut Transform)>) {
    for (_, mut transform) in pipes.iter_mut() {
        transform.translation.x -= 3.0;
    }
}

fn despawn_pipes(mut commands: Commands, pipes: Query<(Entity, &Transform, &Pipe)>, windows: Res<Windows>) {
    let window = windows.get_primary().unwrap();
    let (width, _) = (window.width(), window.height());
    
    for (entity, transform, _) in pipes.iter() {
        if transform.translation.x < (-width - 128.0) / 2.0 {
            commands.entity(entity).despawn();
        }
    }
}
