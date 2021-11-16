use std::str::FromStr;

use log::{info, LevelFilter};
use module::PitchGeneratorType;
use nannou::ui::widget::range_slider::Edge;
use nannou::ui::widget::*;
use nannou::{prelude::*, ui::widget::drop_down_list::Idx, ui::*, Ui};
use pitch_calc::{Letter, LetterOctave, Step};
use sequencer::{Sequencer, SequencerConfiguration};
use simple_logger::SimpleLogger;

use crate::module::format_letter_octave;

mod module;
mod sequencer;

const WIDGET_COLOR: Color = Color::Rgba(0.3, 0.3, 0.3, 1.0);
const LABEL_COLOR: Color = Color::Rgba(1.0, 1.0, 1.0, 1.0);
const CANVAS_COLOR: Color = color::LIGHT_PURPLE;
const MELODY_PITCH_MIN_VALUE: LetterOctave = LetterOctave(Letter::C, 0);
const MELODY_PITCH_MAX_VALUE: LetterOctave = LetterOctave(Letter::C, 7);
const MELODY_MIN_PITCH_DEFAULT_VALUE: LetterOctave = LetterOctave(Letter::C, 3);
const MELODY_MAX_PITCH_DEFAULT_VALUE: LetterOctave = LetterOctave(Letter::C, 5);
const MELODY_PITCH_GENERATOR_TYPE_DEFAULT_VALUE: Idx = 0;
const MELODY_PITCH_GENERATOR_CYCLE_LENGTH_DEFAULT_VALUE: f32 = 64.0;
const TRANSPOSITION_MIN_VALUE: Step = Step(0.0);
const TRANSPOSITION_MAX_VALUE: Step = Step(24.0);
const TRANSPOSITION_MIN_PITCH_DEFAULT_VALUE: Step = Step(0.0);
const TRANSPOSITION_MAX_PITCH_DEFAULT_VALUE: Step = Step(12.0);
const TRANSPOSITION_PITCH_GENERATOR_TYPE_DEFAULT_VALUE: Idx = 1;
const TRANSPOSITION_PITCH_GENERATOR_CYCLE_LENGTH_DEFAULT_VALUE: f32 = 128.0;
const BPM_DEFAULT_VALUE: f32 = 120.0;
const TRIGGER_PROBABILITY_DEFAULT_VALUE: f32 = 1.0;
const TRIGGER_PROBABILITY_MIN_VALUE: f32 = 0.0;
const TRIGGER_PROBABILITY_MAX_VALUE: f32 = 1.0;
const CLOCK_DIVIDER_FACTOR_DEFAULT_VALUE: f32 = 16.0;
const CLOCK_DIVIDER_FACTOR_MIN_VALUE: f32 = 1.0;
const CLOCK_DIVIDER_FACTOR_MAX_VALUE: f32 = 24.0;
const PITCH_GENERATOR_CYCLE_LENGTH_MIN_VALUE: f32 = 1.0;
const PITCH_GENERATOR_CYCLE_LENGTH_MAX_VALUE: f32 = 128.0;
const PITCH_GENERATOR_TYPE_NAMES: &[&str] = &["Ramp", "Square", "Random"];
const QUANTIZER_SCALE_INDEX_DEFAULT_VALUE: Idx = 1;
const QUANTIZER_SCALES: &[&[Letter]] = &[
    module::CHROMATIC_SCALE_NOTES,
    module::MAJOR_SCALE_NOTES,
    module::MINOR_SCALE_NOTES,
    module::MAJOR_PENTATONIC_SCALE_NOTES,
    module::MINOR_PENTATONIC_SCALE_NOTES,
];
const QUANTIZER_SCALE_NAMES: &[&str] = &[
    "Chromatic",
    "Major",
    "Minor",
    "Major Pentatonic",
    "Minor Pentatonic",
];

fn main() {
    // Disable logging for all dependencies
    SimpleLogger::new()
        .with_level(LevelFilter::Off)
        .with_module_level("adc21", LevelFilter::Info)
        .init()
        .unwrap();
    // Run the app
    nannou::app(model).update(update).run();
}

