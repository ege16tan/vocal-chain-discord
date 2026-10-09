use std::path::PathBuf;

use tauri::State;

use crate::audio::{
    engine::{list_devices as list_audio_devices_inner, AudioDevices, AudioEngine},
    process_wav as process_wav_file, ChainParams, ProcessReport,
};

#[tauri::command]
pub fn process_wav(
    input_path: String,
    output_path: String,
    params: ChainParams,
) -> Result<ProcessReport, String> {
    process_wav_file(
        &PathBuf::from(input_path),
        &PathBuf::from(output_path),
        params,
    )
    .map_err(|error| format!("{error:#}"))
}

#[tauri::command]
pub fn list_audio_devices() -> Result<AudioDevices, String> {
    list_audio_devices_inner().map_err(|error| format!("{error:#}"))
}

#[tauri::command]
pub fn start_audio(
    input_device: String,
    output_device: String,
    buffer_frames: u32,
    params: ChainParams,
    engine: State<'_, AudioEngine>,
) -> Result<(), String> {
    engine.start(input_device, output_device, buffer_frames, params)
}

#[tauri::command]
pub fn stop_audio(engine: State<'_, AudioEngine>) -> Result<(), String> {
    engine.stop()
}
