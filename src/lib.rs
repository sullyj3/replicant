// Adapted from the sine-synth example from vst-rs
// author: Rob Saunders <hello@robsaunders.io>

#[macro_use]
extern crate vst;

use vst::api::{Events, Supported};
use vst::buffer::AudioBuffer;
use vst::event::Event;
use vst::plugin::{CanDo, Category, Info, Plugin};

use std::f64::consts::PI;
use log::{LevelFilter, debug};

mod envelope;

use envelope::ADSREnvelope;

/// Convert the midi note's pitch into the equivalent frequency.
///
/// This function assumes A4 is 440hz.
fn midi_pitch_to_freq(pitch: u8) -> f64 {
    const A4_PITCH: i8 = 69;
    const A4_FREQ: f64 = 440.0;

    // Midi notes can be 0-127
    ((f64::from(pitch as i8 - A4_PITCH)) / 12.).exp2() * A4_FREQ
}

// struct PolyNote {
//     note: u8,
//     envelope: ADSREnvelope,
// }
// 
// struct Polyreplicant {
//     sample_rate: f64,
//     time: f64,
//     notes: Vec<Voice>,
//     envelope: ADSREnvelope,
// }

struct MonoReplicant {
    sample_rate: f64,
    time: f64,
    note: u8,
    envelope: ADSREnvelope,
}

impl MonoReplicant {
    fn time_per_sample(&self) -> f64 {
        1.0 / self.sample_rate
    }

    /// Process an incoming midi event.
    ///
    /// The midi data is split up like so:
    ///
    /// `data[0]`: Contains the status and the channel. Source: [source]
    /// `data[1]`: Contains the supplemental data for the message - so, if this was a NoteOn then
    ///            this would contain the note.
    /// `data[2]`: Further supplemental data. Would be velocity in the case of a NoteOn message.
    ///
    /// [source]: http://www.midimountain.com/midi/midi_status.htm
    fn process_midi_event(&mut self, data: [u8; 3]) {
        match data[0] {
            128 => self.note_off(data[1]),
            144 => self.note_on(data[1]),
            _ => (),
        }
    }

    fn note_on(&mut self, note: u8) {
        self.envelope.note_on(self.envelope.alpha());
        self.note = note;
    }

    fn note_off(&mut self, _note: u8) {
        self.envelope.note_off();
    }
}

pub const TAU: f64 = PI * 2.0;

impl Default for MonoReplicant {
    fn default() -> MonoReplicant {
        MonoReplicant {
            sample_rate: 44100.0,
            time: 0.0,
            envelope: ADSREnvelope::default(),
            note: 0, // this should never be audible before it is set to something else by note_on()
        }
    }
}

impl Plugin for MonoReplicant {
    fn get_info(&self) -> Info {
        Info {
            name: "Replicant".to_string(),
            vendor: "James Sully".to_string(),
            unique_id: 144_153_144,
            category: Category::Synth,
            inputs: 0,
            outputs: 2,
            parameters: 0,
            initial_delay: 0,
            ..Info::default()
        }
    }

    fn init(&mut self) {
        simple_logging::log_to_file("C:/Users/James/Desktop/replicant.log", LevelFilter::Off);
    }

    #[allow(unused_variables)]
    #[allow(clippy::single_match)]
    fn process_events(&mut self, events: &Events) {
        for event in events.events() {
            match event {
                Event::Midi(ev) => self.process_midi_event(ev.data),
                // More events can be handled here.
                _ => (),
            }
        }
    }

    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = f64::from(rate);
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let samples = buffer.samples();
        let (_, mut outputs) = buffer.split();
        let output_count = outputs.len();
        let time_per_sample = self.time_per_sample();
        let mut output_sample_left;
        let mut output_sample_right;

        for sample_idx in 0..samples {
            let time = self.time;

            let note = self.note;

            // simple stereo effect
            let signal_left = (time+0.01 * midi_pitch_to_freq(note)*0.99 * TAU).sin();
            let signal_right = (time * midi_pitch_to_freq(note)*1.01 * TAU).sin();

            debug!("calling envelope.alpha()");

            // should be 0.0 if release phase is over
            let alpha = self.envelope.alpha();
            debug!("phase: {:?}, phase_elapsed: {:?}, alpha: {:?}",
                   self.envelope.current_phase, self.envelope.phase_elapsed, alpha);

            output_sample_left = (signal_left * alpha) as f32;
            output_sample_right = (signal_right * alpha) as f32;

            self.time += time_per_sample;
            self.envelope.inc_timer(time_per_sample);

            let buff_left = outputs.get_mut(0);
            let buff_right = outputs.get_mut(1);

            buff_left[sample_idx] = output_sample_left;
            buff_right[sample_idx] = output_sample_right;
        }
    }

    fn can_do(&self, can_do: CanDo) -> Supported {
        match can_do {
            CanDo::ReceiveMidiEvent => Supported::Yes,
            _ => Supported::Maybe,
        }
    }
}

plugin_main!(MonoReplicant);

#[cfg(test)]
mod tests {
    use midi_pitch_to_freq;

    #[test]
    fn test_midi_pitch_to_freq() {
        for i in 0..127 {
            // expect no panics
            midi_pitch_to_freq(i);
        }
    }
}
