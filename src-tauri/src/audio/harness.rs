use std::path::Path;

use anyhow::Context;
use ebur128::{EbuR128, Mode};
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use serde::Serialize;

use super::{nodes::Chain, ChainParams};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessReport {
    pub samples: usize,
    pub duration_seconds: f64,
    pub input_peak_dbfs: Option<f64>,
    pub output_peak_dbfs: Option<f64>,
    pub integrated_lufs: Option<f64>,
}

pub fn process_wav(
    input_path: &Path,
    output_path: &Path,
    params: ChainParams,
) -> anyhow::Result<ProcessReport> {
    let input_canonical = input_path
        .canonicalize()
        .with_context(|| format!("Eingabe-WAV nicht gefunden: {}", input_path.display()))?;
    let output_parent = output_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .canonicalize()
        .with_context(|| format!("Ausgabeordner nicht gefunden: {}", output_path.display()))?;
    let output_name = output_path
        .file_name()
        .context("Ausgabepfad muss einen Dateinamen enthalten.")?;
    let output_absolute = if output_path.exists() {
        output_path.canonicalize()?
    } else {
        output_parent.join(output_name)
    };
    anyhow::ensure!(
        input_canonical != output_absolute,
        "Eingabe und Ausgabe dürfen nicht dieselbe Datei sein."
    );

    let mut reader = WavReader::open(&input_canonical)
        .with_context(|| format!("WAV-Datei kann nicht gelesen werden: {}", input_path.display()))?;
    let input_spec = reader.spec();
    anyhow::ensure!(
        input_spec.channels == 1 || input_spec.channels == 2,
        "Unterstützt werden nur Mono- und Stereo-WAV-Dateien."
    );
    params.validate(input_spec.sample_rate)?;

    let samples = match input_spec.sample_format {
        SampleFormat::Float => reader
            .samples::<f32>()
            .map(|sample| sample.map_err(anyhow::Error::from))
            .collect::<anyhow::Result<Vec<_>>>()?,
        SampleFormat::Int => {
            let scale = 2.0_f32.powi(input_spec.bits_per_sample as i32 - 1);
            reader
                .samples::<i32>()
                .map(|sample| {
                    sample
                        .map(|value| value as f32 / scale)
                        .map_err(anyhow::Error::from)
                })
                .collect::<anyhow::Result<Vec<_>>>()?
        }
    };
    anyhow::ensure!(
        !samples.is_empty() && samples.len() % input_spec.channels as usize == 0,
        "Die WAV-Datei enthält keine vollständigen Audioframes."
    );
    anyhow::ensure!(
        samples.iter().all(|sample| sample.is_finite()),
        "WAV enthält NaN oder unendliche Samples."
    );

    let output_spec = WavSpec {
        channels: input_spec.channels,
        sample_rate: input_spec.sample_rate,
        bits_per_sample: 32,
        sample_format: SampleFormat::Float,
    };
    let mut writer = WavWriter::create(&output_absolute, output_spec)
        .with_context(|| format!("Ausgabe-WAV kann nicht erstellt werden: {}", output_path.display()))?;

    let channel_count = input_spec.channels as usize;
    let mut chains = (0..channel_count)
        .map(|_| Chain::new(params.clone(), input_spec.sample_rate))
        .collect::<anyhow::Result<Vec<_>>>()?;
    let mut meter = EbuR128::new(
        input_spec.channels as u32,
        input_spec.sample_rate,
        Mode::I,
    )
    .map_err(|error| anyhow::anyhow!("Loudness-Meter konnte nicht gestartet werden: {error}"))?;
    let mut input_peak = 0.0_f32;
    let mut output_peak = 0.0_f32;
    let mut output_samples = Vec::with_capacity(samples.len());

    for frame in samples.chunks_exact(channel_count) {
        for (channel, input) in frame.iter().copied().enumerate() {
            input_peak = input_peak.max(input.abs());
            let output = chains[channel].process_sample(input);
            output_peak = output_peak.max(output.abs());
            output_samples.push(output);
            writer.write_sample(output)?;
        }
    }
    meter
        .add_frames_f32(&output_samples)
        .map_err(|error| anyhow::anyhow!("Loudness-Messung fehlgeschlagen: {error}"))?;
    writer.finalize().context("Ausgabe-WAV konnte nicht abgeschlossen werden.")?;

    let frame_count = samples.len() / channel_count;
    let integrated_lufs = meter
        .loudness_global()
        .ok()
        .filter(|value| value.is_finite());
    Ok(ProcessReport {
        samples: frame_count,
        duration_seconds: frame_count as f64 / input_spec.sample_rate as f64,
        input_peak_dbfs: peak_to_dbfs(input_peak),
        output_peak_dbfs: peak_to_dbfs(output_peak),
        integrated_lufs,
    })
}

fn peak_to_dbfs(peak: f32) -> Option<f64> {
    if peak <= 0.0 {
        None
    } else {
        Some(20.0 * (peak as f64).log10())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    use hound::{SampleFormat, WavSpec, WavWriter};

    use super::process_wav;
    use crate::audio::ChainParams;

    static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(0);

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "vocal-chain-harness-{}-{}",
                std::process::id(),
                NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).expect("test directory should be created");
            Self(path)
        }

        fn path(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).expect("test directory should be removed");
        }
    }

    fn write_nan_wav(path: &Path) {
        let spec = WavSpec {
            channels: 1,
            sample_rate: 48_000,
            bits_per_sample: 32,
            sample_format: SampleFormat::Float,
        };
        let mut writer = WavWriter::create(path, spec).expect("input WAV should be created");
        writer
            .write_sample(0.25_f32)
            .expect("finite sample should be written");
        writer
            .write_sample(f32::NAN)
            .expect("NaN sample should be written");
        writer.finalize().expect("input WAV should be finalized");
    }

    #[test]
    fn invalid_samples_do_not_create_or_truncate_output() {
        let dir = TestDir::new();
        let input = dir.path("invalid.wav");
        let output = dir.path("output.wav");
        write_nan_wav(&input);

        let original_output = b"existing output must remain untouched";
        fs::write(&output, original_output).expect("existing output should be written");
        let result = process_wav(&input, &output, ChainParams::default());

        assert!(result.is_err());
        assert_eq!(
            fs::read(&output).expect("existing output should remain readable"),
            original_output
        );

        let new_output = dir.path("new-output.wav");
        let result = process_wav(&input, &new_output, ChainParams::default());

        assert!(result.is_err());
        assert!(!new_output.exists());
    }
}
