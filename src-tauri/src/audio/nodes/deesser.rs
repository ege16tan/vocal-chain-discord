use std::f32::consts::PI;

use super::ChainParams;

struct BandPass {
    b0: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BandPass {
    fn new(low_hz: f32, high_hz: f32, sample_rate: u32) -> Self {
        let center = (low_hz * high_hz).sqrt();
        let bandwidth = (high_hz - low_hz).max(1.0);
        let q = (center / bandwidth).clamp(0.2, 20.0);
        let omega = 2.0 * PI * center / sample_rate as f32;
        let alpha = omega.sin() / (2.0 * q);
        let a0 = 1.0 + alpha;
        Self {
            b0: alpha / a0,
            b2: -alpha / a0,
            a1: -2.0 * omega.cos() / a0,
            a2: (1.0 - alpha) / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        output
    }
}

pub struct DeEsser {
    band: BandPass,
    threshold: f32,
    max_reduction_db: f32,
    attack_coeff: f32,
    release_coeff: f32,
    envelope: f32,
    listen: bool,
}

impl DeEsser {
    pub fn new(params: &ChainParams, sample_rate: u32) -> Self {
        Self {
            band: BandPass::new(
                params.deesser_low_hz,
                params.deesser_high_hz,
                sample_rate,
            ),
            threshold: 10.0_f32.powf(params.deesser_threshold_db / 20.0),
            max_reduction_db: params.deesser_reduction_db,
            attack_coeff: (-1.0 / (0.002 * sample_rate as f32)).exp(),
            release_coeff: (-1.0 / (0.080 * sample_rate as f32)).exp(),
            envelope: 0.0,
            listen: params.deesser_listen,
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let band = self.band.process(input);
        let coeff = if band.abs() > self.envelope {
            self.attack_coeff
        } else {
            self.release_coeff
        };
        self.envelope = coeff * self.envelope + (1.0 - coeff) * band.abs();
        if self.listen {
            return band;
        }

        let over_threshold = (self.envelope / self.threshold).max(1.0);
        let drive = (1.0 - 1.0 / over_threshold).clamp(0.0, 1.0);
        let reduction = 10.0_f32.powf(-self.max_reduction_db * drive / 20.0);
        input - band * (1.0 - reduction)
    }
}

#[cfg(test)]
mod tests {
    use super::DeEsser;
    use crate::audio::nodes::ChainParams;

    fn tone_rms(frequency: f32, use_deesser: bool) -> f32 {
        let params = ChainParams {
            deesser_threshold_db: -40.0,
            deesser_reduction_db: 12.0,
            hpf_bypass: true,
            limiter_bypass: true,
            ..ChainParams::default()
        };
        let mut deesser = DeEsser::new(&params, 48_000);
        let mut sum = 0.0;
        let mut count = 0;
        for index in 0..48_000 {
            let input = 0.5 * (2.0 * std::f32::consts::PI * frequency * index as f32 / 48_000.0)
                .sin();
            let output = if use_deesser {
                deesser.process(input)
            } else {
                input
            };
            if index > 24_000 {
                sum += output * output;
                count += 1;
            }
        }
        (sum / count as f32).sqrt()
    }

    #[test]
    fn deesser_reduces_the_sibilance_band() {
        assert!(tone_rms(6_000.0, true) < tone_rms(6_000.0, false) * 0.8);
    }
}
