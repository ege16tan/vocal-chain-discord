# AGENTS.md — vocal-chain-discord

Arbeitsregeln und aktueller Stand für alle beitragenden Agents. Die verbindliche Projektplanung steht in `PLAN.md`.

## Stand und Repository
- Kanonisches Repo: `https://github.com/ege16tan/vocal-chain-discord` (privat).
- Änderungen in der aktuellen Session-Worktree bearbeiten; Zugangsdaten niemals ins Repo kopieren.
- Aktueller Implementierungsstand, fehlende Validierung und nächster Arbeitsauftrag: `NEXT_SESSION.md`.
- Windows-spezifische Einrichtung und Prüfschritte: `WINDOWS-SETUP.md`.

## Projektumfang Stufe 1
- Nur HPF → De-Esser → Limiter plus Loudness-Meter (read-only).
- Kein Preset-System, A/B, Tray, Auto-Start, Expander, EQ oder Compressor.
- Kein Auto-Makeup und keine Cloud.
- 48 kHz fix, Buffer 128–1024 (Default 256); De-Esser 5–8 kHz (einstellbar 3–12 kHz); Limiter True Peak 4x, Ceiling -1 dBTP; Loudness-Referenz -16 LUFS nur Anzeige.
- Discord: Input `CABLE Output`; AGC, Krisp, Noise Suppression und Echo Cancellation aus. VB-Cable-Buffer klein halten.
- Erster CPAL-Passthrough mit Gerätewahl und Reconnect-Prototyp ist vorhanden; Device-Wechsel mid-game muss noch manuell ohne Crash verifiziert werden.

## Technische Leitlinien
- Implementiert sind Offline-Harness, erster 48-kHz-CPAL-Stream mit Reconnect-Prototyp, Live-DSP-Chain und Peak-/Queue-Diagnose. Noch offen: lock-free Parameterupdates mit Smoothing, Live-LUFS/GR-Meter sowie Tests mit realen Audiogeräten.
- Parameter-Smoothing lock-free: Frontend sendet Targets, Rust smootht pro Sample gegen Klicks beim Verstellen.
- Windows ist das Zielsystem. Linux-Builds/Tests belegen keine Windows-Audio-Kompatibilität.
- Änderungen in kleinen, nachvollziehbaren Inkrementen auf Feature-Branches halten; keine Geheimnisse oder lokalen `.env`-Dateien übertragen.
- Nutzer kommuniziert Deutsch; Projektdokumentation und UI auf Deutsch.
