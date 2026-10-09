use std::path::PathBuf;

use crate::audio::{process_wav as process_wav_file, ChainParams, ProcessReport};

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
