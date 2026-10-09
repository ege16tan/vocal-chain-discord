use std::{
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Context};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize, Device, SampleFormat, SampleRate, Stream, StreamConfig,
};
use crossbeam_channel::{bounded, Receiver, Sender, TryRecvError};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::ChainParams;

const SAMPLE_RATE: u32 = 48_000;
const MIN_BUFFER_FRAMES: u32 = 128;
const MAX_BUFFER_FRAMES: u32 = 1_024;
const RECONNECT_DELAY: Duration = Duration::from_secs(2);
const METER_INTERVAL: Duration = Duration::from_millis(16);

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDevices {
    pub inputs: Vec<AudioDevice>,
    pub outputs: Vec<AudioDevice>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AudioStatus {
    state: &'static str,
    message: Option<String>,
}

#[derive(Default)]
struct AudioMetrics {
    input_peak: AtomicU32,
    output_peak: AtomicU32,
    queue_overruns: AtomicU64,
    queue_underruns: AtomicU64,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AudioMetricsSnapshot {
    input_peak_dbfs: Option<f64>,
    output_peak_dbfs: Option<f64>,
    queue_overruns: u64,
    queue_underruns: u64,
}

impl AudioMetrics {
    fn snapshot(&self) -> AudioMetricsSnapshot {
        AudioMetricsSnapshot {
            input_peak_dbfs: peak_dbfs(f32::from_bits(self.input_peak.swap(0, Ordering::Relaxed))),
            output_peak_dbfs: peak_dbfs(f32::from_bits(
                self.output_peak.swap(0, Ordering::Relaxed),
            )),
            queue_overruns: self.queue_overruns.load(Ordering::Relaxed),
            queue_underruns: self.queue_underruns.load(Ordering::Relaxed),
        }
    }
}

#[derive(Clone)]
struct AudioConfig {
    input_device: String,
    output_device: String,
    buffer_frames: u32,
    params: ChainParams,
}

enum WorkerCommand {
    Start(AudioConfig, Sender<Result<(), String>>),
    Stop(Sender<()>),
}

pub struct AudioEngine {
    commands: Sender<WorkerCommand>,
}

impl AudioEngine {
    pub fn new(app: AppHandle) -> std::io::Result<Self> {
        let (commands, receiver) = bounded(8);
        thread::Builder::new()
            .name("vocal-chain-audio-control".into())
            .spawn(move || run_control_thread(app, receiver))?;
        Ok(Self { commands })
    }

    pub fn start(
        &self,
        input_device: String,
        output_device: String,
        buffer_frames: u32,
        params: ChainParams,
    ) -> Result<(), String> {
        if !(MIN_BUFFER_FRAMES..=MAX_BUFFER_FRAMES).contains(&buffer_frames) {
            return Err("Buffergröße muss zwischen 128 und 1024 Samples liegen.".into());
        }

        let (reply, result) = bounded(1);
        self.commands
            .send(WorkerCommand::Start(
                AudioConfig {
                    input_device,
                    output_device,
                    buffer_frames,
                    params,
                },
                reply,
            ))
            .map_err(|_| "Audio-Steuerung ist nicht verfügbar.".to_string())?;
        result
            .recv()
            .map_err(|_| "Audio-Start wurde unerwartet abgebrochen.".to_string())?
    }

    pub fn stop(&self) -> Result<(), String> {
        let (reply, result) = bounded(1);
        self.commands
            .send(WorkerCommand::Stop(reply))
            .map_err(|_| "Audio-Steuerung ist nicht verfügbar.".to_string())?;
        result
            .recv()
            .map_err(|_| "Audio-Stopp wurde unerwartet abgebrochen.".to_string())
    }
}

pub fn list_devices() -> anyhow::Result<AudioDevices> {
    let host = cpal::default_host();
    let inputs = host
        .input_devices()
        .context("Mikrofon-Geräte konnten nicht aufgelistet werden.")?;
    let outputs = host
        .output_devices()
        .context("Ausgabe-Geräte konnten nicht aufgelistet werden.")?;

    Ok(AudioDevices {
        inputs: collect_devices(inputs)?,
        outputs: collect_devices(outputs)?,
    })
}

fn collect_devices(devices: impl Iterator<Item = Device>) -> anyhow::Result<Vec<AudioDevice>> {
    devices
        .enumerate()
        .map(|(index, device)| {
            let name = device
                .name()
                .context("Audiogerät-Name konnte nicht gelesen werden.")?;
            Ok(AudioDevice {
                id: format!("{index}::{name}"),
                name,
            })
        })
        .collect()
}

fn run_control_thread(app: AppHandle, commands: Receiver<WorkerCommand>) {
    let mut configuration: Option<AudioConfig> = None;
    let mut streams: Option<(Stream, Stream)> = None;
    let mut stream_failed = Arc::new(AtomicBool::new(false));
    let mut reconnect_at: Option<Instant> = None;
    let mut metrics: Option<Arc<AudioMetrics>> = None;
    let mut last_meter_at = Instant::now();

    loop {
        match commands.recv_timeout(METER_INTERVAL) {
            Ok(WorkerCommand::Start(config, reply)) => {
                streams = None;
                configuration = None;
                stream_failed = Arc::new(AtomicBool::new(false));
                let audio_metrics = Arc::new(AudioMetrics::default());
                match connect(
                    &config,
                    Arc::clone(&stream_failed),
                    Arc::clone(&audio_metrics),
                ) {
                    Ok(opened) => {
                        streams = Some(opened);
                        configuration = Some(config);
                        metrics = Some(audio_metrics);
                        reconnect_at = None;
                        last_meter_at = Instant::now();
                        emit_status(&app, "running", None);
                        let _ = reply.send(Ok(()));
                    }
                    Err(error) => {
                        metrics = None;
                        reconnect_at = None;
                        emit_status(&app, "error", Some(error.to_string()));
                        let _ = reply.send(Err(format!("{error:#}")));
                    }
                }
            }
            Ok(WorkerCommand::Stop(reply)) => {
                streams = None;
                configuration = None;
                metrics = None;
                reconnect_at = None;
                emit_status(&app, "stopped", None);
                let _ = reply.send(());
            }
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {}
        }

        if configuration.is_some() && stream_failed.swap(false, Ordering::AcqRel) {
            streams = None;
            reconnect_at = Some(Instant::now());
            emit_status(
                &app,
                "reconnecting",
                Some("Audiostream unterbrochen; Verbindung wird wiederhergestellt.".into()),
            );
        }

        if let (Some(config), Some(at)) = (configuration.as_ref(), reconnect_at) {
            if streams.is_none() && Instant::now() >= at {
                stream_failed = Arc::new(AtomicBool::new(false));
                let audio_metrics = metrics
                    .as_ref()
                    .expect("running configuration keeps its metrics")
                    .clone();
                match connect(
                    config,
                    Arc::clone(&stream_failed),
                    Arc::clone(&audio_metrics),
                ) {
                    Ok(opened) => {
                        streams = Some(opened);
                        reconnect_at = None;
                        emit_status(&app, "running", None);
                    }
                    Err(error) => {
                        reconnect_at = Some(Instant::now() + RECONNECT_DELAY);
                        emit_status(
                            &app,
                            "reconnecting",
                            Some(format!("Audiogerät noch nicht verfügbar: {error:#}")),
                        );
                    }
                }
            }
        }

        if streams.is_some() && last_meter_at.elapsed() >= METER_INTERVAL {
            if let Some(audio_metrics) = metrics.as_ref() {
                let _ = app.emit("audio-metrics", audio_metrics.snapshot());
            }
            last_meter_at = Instant::now();
        }
    }
}

fn emit_status(app: &AppHandle, state: &'static str, message: Option<String>) {
    let _ = app.emit("audio-status", AudioStatus { state, message });
}

fn connect(
    config: &AudioConfig,
    stream_failed: Arc<AtomicBool>,
    metrics: Arc<AudioMetrics>,
) -> anyhow::Result<(Stream, Stream)> {
    let host = cpal::default_host();
    let input = resolve_device(host.input_devices()?, &config.input_device)
        .context("Eingabegerät konnte nicht geöffnet werden.")?;
    let output = resolve_device(host.output_devices()?, &config.output_device)
        .context("Ausgabegerät konnte nicht geöffnet werden.")?;

    let (input_config, input_format) = stream_config(&input, true, config.buffer_frames)
        .context("Mikrofon unterstützt kein 48-kHz-Audio.")?;
    let (output_config, output_format) = stream_config(&output, false, config.buffer_frames)
        .context("Ausgabe unterstützt kein 48-kHz-Audio.")?;
    config.params.validate(SAMPLE_RATE)?;
    let input_chains = (0..input_config.channels)
        .map(|_| super::nodes::Chain::new(config.params.clone(), SAMPLE_RATE))
        .collect::<anyhow::Result<Vec<_>>>()?;
    let (sender, receiver) = bounded::<[f32; 2]>((config.buffer_frames as usize * 4).max(512));
    let input_stream = build_input_stream(
        &input,
        &input_config,
        input_format,
        input_chains,
        Arc::clone(&metrics),
        sender,
        Arc::clone(&stream_failed),
    )?;
    let output_stream = build_output_stream(
        &output,
        &output_config,
        output_format,
        metrics,
        receiver,
        stream_failed,
    )?;

    input_stream
        .play()
        .context("Mikrofon-Stream konnte nicht gestartet werden.")?;
    output_stream
        .play()
        .context("Ausgabe-Stream konnte nicht gestartet werden.")?;
    Ok((input_stream, output_stream))
}

fn resolve_device(
    devices: impl Iterator<Item = Device>,
    requested_id: &str,
) -> anyhow::Result<Device> {
    let (requested_index, requested_name) = requested_id
        .split_once("::")
        .ok_or_else(|| anyhow!("Ungültige Geräteauswahl."))?;
    let requested_index = requested_index
        .parse::<usize>()
        .context("Ungültige Geräteauswahl.")?;
    let devices = devices.collect::<Vec<_>>();

    if let Some(device) = devices.get(requested_index) {
        if device
            .name()
            .context("Audiogerät-Name konnte nicht gelesen werden.")?
            == requested_name
        {
            return Ok(device.clone());
        }
    }
    for device in devices {
        if device
            .name()
            .context("Audiogerät-Name konnte nicht gelesen werden.")?
            == requested_name
        {
            return Ok(device);
        }
    }
    Err(anyhow!("Gerät „{requested_name}“ ist nicht verfügbar."))
}

fn stream_config(
    device: &Device,
    input: bool,
    buffer_frames: u32,
) -> anyhow::Result<(StreamConfig, SampleFormat)> {
    if input {
        choose_stream_config(device.supported_input_configs()?, buffer_frames)
    } else {
        choose_stream_config(device.supported_output_configs()?, buffer_frames)
    }
}

fn choose_stream_config(
    supported: impl Iterator<Item = cpal::SupportedStreamConfigRange>,
    buffer_frames: u32,
) -> anyhow::Result<(StreamConfig, SampleFormat)> {
    let chosen = supported
        .filter(|range| {
            (range.channels() == 1 || range.channels() == 2)
                && range.min_sample_rate().0 <= SAMPLE_RATE
                && range.max_sample_rate().0 >= SAMPLE_RATE
                && matches!(
                    range.sample_format(),
                    SampleFormat::F32 | SampleFormat::I16 | SampleFormat::U16
                )
        })
        .max_by_key(|range| {
            (
                range.sample_format() == SampleFormat::F32,
                range.channels() == 2,
            )
        })
        .ok_or_else(|| anyhow!("Kein unterstütztes Mono-/Stereoformat bei 48 kHz gefunden."))?;
    let format = chosen.sample_format();
    let mut config = chosen.with_sample_rate(SampleRate(SAMPLE_RATE)).config();
    config.buffer_size = BufferSize::Fixed(buffer_frames);
    Ok((config, format))
}

trait ToFloat {
    fn to_float(self) -> f32;
}

impl ToFloat for f32 {
    fn to_float(self) -> f32 {
        if self.is_finite() {
            self
        } else {
            0.0
        }
    }
}

impl ToFloat for i16 {
    fn to_float(self) -> f32 {
        self as f32 / 32_768.0
    }
}

impl ToFloat for u16 {
    fn to_float(self) -> f32 {
        (self as f32 - 32_768.0) / 32_768.0
    }
}

trait FromFloat {
    fn from_float(value: f32) -> Self;
}

impl FromFloat for f32 {
    fn from_float(value: f32) -> Self {
        value
    }
}

impl FromFloat for i16 {
    fn from_float(value: f32) -> Self {
        (value.clamp(-1.0, 1.0) * 32_767.0).round() as i16
    }
}

impl FromFloat for u16 {
    fn from_float(value: f32) -> Self {
        ((value.clamp(-1.0, 1.0) * 0.5 + 0.5) * 65_535.0).round() as u16
    }
}

fn build_input_stream_typed<T: cpal::SizedSample + ToFloat + 'static>(
    device: &Device,
    config: &StreamConfig,
    sample_format: SampleFormat,
    mut chains: Vec<super::nodes::Chain>,
    metrics: Arc<AudioMetrics>,
    sender: Sender<[f32; 2]>,
    stream_failed: Arc<AtomicBool>,
) -> anyhow::Result<Stream> {
    if T::FORMAT != sample_format {
        return Err(anyhow!(
            "Audiogerät lieferte ein unerwartetes Sample-Format."
        ));
    }
    let channels = config.channels as usize;
    device
        .build_input_stream(
            config,
            move |data: &[T], _| {
                for frame in data.chunks_exact(channels) {
                    let input_left = frame[0].to_float();
                    metrics
                        .input_peak
                        .fetch_max(input_left.abs().to_bits(), Ordering::Relaxed);
                    let left = finite_or_silence(chains[0].process_sample(input_left));
                    let right = if channels == 1 {
                        left
                    } else {
                        let input_right = frame[1].to_float();
                        metrics
                            .input_peak
                            .fetch_max(input_right.abs().to_bits(), Ordering::Relaxed);
                        finite_or_silence(chains[1].process_sample(input_right))
                    };
                    if sender.try_send([left, right]).is_err() {
                        metrics.queue_overruns.fetch_add(1, Ordering::Relaxed);
                    }
                }
            },
            move |_| stream_failed.store(true, Ordering::Release),
            None,
        )
        .context("Mikrofon-Stream konnte nicht erstellt werden.")
}

fn build_output_stream_typed<T: cpal::SizedSample + FromFloat + 'static>(
    device: &Device,
    config: &StreamConfig,
    sample_format: SampleFormat,
    metrics: Arc<AudioMetrics>,
    receiver: Receiver<[f32; 2]>,
    stream_failed: Arc<AtomicBool>,
) -> anyhow::Result<Stream> {
    if T::FORMAT != sample_format {
        return Err(anyhow!(
            "Audiogerät lieferte ein unerwartetes Sample-Format."
        ));
    }
    let channels = config.channels as usize;
    device
        .build_output_stream(
            config,
            move |data: &mut [T], _| {
                for frame in data.chunks_exact_mut(channels) {
                    let samples = match receiver.try_recv() {
                        Ok(samples) => samples,
                        Err(TryRecvError::Empty | TryRecvError::Disconnected) => {
                            metrics.queue_underruns.fetch_add(1, Ordering::Relaxed);
                            [0.0, 0.0]
                        }
                    };
                    metrics
                        .output_peak
                        .fetch_max(samples[0].abs().to_bits(), Ordering::Relaxed);
                    metrics
                        .output_peak
                        .fetch_max(samples[1].abs().to_bits(), Ordering::Relaxed);
                    if channels == 1 {
                        frame[0] = T::from_float((samples[0] + samples[1]) * 0.5);
                    } else {
                        frame[0] = T::from_float(samples[0]);
                        frame[1] = T::from_float(samples[1]);
                    }
                }
            },
            move |_| stream_failed.store(true, Ordering::Release),
            None,
        )
        .context("Ausgabe-Stream konnte nicht erstellt werden.")
}

