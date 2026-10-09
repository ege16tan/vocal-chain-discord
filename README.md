# Vocal Chain Discord

Kleine Windows-Desktop-App für die Discord-Gaming-Stimme. Die App enthält einen **Offline-WAV-Harness** und eine erste Live-Kette mit HPF → De-Esser → Limiter. Live-Pegel und Queue-Unter-/Überläufe werden als Diagnose angezeigt; EBU-R128-Integrated-Loudness ist derzeit nur im Offline-Harness verfügbar.

Die Live-Kette verwendet beim Streamstart die aktuellen UI-Werte. Während Audio läuft, sind die DSP-Regler gesperrt; zum Ändern muss der Stream gestoppt und neu gestartet werden. Die aktuelle CPAL-Verbindung ist noch nicht auf Hardware-Clock-Drift oder Ende-zu-Ende-Latenz abgestimmt und muss mit echten Geräten getestet werden.

## Starten und testen

Voraussetzungen siehe [`WINDOWS-SETUP.md`](WINDOWS-SETUP.md). Im Projektordner:

```powershell
npm install
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri dev
```

In der App können Mikrofon, Ausgabe und Buffer (128–1024 Samples) ausgewählt werden. Als Ausgabe für Discord ist `CABLE Input` vorgesehen. Die App meldet Verbindungsabbrüche und versucht, den Stream alle zwei Sekunden neu aufzubauen. Capture und Ausgabe laufen derzeit beide mit 48 kHz; unterschiedliches Hardware-Clocking und die Ende-zu-Ende-Latenz sind noch nicht korrigiert oder gemessen. Teste zunächst mit Kopfhörern oder VB-CABLE, um Lautsprecher-Rückkopplung zu vermeiden.

Eine Mono- oder Stereo-WAV-Datei und einen **anderen** Ausgabepfad angeben, dann „WAV verarbeiten“ klicken. Die Eingabe bleibt unverändert; die Ausgabe ist 32-bit Float WAV mit der Abtastrate der Eingabe. Das Loudness-Meter zeigt Integrated LUFS, soweit die Datei lang genug messbar ist. Stille wird als „nicht messbar“ angezeigt.

Der Offline-Harness arbeitet dateibasiert, lädt die Samples vollständig in den Speicher und ist ausdrücklich nicht für Echtzeit-Mikrofonverarbeitung gedacht. Er konvertiert die Sample-Rate nicht.

Der Offline-Harness ist zusätzlich als CLI-Binary verfügbar:

```powershell
cargo run --manifest-path src-tauri/Cargo.toml --bin harness -- input.wav output.wav
cargo run --manifest-path src-tauri/Cargo.toml --bin harness -- --help
```

Parameter wie `--hpf-hz`, `--deesser-threshold-db` und `--limiter-ceiling-db` können per Flag gesetzt werden.

Windows-Paket bauen:

```powershell
npm run tauri build
```

## DSP-Hinweis

HPF und De-Esser sind ein erster, testbarer DSP-Prototyp. Die Limiter-Spitzenanalyse verwendet eine 4× kubische Sample-Interpolation und begrenzt zusätzlich den Sample-Peak. Sie ist noch kein validierter True-Peak-Standard-konformer Oversampler; vor täglichem Live-Einsatz muss der Limiter gezielt messtechnisch validiert werden. Windows-Installations-, Test- und Diagnoseprogramme stehen in [`WINDOWS-SETUP.md`](WINDOWS-SETUP.md).

## Scope und Weiterarbeit

Die verbindlichen Anforderungen stehen in [`PLAN.md`](PLAN.md). Noch offen sind lock-free Parameter-Updates/Smoothing, LUFS- und Gain-Reduction-Meter im Live-Pfad sowie Tests mit echten Geräten, Reconnects und gemessener Latenz. Offene Prüfaufgaben stehen in [`NEXT_SESSION.md`](NEXT_SESSION.md) und [`WINDOWS-SETUP.md`](WINDOWS-SETUP.md).

Das Projekt ist offline und lokal ausgelegt; die App benötigt für die Oberfläche keine externen Webfonts oder Cloud-Dienste. Zugangsdaten gehören nie ins Repo. Lokale `.env`-Dateien werden ignoriert.
