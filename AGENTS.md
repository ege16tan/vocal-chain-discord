# AGENTS.md — vocal-chain-discord

Plan-only repo. Single source of truth: `PLAN.md` (v2, gestuft). Kein Code, keine Manifests, keine CI bisher.

## Stand
- Git: `main`, remote `origin https://github.com/ege16tan/vocal-chain-discord.git` (private). 1 Commit (PLAN.md v2).
- `gh` auth als `ege16tan` bereits erfolgt. Push via `gh repo create` erledigt.
- Nächster Schritt: Tauri-2-Scaffold + `harness.rs` (WAV rein/raus) VOR Echtzeit-Schleife. Noch nicht begonnen.

## Scope Stufe 1 (jetzt)
- Nur: HPF → De-Esser → Limiter + Loudness-Meter (read-only).
- Explizit RAUS: Preset-System, A/B, Tray, Auto-Start, Expander, EQ, Compressor (alles Stufe 2).
- Kein Auto-Makeup. Keine Cloud.

## Fest entschieden (nicht erneut fragen)
- 48 kHz fix, Buffer user-wählbar 128–1024, Default 256.
- De-Esser Default 5–8 kHz (einstellbar 3–12 kHz). Limiter True Peak 4x, Ceiling -1 dBTP. Loudness-Ref -16 LUFS (nur Anzeige, Discord normalisiert nicht).
- Discord-Setup: Input = CABLE Output, AGC / Krisp / Noise Suppression / Echo Cancellation AUS (sonst Doppel-Processing). VB-Cable-Buffer klein stellen, sonst <20 ms Ende-zu-Ende unrealistisch.
- Robustheit: Device-Wechsel mid-game muss Stream ohne Crash neu aufbauen. Unsigned .exe → SmartScreen-Hinweis einplanen.

## Umgebung / Kommandos
- Windows + PowerShell. `ls -la` geht NICHT — `Get-ChildItem` oder `git -C` nutzen.
- Git-Befehle mit `git -C vocal-chain-discord ...` aus `C:\Users\egeta` heraus, oder `workdir` setzen.
- User kommuniziert Deutsch. Antworten Deutsch, kurz, keine Emojis.
- User-Use-Case: Gaming/Discord, täglich nutzbares Binary. Kein kommerzielles/Plugin-Ziel aktuell.

## Beim Bauen beachten
- Parameter-Smoothing lock-free (Frontend sendet nur Targets, Rust smootht pro Sample) — gegen Klicks beim Drehen.
- Reihenfolge einhalten: `0. Harness` → `1. Engine minimal (Reconnect!)` → `2. DSP Stufe 1` → `3. Frontend Stufe 1`. Details in PLAN.md.
- `PLAN.md` bei Scope-Änderungen mitziehen und committen.