#[derive(Clone)]
pub struct SequencerModel {
    melody_min_pitch: f32,
    melody_max_pitch: f32,
    melody_pitch_generator_type_index: Option<Idx>,
    melody_cycle_length: f32,
    transposition_min_pitch: f32,
    transposition_max_pitch: f32,
    transposition_pitch_generator_type_index: Option<Idx>,
    transposition_cycle_length: f32,
    trigger_probability: f32,
    clock_divider_factor: f32,
    quantizer_scale_index: Option<Idx>,
    bpm: f32,
}

impl From<SequencerModel> for SequencerConfiguration {
    fn from(model: SequencerModel) -> Self {
        SequencerConfiguration {
            melody_min_pitch: Step(model.melody_min_pitch).to_letter_octave(),
            melody_max_pitch: Step(model.melody_max_pitch).to_letter_octave(),
            melody_pitch_generator_type: pitch_generator_type_from_index(
                model.melody_pitch_generator_type_index,
            ),
            melody_cycle_length: model.melody_cycle_length as u32,
            transposition_min_pitch: Step(model.transposition_min_pitch).to_letter_octave(),
            transposition_max_pitch: Step(model.transposition_max_pitch).to_letter_octave(),
            transposition_pitch_generator_type: pitch_generator_type_from_index(
                model.transposition_pitch_generator_type_index,
            ),
            transposition_cycle_length: model.transposition_cycle_length as u32,
            trigger_probablilty: model.trigger_probability,
            clock_divider_factor: model.clock_divider_factor as u32,
            quantizer_scale: QUANTIZER_SCALES[model.quantizer_scale_index.unwrap()].to_vec(),
            bpm: model.bpm,
        }
    }
}

struct Model {
    ui: Ui,
    ids: Ids,
    sequencer: Sequencer,
    sequencer_model: SequencerModel,
    is_playing: bool,
}

// Generate unique widget IDs
widget_ids! {
    struct Ids {
        // widgets
        melody_pitch_range_slider,
        melody_pitch_generator_type_drop_down_list,
        melody_pitch_generator_cycle_length_slider,
        transposition_pitch_range_slider,
        transposition_pitch_generator_type_drop_down_list,
        transposition_pitch_generator_cycle_length_slider,
        is_playing_toggle,
        reset_button,
        trigger_probability_slider,
        clock_divider_factor_slider,
        quantizer_scale_drop_down,
        // layout
        top_level_canvas,
        pitch_canvas,
        pitch_canvas_left_column,
        pitch_canvas_middle_column,
        pitch_canvas_right_column,
        transposition_pitch_canvas,
        transposition_pitch_canvas_left_column,
        transposition_pitch_canvas_middle_column,
        transposition_pitch_canvas_right_column,
        global_canvas,
        global_canvas_left_column,
        global_canvas_middle_column,
        global_canvas_right_column,
        transport_canvas,
        transport_canvas_left_column,
        transport_canvas_right_column
    }
}

fn pitch_generator_type_from_index(idx: Option<Idx>) -> PitchGeneratorType {
    PitchGeneratorType::from_str(PITCH_GENERATOR_TYPE_NAMES[idx.unwrap()]).unwrap()
}


