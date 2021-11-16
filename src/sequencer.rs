use std::{sync::mpsc, thread::sleep};

use chrono::Duration;
use log::info;
use pitch_calc::{Letter, LetterOctave};
use timer::Timer;

use midir::MidiOutputConnection;

use crate::module::{
    format_letter_octave, ClockDivider, PitchAdder, PitchGeneratorType, PitchModule,
    PitchQuantizer, RampPitchGenerator, RandomPitchGenerator, RandomTriggerGenerator,
    SquarePitchGenerator, Trigger, TriggerModule,
};

const TICKS_PER_QUARTER_NOTE: u32 = 24;

pub struct SequencerConfiguration {
    pub melody_min_pitch: LetterOctave,
    pub melody_max_pitch: LetterOctave,
    pub melody_pitch_generator_type: PitchGeneratorType,
    pub melody_cycle_length: u32,
    pub transposition_min_pitch: LetterOctave,
    pub transposition_max_pitch: LetterOctave,
    pub transposition_pitch_generator_type: PitchGeneratorType,
    pub transposition_cycle_length: u32,
    pub trigger_probablilty: f32,
    pub clock_divider_factor: u32,
    pub quantizer_scale: Vec<Letter>,
    pub bpm: f32,
}

enum SequencerCommand {
    Start,
    Stop,
    SetPitchGenerator(Box<dyn PitchModule>),
    SetTriggerGenerator(Box<dyn TriggerModule>),
}

pub struct Sequencer {
    sender: mpsc::Sender<SequencerCommand>,
    _timer: Timer,
}

impl Sequencer {
    pub fn new(config: SequencerConfiguration, is_playing: bool) -> Sequencer {
        // Create async communication channel to the sequencer thread
        let (tx, rx) = mpsc::channel();
        let mut thread = SequencerThread::new(
            rx,
            Sequencer::build_pitch_generator(&config),
            Sequencer::build_trigger_generator(&config),
            is_playing,
        );

        // Schedule the sequencer thread
        let timer = Timer::new();
        let guard = timer.schedule_repeating(
            Duration::milliseconds((60_000.0 / config.bpm / TICKS_PER_QUARTER_NOTE as f32) as i64),
            move || thread.tick(),
        );
        guard.ignore();

        Sequencer {
            sender: tx,
            _timer: timer,
        }
    }

    pub fn start(&self) {
        info!("Start");
        self.sender.send(SequencerCommand::Start).unwrap();
    }

    pub fn stop(&self) {
        info!("Stop");
        self.sender.send(SequencerCommand::Stop).unwrap();
    }

    fn build_pitch_generator(config: &SequencerConfiguration) -> Box<dyn PitchModule> {
        let melody_pitch_generator: Box<dyn PitchModule> = match config.melody_pitch_generator_type
        {
            PitchGeneratorType::Random => Box::new(RandomPitchGenerator::new(
                config.melody_min_pitch,
                config.melody_max_pitch,
            )),
            PitchGeneratorType::RampUp => Box::new(RampPitchGenerator::new(
                config.melody_cycle_length as u32,
                config.melody_min_pitch,
                config.melody_max_pitch,
            )),
            PitchGeneratorType::Square => Box::new(SquarePitchGenerator::new(
                config.melody_cycle_length as u32,
                config.melody_min_pitch,
                config.melody_max_pitch,
            )),
        };
        let transposition_pitch_generator: Box<dyn PitchModule> =
            match config.transposition_pitch_generator_type {
                PitchGeneratorType::Random => Box::new(RandomPitchGenerator::new(
                    config.transposition_min_pitch,
                    config.transposition_max_pitch,
                )),
                PitchGeneratorType::RampUp => Box::new(RampPitchGenerator::new(
                    config.transposition_cycle_length as u32,
                    config.transposition_min_pitch,
                    config.transposition_max_pitch,
                )),
                PitchGeneratorType::Square => Box::new(SquarePitchGenerator::new(
                    config.transposition_cycle_length as u32,
                    config.transposition_min_pitch,
                    config.transposition_max_pitch,
                )),
            };

        Box::new(PitchQuantizer::new(
            Box::new(PitchAdder::new(
                melody_pitch_generator,
                transposition_pitch_generator,
            )),
            config.quantizer_scale.clone(),
        ))
    }

