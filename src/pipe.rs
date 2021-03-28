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

pub struct Pipe(pub f32);

fn spawn_pipes(
    time: Res<Time>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut timer: Query<&mut Timer>,
    mut random: Query<&mut Random>,
) {
    if let Ok(mut timer) = timer.single_mut() {
        timer.0 += time.delta_seconds();
        if timer.0 > 2.5 {
            timer.0 = 0.0;

            let width = 64.0 * 2.0;
            let height = 64.0 * 7.5;
            let color = Color::rgb(0.44, 0.81, 0.42);

            let mut random = random.single_mut().expect("Failed to get random");

            let difficulty = 5;
            let random = random.rand_range(0..difficulty) as f32;
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

fn despawn_pipes(mut commands: Commands, query: Query<(Entity, &Transform, &Pipe)>) {
    for (entity, transform, _) in query.iter() {
        if transform.translation.x < (-WIDTH - 128.0) / 2.0 {
            commands.entity(entity).despawn();
        }
    }
}
