# Windows-Setup, Apps und Entwicklungsfahrplan

Dieses Dokument enthält die Windows-Schritte, die in der aktuellen Linux-Arbeitsumgebung nicht ausführbar sind. Die Informationen sind für die KI gedacht, die das öffentliche Forgejo-Repo unter Windows klont und dort weiterarbeitet.

## Programme, die wirklich nötig sind

Installiere nur diese Grundausstattung; keine Audio-Plugins, Voicemeeter, ASIO4ALL, CMake, CUDA oder Electron nötig:

| Programm | Warum |
|---|---|
| Git for Windows | Repo klonen, kleine Commits und Pushes |
| Rust stable über rustup, Ziel `x86_64-pc-windows-msvc` | Tauri- und DSP-Backend |
| Visual Studio 2022 Build Tools mit **Desktop development with C++** | MSVC-Linker und Windows SDK; den Workload und Windows SDK im Installer mitauswählen |
| Microsoft Edge WebView2 Evergreen Runtime | Tauri-Frontend unter Windows |
| Node.js LTS samt npm | React/Vite-Abhängigkeiten und Frontend-Build |
| Audacity (optional, aber hilfreich) | Test-WAVs erzeugen, aufzeichnen und Output abhören |
| VB-CABLE von VB-Audio (erst für Live-Audiotests) | Virtuelles Windows-Audiokabel Mic-App → Discord |
| Discord Desktop | Abschließender Eingangstest und Deaktivieren von Doppelverarbeitung |

Rust Analyzer und rustfmt können im vorhandenen Editor genutzt werden, sind aber keine Build-Voraussetzung. Windows Performance Recorder/Analyzer (Windows ADK) **nur bei nachgewiesenem Timing-/CPU-Problem** installieren. Keine neuen Abhängigkeiten zur App hinzufügen, nur weil ein optionales Tool installiert ist.

