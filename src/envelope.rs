use std::fmt;
use vst::util::AtomicFloat;
use std::sync::Arc;

#[derive(PartialEq, Debug)]
pub enum ADSRPhase {
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug)]
pub struct ADSREnvelope {
    pub current_phase: ADSRPhase,
    pub phase_elapsed: f64,

    note_on_volume: f64,
    note_off_volume: f64,
}

impl fmt::Debug for ADSRParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (a,d,s,r) = (self.attack.get(), self.decay.get(), self.sustain.get(), self.release.get());
        write!(f, "ADSRParams({}, {}, {}, {})", a,d,s,r)
    }
}

impl ADSRParams {
    fn new(attack: f32, decay: f32, sustain: f32, release: f32) -> ADSRParams {
        ADSRParams {
            attack: AtomicFloat::new(attack),
            decay: AtomicFloat::new(decay),
            sustain: AtomicFloat::new(sustain), // 0.0 to 1.0
            release: AtomicFloat::new(release),
        }
    }
}

impl ADSREnvelope {
    pub fn new() -> ADSREnvelope {
        let (attack, decay, sustain, release) = (0.0005, 0.0005, 1.0, 0.0005);

        ADSREnvelope {

            // we begin at the "end of the release phase" - nothing plays.
            current_phase: ADSRPhase::Release,
            phase_elapsed: release.into(),

            note_on_volume: 0.0,

            // todo: Arc
            params: Arc::new(ADSRParams::new(attack, decay, sustain, release)),

            // this shouldn't be used before being set by note_off()
            note_off_volume: sustain.into(),
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

    pub fn inc_timer(&mut self, dt: f64, attack: f64, decay: f64, sustain: f64, release: f64) {
        self.phase_elapsed += dt;

        // TODO potential bug if dt exceeds the duration of a phase
        if self.current_phase == ADSRPhase::Attack {
            if self.phase_elapsed > params.attack.get().into() {
                self.current_phase = ADSRPhase::Decay;
                let attack: f64 = params.attack.get().into();
                self.phase_elapsed %= attack;
            }
        }

        // theoretically, could go straight from attack to sustain in one 
        // inc_time() call if dt is large
        if self.current_phase == ADSRPhase::Decay {
            if self.phase_elapsed > params.decay.get().into() {
                self.current_phase = ADSRPhase::Sustain;
                let decay: f64 = params.decay.get().into();
                self.phase_elapsed %= decay;
            }
        }

        // don't need to do anything for sustain or release
    }

    // for now we just lerp. TODO: learn decibels and best curve shapes
    pub fn alpha(&self, params: &ReplicantParameters) -> f64 {
        match self.current_phase {
            ADSRPhase::Attack  => {
                let attack: f64 = params.attack.get().into();
                lerp(self.note_on_volume, 1.0, self.phase_elapsed / attack)
            },
            ADSRPhase::Decay   => {
                let decay: f64 = params.decay.get().into();
                let sustain: f64 = params.sustain.get().into();
                lerp_down(1.0, sustain, self.phase_elapsed / decay)
            },
            ADSRPhase::Sustain => params.sustain.get().into(),
            ADSRPhase::Release => {
                let release: f64 = params.release.get().into();
                let alpha = lerp_down(self.note_off_volume,
                                      0.0,
                                      self.phase_elapsed / release);
                
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
