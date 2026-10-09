# Windows-Setup, Apps und Entwicklungsfahrplan

Dieses Dokument enthält Windows-Voraussetzungen und manuelle Prüfschritte für `ege16tan/vocal-chain-discord`. Die aktuelle App hat Offline-WAV-Processing und eine erste CPAL-Live-Kette (HPF → De-Esser → Limiter); Live-LUFS/GR, Parameter-Smoothing und echte Hardware-Validierung fehlen noch.

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
git clone https://github.com/ege16tan/vocal-chain-discord.git
cd vocal-chain-discord
npm install
cargo test --manifest-path src-tauri/Cargo.toml
npm run build
npm run tauri dev
```

Das Repository ist privat; GitHub-Authentifizierung erfolgt über die eingerichtete Git-Authentifizierung. Keine Zugangsdaten in Dateien oder URLs eintragen. Vor jedem Commit `git status --short` ansehen. Lokale Zugangsdaten, Aufnahmen mit privaten Gesprächen und persönliche Geräte-/Nutzernamen gehören nicht ins Repo.

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
4. Live-Kette mit Kopfhörern und anschließend VB-CABLE testen; Windows-Eingabe = physisches Mikrofon, App-Ausgabe = `CABLE Input`, Discord-Eingabe = `CABLE Output`. Discord-AGC, Krisp/Noise Suppression und Echo Cancellation gemäß `PLAN.md` deaktivieren.
5. Bei Rucklern zuerst mit größerem Buffer vergleichen, dann Callback-Auslastung, Underruns und VB-CABLE-Buffer getrennt messen. Buffer-Größe, die CPAL anfragt, muss nicht der tatsächlichen Callback-Größe entsprechen.
6. WPR/WPA aus dem Windows ADK ist ein optionales Diagnosewerkzeug für ETW-/Scheduler-Probleme, kein Muss für den ersten App-Build.

## Umsetzungsreihenfolge für Live-Audio

Der aktuelle Live-Pfad hat die geplante DSP-Kette, ist aber noch nicht durch reale Audiogeräte- und Latenztests validiert.

1. CPAL-Geräte sind aufgelistet und auswählbar; als Nächstes unterstützte Sample-Formate, 48-kHz-Unterstützung, Kanäle und Buffer-Ranges für **jedes** Gerät anzeigen. Nicht davon ausgehen, dass jedes Mikro exakt dieselben Streamparameter wie VB-CABLE unterstützt.
2. Ein-/Ausgabe-Stream mit DSP starten und stoppen; Verhalten und Fehlermeldungen bei fehlendem Gerät, inkompatiblem Format und Streamstart-Fehler testen.
3. Audio zwischen getrennten CPAL-Callbacks über einen vorab allokierten begrenzten SPSC-Ringbuffer führen. Im Audio-Callback keine Locks, Dateizugriffe, Logs, Allokationen oder Tauri-IPC. Bei Über-/Unterlauf Zähler führen und sicheren Stille-Fallback definieren.
4. Vorhandenen Reconnect-Prototyp testen: Gerät abziehen/abschalten und wieder anschließen, Standardgerät wechseln und Streamfehler provozieren. Reconnect auf Kontrollthread neu aufbauen; keine Panics und keine unendlichen schnellen Retry-Schleifen.
5. Live-DSP ist integriert. Parameter werden aktuell beim Start übernommen und Regler während der Wiedergabe gesperrt; vor veränderbaren Parametern lock-free Targets mit sampleweisem Smoothing implementieren.
6. Windows-Mikrofon → App → VB-CABLE → Discord messen und anhören. Ende-zu-Ende-Latenz enthält Treiber-, Capture-/Playback-, Kabel- und Discord-Buffer; kleiner App-Buffer allein beweist keine `<20 ms`.

CPALs `BufferSize::Fixed(n)` ist eine **Anfrage**, keine Garantie, dass jeder Callback exakt n Frames erhält. Tatsächliche Größen, Sample-Rate, Underruns, CPU-Zeit und Stream-Neustarts instrumentieren. Die CPAL-Dokumentation erklärt diesen Unterschied und den Zielkonflikt: [CPAL BufferSize](https://docs.rs/cpal/latest/cpal/enum.BufferSize.html).

## DSP-Optimierungen erst nach Messung

- Den Limiter nicht als validierten True-Peak-Limiter freigeben: aktuelle kubische 4×-Interpolation ist nur ein Prototyp, kein geeigneter FIR-Übersampler-Nachweis.
- Laufzeit des aktuellen Peak-Scans messen: pro Sample wird über den Lookahead gescannt; für Live-Betrieb ist ein begrenzter Sliding-Peak/4×-FIR-Ansatz zu entwickeln und mit Inter-Sample-Peak-Testsignalen gegen eine Referenz zu vergleichen.
- De-Esser berechnet Potenzen im Sample-Pfad. Zuerst Profiling bei 48 kHz; falls relevant, Gain-Kurve vereinfachen/vorberechnen, ohne hörbare Kennlinie oder Limits still zu verändern.
- Offline-Harness liest aktuell die gesamte WAV in Speicher. Nur wenn lange Dateien problematisch sind, blockweise verarbeiten und Loudness über die Chunks fortschreiben.
- Bei langen 48-kHz-Sessions auf mögliche Capture-/Playback-Clock-Drift achten. Erst Unter-/Überlaufmessungen sammeln; adaptive Resampling-Lösung nur mit reproduzierbarem Fehler und Tests hinzufügen.
- Im Kaltstart/UI/Loudness-Test auch leere Dateien, Stille, sehr kurze WAVs, kaputte Header, nicht unterstützte Kanäle/Samplerates und dieselben Ein-/Ausgabepfade prüfen.

Ziel für ein täglich nutzbares Binary bleibt Windows.
