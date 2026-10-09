# Fragen und offene Validierung

Hier werden blockierende Fragen gesammelt, damit die Windows-KI sie bei Abwesenheit des Nutzers nachvollziehbar bearbeiten kann. Keine Zugangsdaten oder Tokens eintragen.

## Keine Nutzerentscheidung blockiert den aktuellen Meilenstein

Die bestehende Planung legt Effektumfang, 48 kHz, Buffer-Bereich, De-Esser-Band, Limiter-Ceiling und Discord-Setup bereits fest. Diese Punkte nicht erneut zur Entscheidung stellen.

## Offene technische Validierung

- [ ] Auf Windows `cargo test --manifest-path src-tauri/Cargo.toml` ausführen und eventuelle Build-/Testfehler beheben.
- [ ] `Cargo.lock` und `package-lock.json` erzeugen und committen; danach `cargo test --locked` und `npm ci` erneut prüfen.
- [ ] Offline-Harness mit Mono- und Stereo-WAV (Integer- und Float-Format), Stille, sehr kurzen/ungültigen WAVs und identischen Quell-/Zielpfaden prüfen; Quelldatei muss unverändert bleiben.
- [ ] Prüfen, dass „Sibilanzband abhören“ wirklich isoliertes Band-Monitoring statt des Gesamtsignals liefert.
- [ ] Limiter im nächsten DSP-Schritt mit geeignetem 4× Oversampling-Messverfahren gegen True-Peak-Testsignale validieren; aktuelle kubische Interpolation ist nur ein Prototyp.
- [ ] Im Limiter-Livepfad Peak-Schätzungsaufwand und De-Esser-Potenzen profilieren; vor Optimierung erst CPU-Messwerte bei 48 kHz erfassen.
- [ ] Vor Live-Freigabe Windows-Mikrofon → Processing → VB-Cable → Discord auf Device-Reconnect, Clock-Drift, Klicks, Callback-Underruns, CPU und Ende-zu-Ende-Latenz testen.
- [ ] CPAL-Bufferanforderung gegen tatsächlich beobachtete Callback-Größen verifizieren.

Falls bei der Live-Audio-Implementierung eine neue Produktentscheidung wirklich unvermeidbar ist, hier die konkrete Frage, sinnvolle Optionen, Auswirkungen und vorhandene Evidenz festhalten. Sonst mit den bereits festgelegten Anforderungen aus `PLAN.md` fortfahren.
