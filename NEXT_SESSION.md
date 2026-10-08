# Next Session — Stufe 1 starten

## Stand
- Repo: https://github.com/ege16tan/vocal-chain-discord (`main`, private, 2 Commits).
- Dateien: `PLAN.md` (v2), `AGENTS.md`. Kein Code bisher.
- Zuerst `AGENTS.md` lesen, dann diese Datei.

## Ziel Stufe 1
`harness.rs` (WAV rein → WAV raus) + HPF → De-Esser → Limiter + Loudness-Meter. Kein Tray, keine Presets, kein A/B.

## Konkrete Startbefehle (Windows PowerShell, aus `C:\Users\egeta\vocal-chain-discord`)
1. `cargo install create-tauri-app --locked` (falls fehlt)
2. Scaffold in TEMP, dann Dateien hierher mergen (nicht in existierendes Verzeichnis initen):
   `cargo create-tauri-app --template react-ts C:\Users\egeta\AppData\Local\Temp\opencode\vc-scaffold`
3. Deps: `cpal`, `ebur128`, `crossbeam`, `serde`, `anyhow`, `hound` (nur harness)
4. Reihenfolge: `audio/harness.rs` → `nodes/hpf.rs` → `nodes/deesser.rs` → `nodes/limiter.rs` → `nodes/loudness.rs` → `engine.rs` → Frontend minimal.
5. Tests: `cargo test` muss HPF-Response, DeEss-GR-Kurve, Limiter-Ceiling (-1 dBTP nie überschritten) prüfen.

## Fest (nicht erneut diskutieren)
- 48 kHz fix, Buffer 128–1024 Default 256, DeEss 5–8 kHz, Limiter 4x/-1 dBTP, -16 LUFS nur Anzeige.
- Reconnect bei Device-Wechsel ohne Crash. Param-Smoothing pro Sample.

## Fertig-Definition Stufe 1
`cargo test` grün + `harness input.wav output.wav` klingt sauber + live Mic → VB-Cable → Discord ohne Klicks/Crash.
