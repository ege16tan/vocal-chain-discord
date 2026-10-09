# Nächste Schritte

## Aktueller Stand
- GitHub: `ege16tan/vocal-chain-discord`; aktuelle Änderungen liegen auf `ege16tan-automatic-doodle`.
- Erster Code-Meilenstein: Tauri-Desktop-Shell und Offline-WAV-Processing mit HPF → De-Esser → Limiter, Loudness-Auswertung und DSP-Unit-Tests.
- Erste Live-Kette: CPAL-Gerätewahl, HPF → De-Esser → Limiter, 48 kHz, Buffer 128–1024, Reconnect-Prototyp und Peak-/Queue-Diagnose.
- Noch offen: lock-free Parameterupdates/Smoothing, Live-LUFS/GR-Meter und Messungen mit echten Audiogeräten. Hardware-Clock-Drift und Ende-zu-Ende-Latenz sind ungeprüft.
- `npm run build` ist erfolgreich. Rust-Tests konnten nicht starten, da `link.exe` fehlt; das installierte Build-Tools-Paket enthält noch nicht den C++-Compiler/Windows-SDK-Workload.

## Nächste Arbeitsschritte
Windows-Voraussetzungen, Audio-Testprotokoll und Fertig-Kriterien stehen in `WINDOWS-SETUP.md`.

Kurzfassung:
1. `cargo test --locked` und `cargo check` mit unterstütztem Windows-MSVC-Toolchain ausführen; der alternative GNU-Testlauf wird separat dokumentiert.
2. CPAL-Geräteliste um unterstützte Sample-Formate, Kanäle und Buffer-Ranges ergänzen; Bufferanfrage und tatsächliche Callback-Größen ausgeben.
3. DSP-Parameterübergabe lock-free mit sampleweisem Smoothing ermöglichen; keine Locks, Allokationen oder Tauri-IPC im Callback.
4. LUFS- und Gain-Reduction-Meter in den Live-Pfad ergänzen.
5. Live-Kette mit echten Geräten testen: Abziehen/Wechseln/Wiederverbinden, Queue-Under-/Overruns, hörbare Aussetzer und Ende-zu-Ende-Latenz messen.

Änderungen in kleinen, thematischen Commits auf den Arbeitsbranch pushen. Vor jedem Push Secrets ausschließen; nie `--force` anwenden.

## Fragen und Übergabe
Es sind keine Nutzerentscheidungen blockierend: Sample-Rate, Buffer-Zielbereich, Effektumfang, De-Esser, Limiter und Discord-Setup sind in `PLAN.md` festgelegt. CPALs angefragte Buffergröße ist kein Versprechen über die tatsächliche Callback-Größe; reale Werte und Audiogerätefähigkeit messen und dokumentieren, statt Unterstützung vorzutäuschen. Neue, unvermeidbare Produktfragen zuerst mit Kontext/Optionen in `QUESTIONS.md` dokumentieren, dann nicht mit erfundenen Defaults in einen anderen Scope wechseln.
