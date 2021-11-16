use std::{fmt::Display, str::FromStr};

use pitch_calc::*;
use rand::prelude::*;

pub const CHROMATIC_SCALE_NOTES: &[Letter] = &[
    Letter::C,
    Letter::Csh,
    Letter::D,
    Letter::Dsh,
    Letter::E,
    Letter::F,
    Letter::Fsh,
    Letter::G,
    Letter::Gsh,
    Letter::A,
    Letter::Ash,
    Letter::B,
];
pub const MAJOR_SCALE_NOTES: &[Letter] = &[
    Letter::C,
    Letter::D,
    Letter::E,
    Letter::F,
    Letter::G,
    Letter::A,
    Letter::B,
];
pub const MINOR_SCALE_NOTES: &[Letter] = &[
    Letter::C,
    Letter::D,
    Letter::Eb,
    Letter::F,
    Letter::G,
    Letter::Ab,
    Letter::Bb,
];
pub const MAJOR_PENTATONIC_SCALE_NOTES: &[Letter] =
    &[Letter::C, Letter::D, Letter::E, Letter::G, Letter::A];
pub const MINOR_PENTATONIC_SCALE_NOTES: &[Letter] =
    &[Letter::C, Letter::Eb, Letter::F, Letter::G, Letter::Bb];

pub fn format_letter_octave(letter_octave: LetterOctave) -> String {
    let letter_name = match letter_octave.letter() {
        Letter::C => "C",
        Letter::Csh => "C#",
        Letter::Db => "Db",
        Letter::D => "D",
        Letter::Dsh => "D#",
        Letter::Eb => "Eb",
        Letter::E => "E",
        Letter::F => "F",
        Letter::Fsh => "F#",
        Letter::Gb => "Gb",
        Letter::G => "G",
        Letter::Gsh => "G#",
        Letter::Ab => "Ab",
        Letter::A => "A",
        Letter::Ash => "A#",
        Letter::Bb => "Bb",
        Letter::B => "B",
    };
    format!("{}{}", letter_name, letter_octave.octave())
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Trigger {
    Off,
    On,
}

impl Trigger {
    pub fn from_bool(b: bool) -> Trigger {
        if b {
            Trigger::On
        } else {
            Trigger::Off
        }
    }
}

pub trait TriggerModule: Send + Sync {
    fn tick(&mut self) -> Trigger;
}

pub struct RandomTriggerGenerator<R: Rng> {
    rng: R,
    p: f32,
}

impl RandomTriggerGenerator<SmallRng> {
    pub fn new(probability: f32) -> RandomTriggerGenerator<SmallRng> {
        RandomTriggerGenerator {
            rng: SmallRng::from_entropy(),
            p: probability,
        }
    }
}

impl<R: Rng + Send + Sync> TriggerModule for RandomTriggerGenerator<R> {
    fn tick(&mut self) -> Trigger {
        Trigger::from_bool(self.rng.gen_bool(self.p as f64))
    }
}

pub struct ClockDivider {
    factor: u32,
    counter: u32,
    input: Box<dyn TriggerModule>,
}

impl ClockDivider {
    pub fn new(input: Box<dyn TriggerModule>, factor: u32) -> ClockDivider {
        ClockDivider {
            factor: factor,
            counter: 0,
            input: input,
        }
    }
}

impl TriggerModule for ClockDivider {
    fn tick(&mut self) -> Trigger {
        let trigger = if self.counter % self.factor == 0 {
            self.counter = 0;
            self.input.tick()
        } else {
            Trigger::Off
        };
        self.counter += 1;
        trigger
    }
}

#[derive(PartialEq)]
pub enum PitchGeneratorType {
    Random,
    RampUp,
    Square,
}

impl Display for PitchGeneratorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            PitchGeneratorType::Random => write!(f, "Random"),
            PitchGeneratorType::RampUp => write!(f, "Ramp"),
            PitchGeneratorType::Square => write!(f, "Square"),
        }
    }
}

impl FromStr for PitchGeneratorType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Random" => Ok(PitchGeneratorType::Random),
            "Ramp" => Ok(PitchGeneratorType::RampUp),
            "Square" => Ok(PitchGeneratorType::Square),
            _ => Err(()),
        }
    }
}

pub trait PitchModule: Send + Sync {
    fn tick(&mut self) -> LetterOctave;
}

pub struct RandomPitchGenerator<R: Rng + Send + Sync> {
    rng: R,
    min: f32,
    max: f32,
}

impl<R: Rng + Send + Sync> PitchModule for RandomPitchGenerator<R> {
    fn tick(&mut self) -> LetterOctave {
        if self.min != self.max {
            let r: f32 = self.rng.gen_range(self.min..self.max);
            Step(r).to_letter_octave()
        } else {
            Step(self.min).to_letter_octave()
        }
    }
}

impl RandomPitchGenerator<SmallRng> {
    pub fn new(min: LetterOctave, max: LetterOctave) -> RandomPitchGenerator<SmallRng> {
        RandomPitchGenerator {
            rng: SmallRng::from_entropy(),
            min: min.step(),
            max: max.step(),
        }
    }
}

