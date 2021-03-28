use knarkzel::prelude::Random;

pub mod bird;
pub mod pipe;

pub const WIDTH: f32 = 1280.0;
pub const HEIGHT: f32 = 960.0;
pub const BIRDS: usize = 1000;

#[derive(Default)]
pub struct Timer(pub f32);

#[derive(Default)]
pub struct Randomizer(pub Random);
