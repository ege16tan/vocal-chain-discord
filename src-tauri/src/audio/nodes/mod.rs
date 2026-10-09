mod deesser;
mod hpf;
mod limiter;

use serde::{Deserialize, Serialize};

use self::deesser::DeEsser;
use self::hpf::HighPass;
use self::limiter::Limiter;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ChainParams {
    pub hpf_hz: f32,
    pub hpf_slope_db: u8,
    pub hpf_bypass: bool,
    pub deesser_low_hz: f32,
    pub deesser_high_hz: f32,
    pub deesser_threshold_db: f32,
    pub deesser_reduction_db: f32,
    pub deesser_listen: bool,
    pub deesser_bypass: bool,
    pub limiter_ceiling_db: f32,
    pub limiter_release_ms: f32,
    pub limiter_lookahead_ms: f32,
    pub limiter_bypass: bool,
}

impl Default for ChainParams {
    fn default() -> Self {
        Self {
            hpf_hz: 100.0,
            hpf_slope_db: 12,
            hpf_bypass: false,
            deesser_low_hz: 5_000.0,
            deesser_high_hz: 8_000.0,
            deesser_threshold_db: -20.0,
            deesser_reduction_db: 6.0,
            deesser_listen: false,
            deesser_bypass: false,
            limiter_ceiling_db: -1.0,
            limiter_release_ms: 50.0,
            limiter_lookahead_ms: 1.0,
            limiter_bypass: false,
        }
    }
}

impl ChainParams {
    pub fn validate(&self, sample_rate: u32) -> anyhow::Result<()> {
        anyhow::ensure!(sample_rate >= 8_000, "Sample-Rate muss mindestens 8 kHz sein.");
        anyhow::ensure!(
            (20.0..=200.0).contains(&self.hpf_hz),
            "HPF-Frequenz muss zwischen 20 und 200 Hz liegen."
        );
        anyhow::ensure!(
            self.hpf_slope_db == 12 || self.hpf_slope_db == 24,
            "HPF-Flanke muss 12 oder 24 dB/Oktave sein."
        );
        anyhow::ensure!(
            (3_000.0..=12_000.0).contains(&self.deesser_low_hz)
                && (3_000.0..=12_000.0).contains(&self.deesser_high_hz)
                && self.deesser_low_hz < self.deesser_high_hz
                && self.deesser_high_hz < sample_rate as f32 * 0.49,
            "De-Esser-Band muss aufsteigend zwischen 3 und 12 kHz liegen."
        );
        anyhow::ensure!(
            (-60.0..=0.0).contains(&self.deesser_threshold_db),
            "De-Esser-Schwelle muss zwischen -60 und 0 dB liegen."
        );
        anyhow::ensure!(
            (0.0..=24.0).contains(&self.deesser_reduction_db),
            "De-Esser-Reduktion muss zwischen 0 und 24 dB liegen."
        );
        anyhow::ensure!(
            (-6.0..=0.0).contains(&self.limiter_ceiling_db),
            "Limiter-Ceiling muss zwischen -6 und 0 dB liegen."
        );
        anyhow::ensure!(
            (1.0..=500.0).contains(&self.limiter_release_ms),
            "Limiter-Release muss zwischen 1 und 500 ms liegen."
        );
        anyhow::ensure!(
            (0.0..=20.0).contains(&self.limiter_lookahead_ms),
            "Limiter-Lookahead muss zwischen 0 und 20 ms liegen."
        );
        Ok(())
    }
}

pub struct Chain {
    params: ChainParams,
    hpf: HighPass,
    deesser: DeEsser,
    limiter: Limiter,
}

impl Chain {
    pub fn new(params: ChainParams, sample_rate: u32) -> anyhow::Result<Self> {
        params.validate(sample_rate)?;
        Ok(Self {
            hpf: HighPass::new(params.hpf_hz, params.hpf_slope_db, sample_rate),
            deesser: DeEsser::new(&params, sample_rate),
            limiter: Limiter::new(&params, sample_rate),
            params,
        })
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        let mut sample = input;
        if !self.params.hpf_bypass {
            sample = self.hpf.process(sample);
        }
        if !self.params.deesser_bypass {
            sample = self.deesser.process(sample);
        }
        if !self.params.limiter_bypass {
            sample = self.limiter.process(sample);
        }
        sample
    }
}

#[cfg(test)]
mod tests {
    use super::{Chain, ChainParams};

    #[test]
    fn rejects_invalid_effect_parameters() {
        let params = ChainParams {
            hpf_hz: 0.0,
            ..ChainParams::default()
        };
        assert!(Chain::new(params, 48_000).is_err());
    }

    #[test]
    fn limiter_keeps_output_samples_under_ceiling() {
        let params = ChainParams {
            hpf_bypass: true,
            deesser_bypass: true,
            limiter_lookahead_ms: 0.0,
            ..ChainParams::default()
        };
        let mut chain = Chain::new(params, 48_000).unwrap();
        let ceiling = 10.0_f32.powf(-1.0 / 20.0);
        for index in 0..2_000 {
            let input = if index % 31 == 0 { 1.5 } else { 0.3 };
            assert!(chain.process_sample(input).abs() <= ceiling + 1.0e-5);
        }
    }
}
