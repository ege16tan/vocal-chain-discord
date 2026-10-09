use std::f32::consts::PI;

#[derive(Clone, Copy)]
struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Biquad {
    fn high_pass(frequency: f32, sample_rate: u32) -> Self {
        let omega = 2.0 * PI * frequency / sample_rate as f32;
        let cosine = omega.cos();
        let alpha = omega.sin() / (2.0 * std::f32::consts::FRAC_1_SQRT_2);
        let a0 = 1.0 + alpha;
        Self {
            b0: (1.0 + cosine) * 0.5 / a0,
            b1: -(1.0 + cosine) / a0,
            b2: (1.0 + cosine) * 0.5 / a0,
            a1: -2.0 * cosine / a0,
            a2: (1.0 - alpha) / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        output
    }
}

pub struct HighPass {
    filters: [Biquad; 2],
    count: usize,
}

impl HighPass {
    pub fn new(frequency: f32, slope_db: u8, sample_rate: u32) -> Self {
        let filter = Biquad::high_pass(frequency, sample_rate);
        Self {
            filters: [filter; 2],
            count: if slope_db == 24 { 2 } else { 1 },
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let mut output = self.filters[0].process(input);
        if self.count == 2 {
            output = self.filters[1].process(output);
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::HighPass;

    fn steady_gain(frequency: f32) -> f32 {
        let mut filter = HighPass::new(100.0, 12, 48_000);
        let mut sum = 0.0;
        let mut count = 0;
        for index in 0..48_000 {
            let input = (2.0 * std::f32::consts::PI * frequency * index as f32 / 48_000.0)
                .sin();
            let output = filter.process(input);
            if index > 24_000 {
                sum += output * output;
                count += 1;
            }
        }
        (sum / count as f32).sqrt() * std::f32::consts::SQRT_2
    }

    #[test]
    fn high_pass_attenuates_bass_and_passes_voice_band() {
        assert!(steady_gain(50.0) < 0.4);
        assert!(steady_gain(1_000.0) > 0.9);
    }
}
