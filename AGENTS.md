# AGENTS.md — vocal-chain-discord

Arbeitsregeln und aktueller Stand für alle beitragenden Agents. Die verbindliche Projektplanung steht in `PLAN.md`.

## Stand und Repository
- Kanonisches Repo: `http://192.168.178.31:3000/githubrocky/vocal-chain-discord` (Forgejo, öffentlich).
- Entwicklungsordner: `/home/ege/projects/vocal-chain-discord`.
- Zugangsdaten niemals ins Repo kopieren. Die lokale Forgejo-Notiz liegt außerhalb des Repos in `/home/ege/forgejo-login.md`.
- Aktueller Implementierungsstand, fehlende Validierung und nächster Arbeitsauftrag: `NEXT_SESSION.md`.
- Windows-spezifische Einrichtung und Prüfschritte: `WINDOWS-SETUP.md`.

## Projektumfang Stufe 1
- Nur HPF → De-Esser → Limiter plus Loudness-Meter (read-only).
- Kein Preset-System, A/B, Tray, Auto-Start, Expander, EQ oder Compressor.
- Kein Auto-Makeup und keine Cloud.
- 48 kHz fix, Buffer 128–1024 (Default 256); De-Esser 5–8 kHz (einstellbar 3–12 kHz); Limiter True Peak 4x, Ceiling -1 dBTP; Loudness-Referenz -16 LUFS nur Anzeige.
- Discord: Input `CABLE Output`; AGC, Krisp, Noise Suppression und Echo Cancellation aus. VB-Cable-Buffer klein halten.
- Device-Wechsel mid-game muss künftig den Stream ohne Crash neu aufbauen.

## Technische Leitlinien
- Reihenfolge: Offline-WAV-Harness und DSP-Tests → Audio-Engine mit Reconnect → DSP live integrieren → Frontend vervollständigen.
- Parameter-Smoothing lock-free: Frontend sendet Targets, Rust smootht pro Sample gegen Klicks beim Verstellen.
- Windows ist das Zielsystem. Linux-Builds/Tests belegen keine Windows-Audio-Kompatibilität.
- Änderungen in kleinen, nachvollziehbaren Inkrementen nach Forgejo `main` hochladen; keine Geheimnisse oder lokalen `.env`-Dateien übertragen.
- Nutzer kommuniziert Deutsch; Projektdokumentation und UI auf Deutsch.