fn model(app: &App) -> Model {
    // Create a window
    let w_id = app
        .new_window()
        .size(900, 300)
        .key_pressed(key_pressed)
        .view(view)
        .build()
        .unwrap();

    // Create the UI for our window
    let mut ui = app.new_ui().window(w_id).build().unwrap();

    // Generate IDs for our widgets
    let ids = Ids::new(ui.widget_id_generator());

    // Create and initialize sequencer
    let sequencer_model = SequencerModel {
        melody_min_pitch: MELODY_MIN_PITCH_DEFAULT_VALUE.step(),
        melody_max_pitch: MELODY_MAX_PITCH_DEFAULT_VALUE.step(),
        melody_pitch_generator_type_index: Some(MELODY_PITCH_GENERATOR_TYPE_DEFAULT_VALUE),
        melody_cycle_length: MELODY_PITCH_GENERATOR_CYCLE_LENGTH_DEFAULT_VALUE,
        transposition_min_pitch: TRANSPOSITION_MIN_PITCH_DEFAULT_VALUE.step(),
        transposition_max_pitch: TRANSPOSITION_MAX_PITCH_DEFAULT_VALUE.step(),
        transposition_pitch_generator_type_index: Some(
            TRANSPOSITION_PITCH_GENERATOR_TYPE_DEFAULT_VALUE,
        ),
        transposition_cycle_length: TRANSPOSITION_PITCH_GENERATOR_CYCLE_LENGTH_DEFAULT_VALUE,
        trigger_probability: TRIGGER_PROBABILITY_DEFAULT_VALUE,
        clock_divider_factor: CLOCK_DIVIDER_FACTOR_DEFAULT_VALUE,
        quantizer_scale_index: Some(QUANTIZER_SCALE_INDEX_DEFAULT_VALUE),
        bpm: BPM_DEFAULT_VALUE,
    };
    let is_playing = true;
    let sequencer = Sequencer::new(sequencer_model.clone().into(), is_playing);

    Model {
        ui: ui,
        ids: ids,
        sequencer,
        sequencer_model,
        is_playing,
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Space => {
            // Toggle sequencer playback
            if model.is_playing {
                info!("Stop sequencer");
                model.is_playing = false;
                model.sequencer.stop()
            } else {
                info!("Start sequencer");
                model.is_playing = true;
                model.sequencer.start()
            }
        }
        _ => (),
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    // Create context for instantiating widgets
    let ui = &mut model.ui.set_widgets();

    // Construct the top level layout
    widget::Canvas::new()
        .flow_down(&[
            (
                model.ids.global_canvas,
                widget::Canvas::new().length(60.0).flow_right(&[
                    (model.ids.global_canvas_left_column, column_canvas()),
                    (model.ids.global_canvas_middle_column, column_canvas()),
                    (model.ids.global_canvas_right_column, column_canvas()),
                ]),
            ),
            (
                model.ids.pitch_canvas,
                widget::Canvas::new().length(60.0).flow_right(&[
                    (
                        model.ids.pitch_canvas_left_column,
                        column_canvas().length_weight(1.0),
                    ),
                    (
                        model.ids.pitch_canvas_middle_column,
                        column_canvas().length_weight(4.0),
                    ),
                    (
                        model.ids.pitch_canvas_right_column,
                        column_canvas().length_weight(3.0),
                    ),
                ]),
            ),
            (
                model.ids.transposition_pitch_canvas,
                widget::Canvas::new().length(60.0).flow_right(&[
                    (
                        model.ids.transposition_pitch_canvas_left_column,
                        column_canvas().length_weight(1.0),
                    ),
                    (
                        model.ids.transposition_pitch_canvas_middle_column,
                        column_canvas().length_weight(4.0),
                    ),
                    (
                        model.ids.transposition_pitch_canvas_right_column,
                        column_canvas().length_weight(3.0),
                    ),
                ]),
            ),
            (
                model.ids.transport_canvas,
                widget::Canvas::new().flow_right(&[
                    (
                        model.ids.transport_canvas_left_column,
                        column_canvas().length_weight(1.0),
                    ),
                    (
                        model.ids.transport_canvas_right_column,
                        column_canvas().length_weight(1.0),
                    ),
                ]),
            ),
        ])
        .color(CANVAS_COLOR)
        .pad(5.0)
        .set(model.ids.top_level_canvas, ui);

    // Create melody pitch generator widgets
    for melody_pitch_generator_type_value in drop_down_list(
        PITCH_GENERATOR_TYPE_NAMES,
        model.sequencer_model.melody_pitch_generator_type_index,
    )
    .padded_wh_of(model.ids.pitch_canvas_left_column, 5.0)
    .middle_of(model.ids.pitch_canvas_left_column)
    .set(model.ids.melody_pitch_generator_type_drop_down_list, ui)
    {
        info!(
            "Set transposition generator type to: {}",
            pitch_generator_type_from_index(Some(melody_pitch_generator_type_value))
        );
        model.sequencer_model.melody_pitch_generator_type_index =
            Some(melody_pitch_generator_type_value);
        model
            .sequencer
            .update_pitch_generator(model.sequencer_model.clone().into());
    }

    let melody_pitch_range_label = format!(
        "Range: {} - {}",
        format_letter_octave(Step(model.sequencer_model.melody_min_pitch).to_letter_octave()),
        format_letter_octave(Step(model.sequencer_model.melody_max_pitch).to_letter_octave())
    );
    for melody_pitch_range_value in range_slider(
        model.sequencer_model.melody_min_pitch,
        model.sequencer_model.melody_max_pitch,
        MELODY_PITCH_MIN_VALUE.step(),
        MELODY_PITCH_MAX_VALUE.step(),
    )
    .padded_wh_of(model.ids.pitch_canvas_middle_column, 5.0)
    .middle_of(model.ids.pitch_canvas_middle_column)
    .label(&melody_pitch_range_label)
    .set(model.ids.melody_pitch_range_slider, ui)
    {
        match melody_pitch_range_value {
            (Edge::Start, min) => {
                let new_value = min.round();
                // only update the sequencer when the value has changed
                if model.sequencer_model.melody_min_pitch != new_value {
                    info!("Set melody pitch range minimum to: {}", new_value);
                    model.sequencer_model.melody_min_pitch = new_value;
                    model
                        .sequencer
                        .update_pitch_generator(model.sequencer_model.clone().into());
                }
            }
            (Edge::End, max) => {
                let new_value = max.round();
                // only update the sequencer when the value has changed
                if model.sequencer_model.melody_max_pitch != new_value {
                    info!("Set melody pitch range maximum to: {}", new_value);
                    model.sequencer_model.melody_max_pitch = new_value;
                    model
                        .sequencer
                        .update_pitch_generator(model.sequencer_model.clone().into());
                }
            }
        }
    }

    // Create cycle length slider when the generator type is not random
    if pitch_generator_type_from_index(model.sequencer_model.melody_pitch_generator_type_index)
        != PitchGeneratorType::Random
    {
        let melody_pitch_generator_cycle_length_label = format!(
            "Cycle length: {}",
            model.sequencer_model.melody_cycle_length as u32
        );
        for melody_pitch_generator_cycle_length_value in slider(
            model.sequencer_model.melody_cycle_length,
            PITCH_GENERATOR_CYCLE_LENGTH_MIN_VALUE,
            PITCH_GENERATOR_CYCLE_LENGTH_MAX_VALUE,
        )
        .padded_wh_of(model.ids.pitch_canvas_right_column, 5.0)
        .middle_of(model.ids.pitch_canvas_right_column)
        .label(&melody_pitch_generator_cycle_length_label)
        .set(model.ids.melody_pitch_generator_cycle_length_slider, ui)
        {
            // quick and dirty way to restrict to multiples of 16
            let new_value = (melody_pitch_generator_cycle_length_value as u32 + 15 & !15) as f32;
            // only update the sequencer when the value has changed
            if model.sequencer_model.melody_cycle_length != new_value {
                info!("Set melody cycle length to: {}", new_value);
                model.sequencer_model.melody_cycle_length = new_value;
                model
                    .sequencer
                    .update_pitch_generator(model.sequencer_model.clone().into());
            }
        }
    }

    // Create transposition pitch generator widgets
    for transposition_pitch_generator_type_value in drop_down_list(
        PITCH_GENERATOR_TYPE_NAMES,
        model
            .sequencer_model
            .transposition_pitch_generator_type_index,
    )
    .padded_wh_of(model.ids.transposition_pitch_canvas_left_column, 5.0)
    .middle_of(model.ids.transposition_pitch_canvas_left_column)
    .set(
        model.ids.transposition_pitch_generator_type_drop_down_list,
        ui,
    ) {
        info!(
            "Set transposition generator type to: {}",
            pitch_generator_type_from_index(Some(transposition_pitch_generator_type_value))
        );
        model
            .sequencer_model
            .transposition_pitch_generator_type_index =
            Some(transposition_pitch_generator_type_value);
        model
            .sequencer
            .update_pitch_generator(model.sequencer_model.clone().into());
    }

    let transposition_pitch_range_label = format!(
        "Range: {} - {}",
        Step(model.sequencer_model.transposition_min_pitch).step(),
        Step(model.sequencer_model.transposition_max_pitch).step(),
    );
    for transposition_pitch_range_value in range_slider(
        model.sequencer_model.transposition_min_pitch,
        model.sequencer_model.transposition_max_pitch,
        TRANSPOSITION_MIN_VALUE.step(),
        TRANSPOSITION_MAX_VALUE.step(),
    )
    .padded_wh_of(model.ids.transposition_pitch_canvas_middle_column, 5.0)
    .middle_of(model.ids.transposition_pitch_canvas_middle_column)
    .label(&transposition_pitch_range_label)
    .set(model.ids.transposition_pitch_range_slider, ui)
    {
        match transposition_pitch_range_value {
            (Edge::Start, min) => {
                let new_value = min.round();
                // only update the sequencer when the value has changed
                if model.sequencer_model.transposition_min_pitch != new_value {
                    info!("Set transposition range minimum to: {}", new_value);
                    model.sequencer_model.transposition_min_pitch = new_value;
                    model
                        .sequencer
                        .update_pitch_generator(model.sequencer_model.clone().into());
                }
            }
            (Edge::End, max) => {
                let new_value = max.round();
                // only update the sequencer when the value has changed
                if model.sequencer_model.transposition_max_pitch != new_value {
                    info!("Set transposition range maximum to: {}", new_value);
                    model.sequencer_model.transposition_max_pitch = new_value;
                    model
                        .sequencer
                        .update_pitch_generator(model.sequencer_model.clone().into());
                }
            }
        }
    }

    // Create cycle length slider when the generator type is not random
    if pitch_generator_type_from_index(
        model
            .sequencer_model
            .transposition_pitch_generator_type_index,
    ) != PitchGeneratorType::Random
    {
        let transposition_pitch_generator_cycle_length_label = format!(
            "Cycle length: {}",
            model.sequencer_model.transposition_cycle_length as u32
        );
        for transposition_pitch_generator_cycle_length_value in slider(
            model.sequencer_model.transposition_cycle_length,
            PITCH_GENERATOR_CYCLE_LENGTH_MIN_VALUE,
            PITCH_GENERATOR_CYCLE_LENGTH_MAX_VALUE,
        )
        .padded_wh_of(model.ids.transposition_pitch_canvas_right_column, 5.0)
        .middle_of(model.ids.transposition_pitch_canvas_right_column)
        .label(&transposition_pitch_generator_cycle_length_label)
        .set(
            model.ids.transposition_pitch_generator_cycle_length_slider,
            ui,
        ) {
            // quick and dirty way to restrict to multiples of 16
            let new_value =
                (transposition_pitch_generator_cycle_length_value as u32 + 15 & !15) as f32;
            // only update the sequencer when the value has changed
            if model.sequencer_model.transposition_cycle_length != new_value {
                info!("Set transposition cycle length to: {}", new_value);
                model.sequencer_model.transposition_cycle_length = new_value;
                model
                    .sequencer
                    .update_pitch_generator(model.sequencer_model.clone().into());
            }
        }
    }

    // Create pitch quantizer scale drop-down list
    for quantizer_scale_value in drop_down_list(
        QUANTIZER_SCALE_NAMES,
        model.sequencer_model.quantizer_scale_index,
    )
    .padded_wh_of(model.ids.global_canvas_left_column, 5.0)
    .middle_of(model.ids.global_canvas_left_column)
    .set(model.ids.quantizer_scale_drop_down, ui)
    {
        // Handle new drop-down list value
        model.sequencer_model.quantizer_scale_index = Some(quantizer_scale_value);
        info!(
            "Set pitch quantizer scale to: {}",
            QUANTIZER_SCALE_NAMES[quantizer_scale_value]
        );
        model
            .sequencer
            .update_pitch_generator(model.sequencer_model.clone().into());
    }

    // Create trigger probability slider
    let trigger_probability_label = format!(
        "Probability: {:.0}%",
        model.sequencer_model.trigger_probability * 100.0
    );
    for trigger_probability_value in slider(
        model.sequencer_model.trigger_probability,
        TRIGGER_PROBABILITY_MIN_VALUE,
        TRIGGER_PROBABILITY_MAX_VALUE,
    )
    .padded_wh_of(model.ids.global_canvas_middle_column, 5.0)
    .middle_of(model.ids.global_canvas_middle_column)
    .label(&trigger_probability_label)
    .set(model.ids.trigger_probability_slider, ui)
    {
        let new_value = (trigger_probability_value * 100.0).round() / 100.0;
        // only update the sequencer when the value has changed
        if model.sequencer_model.trigger_probability != new_value {
            info!("Set trigger probability to: {}", new_value);
            model.sequencer_model.trigger_probability = new_value;
            model
                .sequencer
                .update_trigger_generator(model.sequencer_model.clone().into());
        }
    }

    // Create clock divider factor slider
    let clock_divider_factor_label = format!(
        "Clock division: {}",
        model.sequencer_model.clock_divider_factor as u32
    );
    for clock_divider_factor_value in slider(
        model.sequencer_model.clock_divider_factor,
        CLOCK_DIVIDER_FACTOR_MIN_VALUE,
        CLOCK_DIVIDER_FACTOR_MAX_VALUE,
    )
    .padded_wh_of(model.ids.global_canvas_right_column, 5.0)
    .middle_of(model.ids.global_canvas_right_column)
    .label(&clock_divider_factor_label)
    .set(model.ids.clock_divider_factor_slider, ui)
    {
        let new_value = clock_divider_factor_value.round();
        // only update the sequencer when the value has changed
        if model.sequencer_model.clock_divider_factor != new_value {
            info!("Set clock divider factor to: {}", new_value);
            model.sequencer_model.clock_divider_factor = new_value;
            model
                .sequencer
                .update_trigger_generator(model.sequencer_model.clone().into());
        }
    }

    // Create Play/Pause toggle
    let is_playing_label = if model.is_playing { "Pause" } else { "Play" };
    for is_playing_toggle_value in Toggle::new(model.is_playing)
        .padded_wh_of(model.ids.transport_canvas_right_column, 5.0)
        .middle_of(model.ids.transport_canvas_right_column)
        .label(&is_playing_label)
        .label_font_size(20)
        .color(WIDGET_COLOR)
        .label_color(LABEL_COLOR)
        .border(0.0)
        .set(model.ids.is_playing_toggle, ui)
    {
        // Handle new toggle value
        model.is_playing = is_playing_toggle_value;
        if model.is_playing {
            info!("Start sequencer");
            model.sequencer.start()
        } else {
            info!("Stop sequencer");
            model.sequencer.stop()
        }
    }

    // Create reset button
    for _ in Button::new()
        .padded_wh_of(model.ids.transport_canvas_left_column, 5.0)
        .middle_of(model.ids.transport_canvas_left_column)
        .label("Reset")
        .label_font_size(20)
        .color(WIDGET_COLOR)
        .label_color(LABEL_COLOR)
        .border(0.0)
        .set(model.ids.reset_button, ui)
    {
        info!("Reset sequencer");
        model
            .sequencer
            .update_pitch_generator(model.sequencer_model.clone().into());
        model
            .sequencer
            .update_trigger_generator(model.sequencer_model.clone().into());
    }
}

fn column_canvas() -> Canvas<'static> {
    widget::Canvas::new()
        .color(CANVAS_COLOR)
        .border(0.0)
        .pad(5.0)
}

fn slider(val: f32, min: f32, max: f32) -> widget::Slider<'static, f32> {
    widget::Slider::new(val, min, max)
        .label_font_size(20)
        .color(WIDGET_COLOR)
        .label_color(LABEL_COLOR)
        .border(0.0)
}

fn range_slider(start: f32, end: f32, min: f32, max: f32) -> widget::RangeSlider<'static, f32> {
    widget::RangeSlider::new(start, end, min, max)
        .label_font_size(20)
        .color(WIDGET_COLOR)
        .label_color(LABEL_COLOR)
        .border(0.0)
}

fn drop_down_list(
    items: &'static [&str],
    selected: Option<Idx>,
) -> widget::DropDownList<'static, &'static str> {
    widget::DropDownList::new(items, selected)
        .label_font_size(20)
        .color(WIDGET_COLOR)
        .label_color(LABEL_COLOR)
        .border(0.0)
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();

    // Render the result of our drawing to the window's frame
    draw.to_frame(app, &frame).unwrap();

    // Draw the state of the `Ui` to the frame
    model.ui.draw_to_frame(app, &frame).unwrap();
}