pub struct RampPitchGenerator {
    cycle_length: u32,
    min: f32,
    max: f32,
    counter: u32,
}

impl RampPitchGenerator {
    pub fn new(cycle_length: u32, min: LetterOctave, max: LetterOctave) -> RampPitchGenerator {
        RampPitchGenerator {
            cycle_length,
            min: min.step(),
            max: max.step(),
            counter: 0,
        }
    }
}

impl PitchModule for RampPitchGenerator {
    fn tick(&mut self) -> LetterOctave {
        let slope = if self.cycle_length > 1 {
            (self.max - self.min) / (self.cycle_length - 1) as f32
        } else {
            0.
        };
        let step = Step(self.min + slope * self.counter as f32);
        let pitch = step.to_letter_octave();
        if self.counter == self.cycle_length - 1 {
            self.counter = 0;
        } else {
            self.counter += 1;
        }
        pitch
    }
}

pub struct SquarePitchGenerator {
    cycle_length: u32,
    min: f32,
    max: f32,
    counter: u32,
}

impl SquarePitchGenerator {
    pub fn new(cycle_length: u32, min: LetterOctave, max: LetterOctave) -> SquarePitchGenerator {
        SquarePitchGenerator {
            cycle_length,
            min: min.step(),
            max: max.step(),
            counter: 0,
        }
    }
}

impl PitchModule for SquarePitchGenerator {
    fn tick(&mut self) -> LetterOctave {
        self.counter += 1;
        let pitch = if self.counter <= self.cycle_length / 2 {
            Step(self.min).to_letter_octave()
        } else {
            if self.counter == self.cycle_length {
                self.counter = 0;
            }
            Step(self.max).to_letter_octave()
        };
        pitch
    }
}

pub struct PitchQuantizer {
    input: Box<dyn PitchModule>,
    enabled_notes: Vec<Letter>,
}

impl PitchQuantizer {
    pub fn new(input: Box<dyn PitchModule>, enabled_notes: Vec<Letter>) -> PitchQuantizer {
        PitchQuantizer {
            input,
            enabled_notes,
        }
    }
}

impl PitchModule for PitchQuantizer {
    fn tick(&mut self) -> LetterOctave {
        let unquantized = self.input.tick();
        self.enabled_notes.sort();
        for enabled_note in &self.enabled_notes {
            if *enabled_note == unquantized.letter() {
                return unquantized;
            } else if *enabled_note > unquantized.letter() {
                // quantize up to the next enabled note
                let quantized = LetterOctave(enabled_note.clone(), unquantized.octave());
                return quantized;
            }
        }

        // handle case when the unquantized note is above the highest enabled note by wrapping around
        let quantized = LetterOctave(self.enabled_notes[0], unquantized.octave() + 1);
        return quantized;
    }
}

pub struct PitchAdder {
    left: Box<dyn PitchModule>,
    right: Box<dyn PitchModule>,
}

impl PitchAdder {
    pub fn new(left: Box<dyn PitchModule>, right: Box<dyn PitchModule>) -> PitchAdder {
        PitchAdder { left, right }
    }
}

impl PitchModule for PitchAdder {
    fn tick(&mut self) -> LetterOctave {
        let right_result = self.right.tick();
        let left_result = self.left.tick();
        let result = left_result + right_result;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_pitch_generator_returns_symmetrical_output_when_length_is_even() {
        let length = 4;
        let min = Step(0.0).to_letter_octave();
        let max = Step(10.0).to_letter_octave();
        let mut generator = SquarePitchGenerator::new(length, min, max);

        let mut actual: Vec<LetterOctave> = Vec::new();
        for _ in 0..length * 2 {
            actual.push(generator.tick());
        }

        assert_eq!(actual, vec![min, min, max, max, min, min, max, max]);
    }

    #[test]
    fn square_pitch_generator_returns_asymmetrical_output_when_length_is_odd() {
        let length = 3;
        let min = LetterOctave(Letter::C, 1);
        let max = LetterOctave(Letter::C, 2);
        let mut generator = SquarePitchGenerator::new(length, min, max);

        let mut actual: Vec<LetterOctave> = Vec::new();
        for _ in 0..length * 2 {
            actual.push(generator.tick());
        }

        assert_eq!(actual, vec![min, max, max, min, max, max]);
    }

    #[test]
    fn ramp_generator_returns_stepped_output_including_min_max_values() {
        let length = 4;
        let min = LetterOctave(Letter::C, 1);
        let max = LetterOctave(Letter::C, 2);
        let mut generator = RampPitchGenerator::new(length, min, max);

        let mut actual: Vec<LetterOctave> = Vec::new();
        for _ in 0..length * 2 {
            actual.push(generator.tick());
        }

        assert_eq!(
            actual,
            vec![
                LetterOctave(Letter::C, 1),
                LetterOctave(Letter::E, 1),
                LetterOctave(Letter::Gsh, 1),
                LetterOctave(Letter::C, 2),
                LetterOctave(Letter::C, 1),
                LetterOctave(Letter::E, 1),
                LetterOctave(Letter::Gsh, 1),
                LetterOctave(Letter::C, 2)
            ]
        );
    }
}
