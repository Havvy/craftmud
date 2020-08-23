#[cfg(test)]
mod mock_rand;

use rand::{
    Rng,
    distributions::{Distribution, Uniform}
};

fn percent_distribution() -> Uniform<u32> {
    Uniform::new_inclusive(1, 100)
}

pub struct Craft {
    state: CraftResult,
}

/// The processing of crafting an item.
#[derive(Debug, Clone, Copy)]
pub struct InProgress {
    durability: i32,
    max_durability: i32,
    quality: i32,
    max_quality: i32,
    progress: i32,
    max_progress: i32,
}

impl InProgress {
    fn apply_operation<RNG: rand::Rng>(&mut self, operation: &Operation, rng: &mut RNG) {
        use std::cmp::{min, max};

        self.durability = max(0, min(self.max_durability, self.durability + operation.diff_durability));

        if operation.chance >= percent_distribution().sample(rng) {
            self.progress = min(self.max_progress, self.progress + operation.diff_progress);
            self.quality = min(self.max_quality, self.quality + operation.diff_quality);
        }
    }

    fn is_complete(&self) -> bool {
        self.progress == self.max_progress
    }

    fn is_failure(&self) -> bool {
        self.durability == 0
    }

    /// Returns true if the craft completed as high quality.
    ///
    /// The chance for success is `Quality**2 / MaxQuality**2`.
    fn check_high_quality<RNG: rand::Rng>(&self, rng: &mut RNG) -> bool {
        let random: i32 = Uniform::new_inclusive(1, self.max_quality * self.max_quality).sample(rng);
        self.quality * self.quality >= random
    }
}

enum CraftResult {
    InProgress(InProgress),
    LowQuality,
    HighQuality,
    Failure,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CraftState {
    InProgress,
    LowQuality,
    HighQuality,
    Failure,
}

/// Error triggered when applying an operation to a craft that's not in progress.
#[derive(Debug, PartialEq, Eq)]
pub struct Error;

impl CraftResult {
    fn apply_operation<RNG: Rng>(&mut self, operation: &Operation, rng: &mut RNG) -> Result<(), Error> {
        match self {
            CraftResult::InProgress(in_progress) => {
                in_progress.apply_operation(operation, rng);

                if in_progress.is_complete() {
                    if in_progress.check_high_quality(rng) {
                        *self = CraftResult::HighQuality;
                    } else {
                        *self = CraftResult::LowQuality;
                    }
                } else if in_progress.is_failure() {
                    *self = CraftResult::Failure;
                }
                Ok(())
            },

            _ => {
                Err(Error)
            }
        }
    }
}

impl Craft {
    pub fn new(quality: i32, max_quality: i32, progress: i32, max_progress: i32, durability: i32) -> Self {
        Self {
            state: CraftResult::InProgress(InProgress {
                quality, max_quality, progress, max_progress, durability,
                max_durability: durability,
            })
        }
    }

    /// Applies the operation if the craft is in progress, returning `Ok(())`.
    /// Otherwise returns `Err(Error)`
    pub fn apply_operation<RNG: Rng>(&mut self, operation: &Operation, rng: &mut RNG) -> Result<(), Error> {
        self.state.apply_operation(operation, rng)
    }

    pub fn state(&self) -> CraftState {
        match self.state {
            CraftResult::InProgress(_) => CraftState::InProgress,
            CraftResult::LowQuality => CraftState::LowQuality,
            CraftResult::HighQuality => CraftState::HighQuality,
            CraftResult::Failure => CraftState::Failure,
        }
    }
}

pub struct Operation {
    diff_quality: i32,
    diff_progress: i32,
    diff_durability: i32,
    chance: u32,
}

impl Default for Operation {
    fn default() -> Self {
        Self {
            diff_quality: 0,
            diff_progress: 0,
            diff_durability: 0,
            chance: 100,
        }
    }
}

impl Operation {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_diff_quality(&mut self, diff: i32) {
        self.diff_quality = diff;
    }

    pub fn set_diff_progress(&mut self, diff: i32) {
        self.diff_progress = diff;
    }

    pub fn set_diff_durability(&mut self, diff: i32) {
        self.diff_durability = diff;
    }

    pub fn set_chance(&mut self, chance: u32) {
        self.chance = std::cmp::max(100, chance);
    }
}

#[cfg(test)]
mod tests {
    // Note(Havvy): The generated sample has to be smaller than the (modified)
    //     chance for success, not greater. So when testing, using high numbers
    //     for failures and low numbers for successes.
    use super::*;

    fn basic_progress() -> Operation {
        let mut op = Operation::new();
        op.set_diff_progress(10);
        op.set_diff_durability(-10);
        op
    }

    fn basic_quality() -> Operation {
        let mut op = Operation::new();
        op.set_diff_durability(-10);
        op.set_diff_quality(10);
        op
    }

    fn chance_quality() -> Operation {
        let mut op: Operation = Operation::new();
        op.set_diff_durability(-10);
        op.set_diff_quality(30);
        op.set_chance(50);
        op
    }

    fn always_success() -> mock_rand::IterRng<std::iter::Repeat<u64>> {
        mock_rand::IterRng::new(
            std::iter::repeat(
                mock_rand::gen_uniform_sample_u32(1, 100, 1) as u64
            )
        )
    }

    #[test]
    fn basic_progress_gives_low_quality() {
        let operation = basic_progress();

        let mut craft = Craft::new(
            0, 20, 0, 40, 40,
        );

        let mut rng = always_success();

        for _ in 0..4 {
            let _ = craft.apply_operation(&operation, &mut rng);
        }

        assert_eq!(CraftState::LowQuality, craft.state());
    }

    #[test]
    fn basic_quality_gives_high_quality() {
        let basic_progress = &basic_progress();
        let basic_quality = &basic_quality();

        let mut craft = Craft::new(
            0, 20, 0, 20, 40,
        );

        let rng = &mut always_success();

        let _ = craft.apply_operation(basic_quality, rng);
        let _ = craft.apply_operation(basic_quality, rng);
        let _ = craft.apply_operation(basic_progress, rng);
        let _ = craft.apply_operation(basic_progress, rng);

        assert_eq!(CraftState::HighQuality, craft.state());
    }

    #[test]
    fn finishing_progress_prevents_operations() {
        let basic_progress = &basic_progress();

        let mut craft = Craft::new(
            0, 20, 0, 20, 30,
        );

        let rng = &mut always_success();

        assert_eq!(Ok(()), craft.apply_operation(basic_progress, rng));
        assert_eq!(Ok(()), craft.apply_operation(basic_progress, rng));
        assert_eq!(Err(Error), craft.apply_operation(basic_progress, rng));
    }

    #[test]
    fn sometimes_operations_fail() {
        let basic_progress = &basic_progress();
        let chance_quality = &chance_quality();

        let mut craft = Craft::new(
            0, 30, 0, 10, 20
        );

        let rng = &mut mock_rand::IterRng::new_from_vec(vec![
            mock_rand::gen_uniform_sample_u32(1, 100, 75),
            mock_rand::gen_uniform_sample_u32(1, 100, 1),
            mock_rand::gen_uniform_sample_u32(1, 30 * 30, 1),
        ]);

        let _ = craft.apply_operation(chance_quality, rng);
        let _ = craft.apply_operation(basic_progress, rng);

        assert_eq!(CraftState::LowQuality, craft.state());
    }
}