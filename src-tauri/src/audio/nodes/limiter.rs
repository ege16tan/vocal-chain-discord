use std::collections::VecDeque;

use super::ChainParams;

pub struct Limiter {
    delay: VecDeque<f32>,
    delay_samples: usize,
    ceiling: f32,
    release_coeff: f32,
    gain: f32,
}

impl Limiter {
    pub fn new(params: &ChainParams, sample_rate: u32) -> Self {
        let delay_samples =
            (params.limiter_lookahead_ms * sample_rate as f32 / 1_000.0).round() as usize;
        Self {
            delay: std::iter::repeat(0.0).take(delay_samples).collect(),
            delay_samples,
            ceiling: 10.0_f32.powf(params.limiter_ceiling_db / 20.0),
            release_coeff: (-1.0
                / (params.limiter_release_ms * sample_rate as f32 / 1_000.0))
                .exp(),
            gain: 1.0,
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        self.delay.push_back(input);
        let output = self.delay.pop_front().unwrap_or(0.0);
        let peak = interpolated_peak_4x(&self.delay).max(output.abs());
        let target_gain = if peak > self.ceiling {
            self.ceiling / peak
        } else {
            1.0
        };
        if target_gain < self.gain {
            self.gain = target_gain;
        } else {
            self.gain = target_gain + self.release_coeff * (self.gain - target_gain);
        }
        let limited = (output * self.gain).clamp(-self.ceiling, self.ceiling);
        limited
    }

    pub fn lookahead_samples(&self) -> usize {
        self.delay_samples
    }
}

fn interpolated_peak_4x(samples: &VecDeque<f32>) -> f32 {
    if samples.len() < 2 {
        return samples.front().copied().unwrap_or(0.0).abs();
    }
    let mut peak: f32 = 0.0;
    for index in 0..samples.len() {
        let p0 = if index == 0 {
            samples[index]
        } else {
            samples[index - 1]
        };
        let p1 = samples[index];
        let p2 = samples[(index + 1).min(samples.len() - 1)];
        let p3 = samples[(index + 2).min(samples.len() - 1)];
        for phase in 0..4 {
            let t = phase as f32 * 0.25;
            let t2 = t * t;
            let t3 = t2 * t;
            let value = 0.5
                * ((2.0 * p1)
                    + (-p0 + p2) * t
                    + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
                    + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3);
            peak = peak.max(value.abs());
        }
    }
    peak
}

#[cfg(test)]
mod tests {
    use super::Limiter;
    use crate::audio::nodes::ChainParams;

    #[test]
    fn limiter_respects_sample_ceiling_after_lookahead() {
        let params = ChainParams {
            hpf_bypass: true,
            deesser_bypass: true,
            ..ChainParams::default()
        };
        let mut limiter = Limiter::new(&params, 48_000);
        assert_eq!(limiter.lookahead_samples(), 48);
        let ceiling = 10.0_f32.powf(-1.0 / 20.0);
        for index in 0..8_000 {
            let input = if (2_000..2_400).contains(&index) {
                1.4 * (2.0 * std::f32::consts::PI * 997.0 * index as f32 / 48_000.0).sin()
            } else {
                0.1
            };
            assert!(limiter.process(input).abs() <= ceiling + 1.0e-5);
        }
    }
}