Maßgebliche Installationsquellen: [Tauri-Windows-Voraussetzungen](https://v2.tauri.app/start/prerequisites/#windows), [Rust/rustup](https://www.rust-lang.org/tools/install), [Node.js LTS](https://nodejs.org/en/download), [Audacity für Windows](https://www.audacityteam.org/download/windows/), [VB-Audio CABLE](https://vb-audio.com/Cable/).

Nach der Installation ein **neues** PowerShell-Fenster öffnen und prüfen:

```powershell
git --version
rustc --version
cargo --version
rustup show
node --version
npm --version
```

Wenn `rustc` fehlt, zuerst Terminal neu starten und `rustup show`/PATH prüfen. Für MSI-Fehler wie `failed to run light.exe` prüfen, ob das optionale Windows-Feature **VBScript** deaktiviert wurde; nur bei diesem Fehler nach Tauri-Anleitung aktivieren. Ein signiertes Installationsbinary ist derzeit nicht vorhanden; SmartScreen-Warnung einplanen.

## Klonen, Abhängigkeiten und erste Build-Runde

```powershell
git clone http://192.168.178.31:3000/githubrocky/vocal-chain-discord.git
cd vocal-chain-discord
npm install
cargo test --manifest-path src-tauri/Cargo.toml
npm run build
npm run tauri dev
```

Das Repository ist öffentlich: Für `git clone` kein Passwort, Token oder `.env` anlegen. Vor jedem Commit `git status --short` ansehen. Lokale Zugangsdaten, Aufnahmen mit privaten Gesprächen und persönliche Geräte-/Nutzernamen gehören nicht ins öffentliche Repo.

Bei der allerersten Installation `Cargo.lock` mit Cargo und `package-lock.json` mit npm erzeugen, beide Dateien kontrollieren und committen. Danach müssen die reproduzierbaren Befehle `cargo test --locked` und `npm ci` funktionieren. Locks bei einer gezielten Abhängigkeitsänderung gemeinsam aktualisieren; nicht blind alle Pakete upgraden.

Mit mindestens einer Mono- und einer Stereo-WAV-Datei (Integer und Float) Eingabe und Ausgabe in der UI testen; zusätzlich `cargo run --manifest-path src-tauri/Cargo.toml --bin harness -- --help` prüfen und den Output in Audacity anhören. Quelldatei vergleichen: sie darf unverändert bleiben. Anschließend:

```powershell
npm run tauri build
```

Erfolgskriterien dieser Runde: `cargo test --locked`, `npm ci`, `npm run build` und Tauri-Devstart sind erfolgreich; Harness-Ausgabe lässt sich öffnen und klingt plausibel; der Windows-Bundle-Build wird erstellt. Fehlermeldung, Toolchain-Version und genaue Befehle in `QUESTIONS.md` dokumentieren, niemals Zugangsdaten oder Audioinhalte.

## Audio-Testdateien und manuelle Apps

1. Audacity: je eine Mono- und Stereo-Testdatei mit Sprache sowie Sinuston/Impuls erzeugen oder aufnehmen. Testfälle mit Art, Sample-Rate, Kanalzahl und erwartetem Verhalten notieren; keine private Sprachprobe hochladen.
2. Offline-App: identische Eingabe mit verschiedenen HPF-/De-Esser-/Limiter-Einstellungen verarbeiten und in Audacity A/B abhören. „Sibilanzband abhören“ soll tatsächlich nur den isolierten Bandanteil hörbar machen.
3. Frequenzen/Peaks nicht nur nach Gehör abnehmen: HPF-Response, De-Esser-Reduktion innerhalb/außerhalb des Bands und True-Peak-Limiter messbar testen.
4. Für den späteren Live-Test VB-CABLE nur von VB-Audio installieren; Windows-Eingabe = physisches Mikrofon, App-Ausgabe = `CABLE Input`, Discord-Eingabe = `CABLE Output`. Discord-AGC, Krisp/Noise Suppression und Echo Cancellation gemäß `PLAN.md` deaktivieren.
5. Bei Rucklern zuerst mit größerem Buffer vergleichen, dann Callback-Auslastung, Underruns und VB-CABLE-Buffer getrennt messen. Buffer-Größe, die CPAL anfragt, muss nicht der tatsächlichen Callback-Größe entsprechen.
6. WPR/WPA aus dem Windows ADK ist ein optionales Diagnosewerkzeug für ETW-/Scheduler-Probleme, kein Muss für den ersten App-Build.

## Umsetzungsreihenfolge für Live-Audio

Die erste Version verarbeitet **nur Dateien**. Nicht behaupten, sie sei schon ein nutzbares Discord-Mikrofon.

1. Erst CPAL-Geräte auflisten; unterstützte Sample-Formate, 48-kHz-Unterstützung, Kanäle und Buffer-Ranges für **jedes** Gerät anzeigen. Nicht davon ausgehen, dass jedes Mikro exakt dieselben Streamparameter wie VB-CABLE unterstützt.
2. Minimalen Ein-/Ausgabe-Stream ohne DSP starten und sauber stoppen; klare UI-/Log-Meldung bei Gerät fehlt, Format inkompatibel oder Streamstart fehlgeschlagen.
3. Audio zwischen getrennten CPAL-Callbacks über einen vorab allokierten begrenzten SPSC-Ringbuffer führen. Im Audio-Callback keine Locks, Dateizugriffe, Logs, Allokationen oder Tauri-IPC. Bei Über-/Unterlauf Zähler führen und sicheren Stille-Fallback definieren.
4. Gerät abziehen/abschalten und wieder anschließen, Standardgerät wechseln und Streamfehler provozieren. Reconnect auf Kontrollthread neu aufbauen; keine Panics und keine unendlichen schnellen Retry-Schleifen.
5. Erst nach stabiler nackter Pipe die DSP-Kette integrieren. UI sendet nur Targets; Sample-genaues Smoothing im DSP. Updates sollen den Audio-Thread nicht blockieren.
6. Windows-Mikrofon → App → VB-CABLE → Discord messen und anhören. Ende-zu-Ende-Latenz enthält Treiber-, Capture-/Playback-, Kabel- und Discord-Buffer; kleiner App-Buffer allein beweist keine `<20 ms`.

CPALs `BufferSize::Fixed(n)` ist eine **Anfrage**, keine Garantie, dass jeder Callback exakt n Frames erhält. Tatsächliche Größen, Sample-Rate, Underruns, CPU-Zeit und Stream-Neustarts instrumentieren. Die CPAL-Dokumentation erklärt diesen Unterschied und den Zielkonflikt: [CPAL BufferSize](https://docs.rs/cpal/latest/cpal/enum.BufferSize.html).

## DSP-Optimierungen erst nach Messung

- Den Limiter nicht als validierten True-Peak-Limiter freigeben: aktuelle kubische 4×-Interpolation ist nur ein Prototyp, kein geeigneter FIR-Übersampler-Nachweis.
- Laufzeit des aktuellen Peak-Scans messen: pro Sample wird über den Lookahead gescannt; für Live-Betrieb ist ein begrenzter Sliding-Peak/4×-FIR-Ansatz zu entwickeln und mit Inter-Sample-Peak-Testsignalen gegen eine Referenz zu vergleichen.
- De-Esser berechnet Potenzen im Sample-Pfad. Zuerst Profiling bei 48 kHz; falls relevant, Gain-Kurve vereinfachen/vorberechnen, ohne hörbare Kennlinie oder Limits still zu verändern.
- Offline-Harness liest aktuell die gesamte WAV in Speicher. Nur wenn lange Dateien problematisch sind, blockweise verarbeiten und Loudness über die Chunks fortschreiben.
- Bei langen 48-kHz-Sessions auf mögliche Capture-/Playback-Clock-Drift achten. Erst Unter-/Überlaufmessungen sammeln; adaptive Resampling-Lösung nur mit reproduzierbarem Fehler und Tests hinzufügen.
- Im Kaltstart/UI/Loudness-Test auch leere Dateien, Stille, sehr kurze WAVs, kaputte Header, nicht unterstützte Kanäle/Samplerates und dieselben Ein-/Ausgabepfade prüfen.

## Linux-Hinweise aus dem ursprünglichen Build-Versuch

Die bereitgestellte Linux-Umgebung hatte weder Rust/Cargo noch Node/npm oder Git im PATH. Der direkte crates.io-Aufruf lieferte HTTP 403, deshalb wurde dort kein Linux-Build behauptet oder ein ungetestetes Artefakt veröffentlicht. Für spätere Linux-Entwicklung sind außerdem Tauri-WebKitGTK- und ALSA-Entwicklungspakete nötig. Das Ziel für ein täglich nutzbares Binary bleibt Windows.