fn build_input_stream(
    device: &Device,
    config: &StreamConfig,
    format: SampleFormat,
    chains: Vec<super::nodes::Chain>,
    metrics: Arc<AudioMetrics>,
    sender: Sender<[f32; 2]>,
    stream_failed: Arc<AtomicBool>,
) -> anyhow::Result<Stream> {
    match format {
        SampleFormat::F32 => build_input_stream_typed::<f32>(
            device,
            config,
            format,
            chains,
            metrics,
            sender,
            stream_failed,
        ),
        SampleFormat::I16 => build_input_stream_typed::<i16>(
            device,
            config,
            format,
            chains,
            metrics,
            sender,
            stream_failed,
        ),
        SampleFormat::U16 => build_input_stream_typed::<u16>(
            device,
            config,
            format,
            chains,
            metrics,
            sender,
            stream_failed,
        ),
        _ => Err(anyhow!(
            "Audiogerät verwendet ein nicht unterstütztes Sample-Format."
        )),
    }
}

fn build_output_stream(
    device: &Device,
    config: &StreamConfig,
    format: SampleFormat,
    metrics: Arc<AudioMetrics>,
    receiver: Receiver<[f32; 2]>,
    stream_failed: Arc<AtomicBool>,
) -> anyhow::Result<Stream> {
    match format {
        SampleFormat::F32 => build_output_stream_typed::<f32>(
            device,
            config,
            format,
            metrics,
            receiver,
            stream_failed,
        ),
        SampleFormat::I16 => build_output_stream_typed::<i16>(
            device,
            config,
            format,
            metrics,
            receiver,
            stream_failed,
        ),
        SampleFormat::U16 => build_output_stream_typed::<u16>(
            device,
            config,
            format,
            metrics,
            receiver,
            stream_failed,
        ),
        _ => Err(anyhow!(
            "Audiogerät verwendet ein nicht unterstütztes Sample-Format."
        )),
    }
}

fn peak_dbfs(peak: f32) -> Option<f64> {
    if peak <= 0.0 {
        None
    } else {
        Some(20.0 * f64::from(peak).log10())
    }
}

fn finite_or_silence(sample: f32) -> f32 {
    if sample.is_finite() {
        sample
    } else {
        0.0
    }
}
