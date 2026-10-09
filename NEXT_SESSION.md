# Nächste Schritte

## Aktueller Stand
- Forgejo: `http://192.168.178.31:3000/githubrocky/vocal-chain-discord` (öffentlich, Branch `main`).
- Erster Code-Meilenstein: Tauri-Desktop-Shell und Offline-WAV-Processing mit HPF → De-Esser → Limiter, Loudness-Auswertung und DSP-Unit-Tests.
- Der Echtzeit-Mikrofon-/VB-Cable-Engine-Pfad ist noch nicht implementiert. Das Offline-Harness ist ausdrücklich kein fertiges Discord-Mikrofon.
- Die aktuelle Linux-Umgebung enthält kein Rust/Cargo, Node/npm oder Git; crates.io lieferte HTTP 403. Daher konnten Build, Tests und Windows-Binary hier nicht ausgeführt werden.

## Arbeitsauftrag für die Windows-KI
Arbeitsreihenfolge, empfohlene Windows-Programme, Audio-Testprotokoll und konkrete Fertig-Kriterien stehen in `WINDOWS-SETUP.md`.

Kurzfassung:
1. Mit den dort genannten Tools clonen, installieren, `cargo test`, `npm run build` und `npm run tauri dev` ausführen. Erst reale Buildfehler lösen; keine Effekte oder Anforderungen dazuerfinden.
2. `Cargo.lock` und `package-lock.json` erzeugen, prüfen und committen. Danach reproduzierbar mit `cargo test --locked` und `npm ci` validieren.
3. Offline-Harness mit mono/stereo und den dokumentierten DSP-Fällen testen. Vor Echtzeit-Integration DSP-Fehler korrigieren und Ergebnis notieren.
4. Erst dann Engine-Grundgerüst: CPAL-Geräte anzeigen, Mic → stiller/ungefilterter VB-Cable-Output, Start/Stop, verlässliche Streamfehler und Reconnect. Noch keine UI-Preset- oder Stufe-2-Arbeit.
5. Echtzeit-DSP erst verbinden, wenn Device-Wechsel, Puffergrenzen, Under-/Overruns und Parameterübergabe getestet sind.
6. Mit Audacity, VB-Cable und Discord manuell testen; Messwerte und offene Fehler in `QUESTIONS.md` dokumentieren. Windows-Test mit echten Geräten kann nicht durch Linux-Builds ersetzt werden.

Änderungen in kleinen, thematischen Commits nach `main` hochladen. Vor jedem Push Secrets ausschließen; nie `--force` auf den geteilten Branch anwenden.

## Fragen und Übergabe
Es sind keine Nutzerentscheidungen blockierend: Sample-Rate, Buffer-Zielbereich, Effektumfang, De-Esser, Limiter und Discord-Setup sind in `PLAN.md` festgelegt. CPALs angefragte Buffergröße ist kein Versprechen über die tatsächliche Callback-Größe; reale Werte und Audiogerätefähigkeit messen und dokumentieren, statt Unterstützung vorzutäuschen. Neue, unvermeidbare Produktfragen zuerst mit Kontext/Optionen in `QUESTIONS.md` dokumentieren, dann nicht mit erfundenen Defaults in einen anderen Scope wechseln.