    fn build_trigger_generator(config: &SequencerConfiguration) -> Box<dyn TriggerModule> {
        Box::new(ClockDivider::new(
            Box::new(RandomTriggerGenerator::new(config.trigger_probablilty)),
            config.clock_divider_factor,
        ))
    }

    pub fn update_pitch_generator(&self, config: SequencerConfiguration) {
        self.sender
            .send(SequencerCommand::SetPitchGenerator(
                Sequencer::build_pitch_generator(&config),
            ))
            .unwrap();
    }

    pub fn update_trigger_generator(&self, config: SequencerConfiguration) {
        self.sender
            .send(SequencerCommand::SetTriggerGenerator(
                Sequencer::build_trigger_generator(&config),
            ))
            .unwrap();
    }
}

struct SequencerThread {
    receiver: mpsc::Receiver<SequencerCommand>,
    pitch_generator: Box<dyn PitchModule>,
    trigger_generator: Box<dyn TriggerModule>,
    midi_output_conn: MidiOutputConnection,
    is_playing: bool,
}

impl SequencerThread {
    fn new(
        receiver: mpsc::Receiver<SequencerCommand>,
        pitch_generator: Box<dyn PitchModule>,
        trigger_generator: Box<dyn TriggerModule>,
        is_playing: bool,
    ) -> SequencerThread {
        // Create MIDI output
        let midi_out = midir::MidiOutput::new("Nannou Generative Sequencer").unwrap();
        info!("Available MIDI output ports:");
        for (i, p) in midi_out.ports().iter().enumerate() {
            info!("\t{}: {}", i, midi_out.port_name(p).unwrap());
        }
        // Connect to the first available MIDI output port (IAC Bus 1)
        let out_port = &midi_out.ports()[0];
        info!("Connecting to {}", midi_out.port_name(out_port).unwrap());
        let out_conn = midi_out
            .connect(out_port, "Nannou Generative Sequencer")
            .unwrap();

        SequencerThread {
            receiver,
            pitch_generator,
            trigger_generator,
            midi_output_conn: out_conn,
            is_playing: is_playing,
        }
    }

    fn tick(&mut self) {
        // Process all pending commands
        for command in self.receiver.try_iter() {
            match command {
                SequencerCommand::Start => {
                    if !self.is_playing {
                        self.is_playing = true
                    }
                }
                SequencerCommand::Stop => {
                    if self.is_playing {
                        self.is_playing = false
                    }
                }
                SequencerCommand::SetPitchGenerator(pg) => {
                    self.pitch_generator = pg;
                }
                SequencerCommand::SetTriggerGenerator(tg) => {
                    self.trigger_generator = tg;
                }
            };
        }

        // Play note
        if self.is_playing {
            let pitch = self.pitch_generator.tick();
            match self.trigger_generator.tick() {
                Trigger::On => {
                    const NOTE_ON_MSG: u8 = 0x90;
                    const NOTE_OFF_MSG: u8 = 0x80;
                    const VELOCITY: u8 = 0x64;

                    // Play the generated MIDI note
                    let note = pitch.step() as u8;

                    info!("Play note: {}", format_letter_octave(pitch));

                    self.midi_output_conn
                        .send(&[NOTE_ON_MSG, note, VELOCITY])
                        .unwrap();
                    sleep(core::time::Duration::from_millis(5));
                    self.midi_output_conn
                        .send(&[NOTE_OFF_MSG, note, VELOCITY])
                        .unwrap();
                }
                Trigger::Off => (),
            }
        }
    }
}
