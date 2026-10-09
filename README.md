# Vocal Chain Discord

Kleine Windows-Desktop-App für die Discord-Gaming-Stimme. Die aktuelle erste Version enthält eine Tauri-Oberfläche und einen **Offline-WAV-Harness** mit HPF → De-Esser → Limiter sowie EBU-R128-Integrated-Loudness-Anzeige. Das Live-Mikrofon, VB-Cable-Routing und Reconnect sind noch nicht implementiert.

## Starten und testen

Voraussetzungen siehe [`WINDOWS-SETUP.md`](WINDOWS-SETUP.md). Im Projektordner:

```powershell
npm install
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri dev
```

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

Die verbindlichen Anforderungen stehen in [`PLAN.md`](PLAN.md). Nächster Meilenstein ist eine separate Echtzeit-Engine mit CPAL/WASAPI, Device-Auswahl, geglätteten Parametern und Reconnect. Dokumentations- und Prüfaufgaben für die Windows-KI stehen in [`NEXT_SESSION.md`](NEXT_SESSION.md) und [`WINDOWS-SETUP.md`](WINDOWS-SETUP.md).

Das Projekt ist offline und lokal ausgelegt; die App benötigt für die Oberfläche keine externen Webfonts oder Cloud-Dienste. Zugangsdaten gehören nie ins Repo. Lokale `.env`-Dateien werden ignoriert.
