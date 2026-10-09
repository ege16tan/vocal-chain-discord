use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use vocal_chain_discord_lib::audio::{process_wav, ChainParams};

#[derive(Debug, Parser)]
#[command(
    name = "harness",
    about = "Verarbeitet eine WAV-Datei offline mit der Vocal-Chain."
)]
struct Args {
    #[arg(value_name = "INPUT.wav")]
    input: PathBuf,
    #[arg(value_name = "OUTPUT.wav")]
    output: PathBuf,
    #[arg(long, default_value_t = 100.0)]
    hpf_hz: f32,
    #[arg(long, default_value_t = 12)]
    hpf_slope_db: u8,
    #[arg(long, default_value_t = 5_000.0)]
    deesser_low_hz: f32,
    #[arg(long, default_value_t = 8_000.0)]
    deesser_high_hz: f32,
    #[arg(long, default_value_t = -20.0)]
    deesser_threshold_db: f32,
    #[arg(long, default_value_t = 6.0)]
    deesser_reduction_db: f32,
    #[arg(long, default_value_t = -1.0)]
    limiter_ceiling_db: f32,
    #[arg(long, default_value_t = 50.0)]
    limiter_release_ms: f32,
    #[arg(long, default_value_t = 1.0)]
    limiter_lookahead_ms: f32,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let params = ChainParams {
        hpf_hz: args.hpf_hz,
        hpf_slope_db: args.hpf_slope_db,
        deesser_low_hz: args.deesser_low_hz,
        deesser_high_hz: args.deesser_high_hz,
        deesser_threshold_db: args.deesser_threshold_db,
        deesser_reduction_db: args.deesser_reduction_db,
        limiter_ceiling_db: args.limiter_ceiling_db,
        limiter_release_ms: args.limiter_release_ms,
        limiter_lookahead_ms: args.limiter_lookahead_ms,
        ..ChainParams::default()
    };
    let report = process_wav(&args.input, &args.output, params).with_context(|| {
        format!(
            "WAV konnte nicht verarbeitet werden ({} → {}).",
            args.input.display(),
            args.output.display()
        )
    })?;
    let peak = report
        .output_peak_dbfs
        .map(|value| format!("{value:.1} dBFS"))
        .unwrap_or_else(|| "−∞".to_string());
    let loudness = report
        .integrated_lufs
        .map(|value| format!("{value:.1} LUFS"))
        .unwrap_or_else(|| "nicht messbar".to_string());
    println!(
        "{:.2} s verarbeitet · Peak {} · Integrated {}",
        report.duration_seconds, peak, loudness
    );
    Ok(())
}
