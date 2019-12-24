
#[derive(PartialEq, Debug)]
pub enum ADSRPhase {
    Attack,
    Decay,
    Sustain,
    Release,
}



impl Default for ADSREnvelope {
    fn default() -> ADSREnvelope {
        ADSREnvelope::new(0.5, 0.5, 0.5, 0.5)
    }
}

#[derive(Debug)]
pub struct ADSREnvelope {
    pub current_phase: ADSRPhase,
    pub phase_elapsed: f64,

    note_on_volume: f64,

    attack: f64,
    decay: f64,
    sustain: f64, // 0.0 to 1.0
    release: f64,

    note_off_volume: f64,
}

impl ADSREnvelope {
    pub fn new(attack: f64, decay: f64, sustain: f64, release: f64) -> ADSREnvelope {
        ADSREnvelope {
            // we begin at the "end of the release phase" - nothing plays.
            current_phase: ADSRPhase::Release,
            phase_elapsed: release,

            note_on_volume: 0.0,

            attack: attack,
            decay: decay,
            sustain: sustain,
            release: release,

            // this shouldn't be used before being set by note_off()
            note_off_volume: sustain,
        }
    }

    pub fn note_on(&mut self, note_on_volume: f64) {
        // note_on_volume exists for the case where there is still audio playing - we don't want to
        // jump to 0 and click, we want to maintain the current volume
        self.note_on_volume = note_on_volume;
        self.current_phase = ADSRPhase::Attack;
        self.phase_elapsed = 0.0;
    }

    pub fn note_off(&mut self) {
        // if we're in the sustain phase, note_off_volume is just the sustain 
        // level. if we're in the attack or decay phase, during release we'll 
        // interpolate down from note_off_volume instead.
        self.note_off_volume = self.alpha();
        self.current_phase = ADSRPhase::Release;
        self.phase_elapsed = 0.0;
    }

    pub fn inc_timer(&mut self, dt: f64) {
        self.phase_elapsed += dt;

        // TODO potential bug if dt exceeds the duration of a phase
        if self.current_phase == ADSRPhase::Attack {
            if self.phase_elapsed > self.attack {
                self.current_phase = ADSRPhase::Decay;
                self.phase_elapsed %= self.attack;
            }
        }

        // theoretically, could go straight from attack to sustain in one 
        // inc_time() call if dt is large
        if self.current_phase == ADSRPhase::Decay {
            if self.phase_elapsed > self.decay {
                self.current_phase = ADSRPhase::Sustain;
                self.phase_elapsed %= self.decay;
            }
        }

        // don't need to do anything for sustain or release
    }

    // for now we just lerp. TODO: learn decibels and best curve shapes
    pub fn alpha(&self) -> f64 {
        match self.current_phase {
            ADSRPhase::Attack  => lerp(self.note_on_volume, 1.0, self.phase_elapsed / self.attack),
            ADSRPhase::Decay   => lerp_down(1.0, self.sustain, self.phase_elapsed / self.decay),
            ADSRPhase::Sustain => self.sustain,
            ADSRPhase::Release => {
                let alpha = lerp_down(self.note_off_volume,
                                      0.0,
                                      self.phase_elapsed / self.release);
                
                // if phase_elapsed is longer than release, clamp to 0 rather than returning a
                // negative value
                clamp(0.0, alpha, 1.0)
                // don't need to do this for other phases, as inc_timer should ensure a phase
                // transition and reset of phase_elapsed whenever the phase_elapsed exceeds that
                // phase's length.
            },

        }
    }
}

fn clamp(a: f64, x: f64, b: f64) -> f64 {
    a.max(x.min(b))
}

// the lerp functions will return values outside a..b for t outside 0..1
fn lerp(a: f64, b:f64, t:f64) -> f64 {
    let result = a + (b - a) * t;
    result
}

fn lerp_down(b: f64, a:f64, t:f64) -> f64 {
    let result = b - (b - a) * t;
    result
}

#[derive(PartialEq)]
pub enum IsDone {
    Continue,
    Done
}
