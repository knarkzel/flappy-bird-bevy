pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn slurp<T: AsRef<str>>(argument: T) -> Result<String> {
    let destination = argument.as_ref();
    let body = std::fs::read_to_string(destination)?;
    Ok(body)
}

pub mod random {
    use crate::*;

    #[derive(Component)]
    pub struct Random {
        generator: oorandom::Rand32,
    }

    impl Default for Random {
        fn default() -> Self {
            let mut buffer = [0u8; 32];
            getrandom::getrandom(&mut buffer).unwrap_or(());
            // parse array of 32 u8s as 32 bit number, upper limit 4294967295
            let seed = buffer.iter().enumerate().fold(0, |acc: u32, (i, x)| {
                let value = if x > &127 { 2u32.pow(i as u32) } else { 0 };
                acc + value
            });
            Self {
                generator: oorandom::Rand32::new(seed as u64),
            }
        }
    }

    impl Random {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn with_seed(self, seed: u64) -> Self {
            Self {
                generator: oorandom::Rand32::new(seed),
            }
        }

        pub fn rand_u32(&mut self) -> u32 {
            self.generator.rand_u32()
        }

        pub fn rand_i32(&mut self) -> i32 {
            self.generator.rand_i32()
        }

        pub fn rand_usize(&mut self) -> usize {
            self.generator.rand_u32() as usize
        }

        pub fn rand_isize(&mut self) -> isize {
            self.generator.rand_i32() as isize
        }

        pub fn rand_f32(&mut self) -> f32 {
            self.generator.rand_float()
        }

        pub fn rand_range(&mut self, range: std::ops::Range<u32>) -> u32 {
            self.generator.rand_range(range)
        }

        pub fn rand_range_f32(&mut self, range: std::ops::Range<f32>) -> f32 {
            range.start + self.rand_f32() * (range.end - range.start)
        }
    }
}

/// `use knarkzel::prelude::*;`
pub mod prelude {
    /*!
    # Filesystem

    The slurp function can be used for reading a file.

    ```rust
    let columns = slurp("mock.csv")?;
    ```

    # Regex

    ```rust
    let text = "Not my favorite movie: 'Citizen Kane' (1941).";
    let regex = Regex::new(r"'([^']+)'\s+\((\d{4})\)")?;
    let captures = regex.captures(text)?;

    assert_eq!(&captures[0], "'Citizen Kane' (1941)");
    assert_eq!(&captures[1], "Citizen Kane");
    assert_eq!(&captures[2], "1941");
    ```

    # Reformation

    ```rust
    #[derive(Reformation, Debug)]
    #[reformation(r"{year}-{month}-{day} {hour}:{minute}")]
    struct Date{
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
    }

    fn main() {
        let date = Date::parse("2018-12-22 20:23")?;
    }
    ```

    # Random

    ```rust
    let mut random = Random::new().with_seed(1234);
    let unsigned = random.rand_u64();
    let signed = random.rand_i64();
    let float = random.rand_float();
    let range = random.rand_range(1..100);
    let float_range = random.rand_range_float(-5.0..5.0);
    ```

    # Itertools

    ```rust
    let items = vec![1, 2, 3];
    let data = items
        .iter()
        .map(|x| x * 3)
        .collect_vec();
    ```
    */

    pub use super::{random::Random, slurp};
    pub use itertools::{self, Itertools};
}
