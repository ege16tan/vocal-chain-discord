# Vocal Chain Discord - Plan v2 (revidiert nach Second Opinion)

## Ziel
Echtzeit-Stimmverarbeitung für Discord/Gaming: Mikrofon → Processing → VB-Cable → Discord. Latenz-Ziel < 20 ms Ende-zu-Ende.

## Revision: Was aus Feedback übernommen wurde
- **Stufen-Schnitt:** Stufe 1 = nur HPF → De-Esser → Limiter + Loudness-Meter (read-only). Kein Preset-System, kein A/B, kein Tray, kein Auto-Start in Stufe 1.
- **Offline-Test-Harness zuerst:** `wav in → wav out` in Rust, bevor Echtzeit-Schleife debuggt wird. DSP-Fehler offline 10x schneller findbar.
- **Latenz-Budget vollständig:** Eigener Buffer + Mic-Treiber + VB-Cable-intern + Discord. VB-Cable-Buffer klein stellen, sonst sind <20 ms nicht haltbar.
- **Robustheit:** Device-Wechsel mid-game (USB-Reconnect) muss Stream neu aufbauen ohne Crash.
- **Kein Auto-Makeup im MVP:** Manuell mit Meter, Auto-Gain ist Fehlerquelle.
- **Unsigned Binary Hinweis:** SmartScreen-Warnung bei Weitergabe einplanen, kein Blocker für lokalen Use.

Nicht übernommen / User-Entscheidung (nicht Teil des Builds):
- Alternative Equalizer APO + TDR Nova + LoudMax als 30-Minuten-Lösung bleibt als Option 0 bestehen, Build geht trotzdem weiter wie gewünscht.

---

## Festgelegte Entscheidungen (ehemals offene Fragen)
1. **Sample Rate:** 48 kHz fix. `src-tauri/src/audio/engine.rs:1`
2. **Buffer Size:** User-wählbar 128–1024, Default 256. Manche USB-Mics mögen kleine Buffer nicht.
3. **Expander vs. Gate:** Nur Expander (Stufe 2). Kein Gate — zu hart für Sprache.
4. **De-Esser Band:** Default 5–8 kHz, Bänder einstellbar 3–12 kHz.
5. **Loudness Target:** -16 LUFS. Nur Meter für User, Discord normalisiert nicht.
6. **Limiter Oversampling:** 4x für Mono 48 kHz OK, messen nicht raten. Optional 2x-Fallback wenn CPU hoch.
7. **Makeup:** Manuell. Kein Auto in Stufe 1/2.
8. **Presets:** Stufe 1 = 1x hardcoded Default. Stufe 2 = lokale JSON, kein Cloud.

---

## Stufe 1 (jetzt bauen): HPF → De-Esser → Limiter

```
Input (Mic) → [HPF 100 Hz] → [De-Esser 5-8 kHz] → [Limiter -1 dBTP] → [Loudness Meter] → Output (VB-Cable)
```

| Modul | Parameter | Default | Bereich |
|-------|-----------|---------|---------|
| **High-Pass** | Freq, Slope 12/24, Bypass | 100 Hz, 12 dB | 20-200 Hz |
| **De-Esser** | Freq Low/High, Thresh, Reduction, Listen, Bypass | 5-8 kHz, -20 dB, 6 dB | 3-12 kHz |
| **Limiter** | Ceiling, Release, Lookahead, True Peak, Bypass | -1 dBTP, 50 ms, 1 ms | -6-0 dBTP |
| **Loudness** | Nur Anzeige: M / S / Integrated | -16 Ref | - |

UI Stufe 1: Ein Fenster ~900x500, Device-Dropdowns In/Out, 3 Modul-Reihen, In/Out-Meter, LUFS/TP/GR-Anzeige, ON/OFF. Kein Tray, kein A/B, kein Preset-Bar.

## Stufe 2 (später, wenn Stufe 1 läuft): Rest
- Expander (-40 dB, 1:3, 10/100 ms) vor De-Esser einfügen
- EQ Presence (3.5 kHz, +2 dB, Q 1.5) + Compressor (-18 dB, 3:1, 15/100 ms, Makeup manuell)
- Preset-System JSON, A/B Compare, Tray + Minimieren, Auto-Start, Installer MSI

Volle Kette Stufe 2:
```
Input → [HPF] → [Expander] → [De-Esser] → [EQ] → [Compressor] → [Limiter] → [Loudness] → Output
```

---

## Tech-Stack (unverändert)

| Component | Choice | Grund |
|-----------|--------|-------|
| App Framework | Tauri 2 (Rust + React/TS) | Native Windows, Tray erst Stufe 2, kein GC-Jitter |
| Audio I/O | CPAL, WASAPI exklusiv wo möglich | Latenz kontrollierbar |
| DSP | Eigenes Rust (Biquad RBJ, Envelope, 4x Oversampling FIR) | Keine Fremd-Deps im Audio-Pfad |
| Loudness | `ebur128` crate | M/S/I nach EBU R128 |
| UI | React + shadcn/ui + Tailwind | Knobs/Slider schnell |
| IPC | Tauri Commands + Events 60 fps Meter | Lock-free Param-Smoothing |

Parameter-Smoothing: Frontend sendet nur Targets (Atomic Float), Rust smootht exponentiell pro Sample gegen Klicks.

---

## Projektstruktur Stufe 1 (reduziert)

```
vocal-chain-discord/
├── src/
│   ├── components/ModuleRow.tsx
│   ├── components/MeterBridge.tsx
│   ├── components/DeviceSelect.tsx
│   ├── hooks/useAudioEngine.ts
│   ├── state/chainStore.ts
│   ├── state/audioParams.ts
│   └── App.tsx
├── src-tauri/src/
│   ├── main.rs
│   ├── audio/engine.rs      # CPAL In->Out, Reconnect-Handling
│   ├── audio/graph.rs       # Fest verdrahtet HPF->DES->LIM
│   ├── audio/nodes/hpf.rs
│   ├── audio/nodes/deesser.rs  # Split-Band + Listen-Mode
│   ├── audio/nodes/limiter.rs  # True Peak 4x + Lookahead
│   ├── audio/nodes/loudness.rs # nur Meter
│   ├── audio/smoothing.rs
│   ├── audio/harness.rs     # Offline WAV in/out Test-Harness
│   ├── ipc/commands.rs
│   └── ipc/events.rs
├── presets/factory-discord-gaming.json # Stufe 1: nur als Referenz-Default, kein UI
└── README.md
```

Stufe 2 fügt hinzu: `nodes/expander.rs`, `nodes/eq.rs`, `nodes/compressor.rs`, `preset.rs`, `config.rs`, Tray in `main.rs`, `PresetBar.tsx`, `usePresets.ts`, `useTray.ts`.

---

## Implementierungs-Reihenfolge (korrigiert)

### 0. Offline-Harness zuerst
- `harness.rs`: WAV lesen → Chain → WAV schreiben, CLI-Flags für Params
- Unit-Tests: Impulse Response HPF, De-Esser GR-Kurve, Limiter Ceiling -1 dBTP wird nie überschritten

### 1. Engine minimal
- 48 kHz fix, Buffer wählbar 128-1024 Default 256
- Device-Liste, Start/Stop, Reconnect bei Device-Loss ohne Crash
- Meter-Events: in_peak, out_peak, gr_des, gr_lim, lufs_m, lufs_s, cpu_ms

### 2. DSP Stufe 1
- HPF: Biquad High-Pass RBJ
- De-Esser: Bandpass-Detektor → GR nur im Band, Listen-Mode zum Abhören
- Limiter: 4x Oversample True Peak, Ceiling -1, Lookahead 1 ms
- Loudness: ebur128 nur Anzeige

### 3. Frontend Stufe 1
- Commands: `get_params`, `set_param`, `list_devices`, `set_devices`
- Keine Preset-Commands in Stufe 1

---

## User-Setup (1x, manuell)

1. **VB-Cable:** https://vb-audio.com/Cable/ installieren → Neustart → Control Panel Buffer klein stellen (128-256 wenn stabil).
2. **Discord:** Input = `CABLE Output`, Input Volume ~80%, **Automatic Gain Control AUS, Noise Suppression AUS (inkl. Krisp), Echo Cancellation AUS** — sonst Doppel-Processing mit eigenem Expander/De-Esser in Stufe 2. In Stufe 1 ohne Expander kann Krisp notfalls AN bleiben, dann aber Redundanz bewusst.
3. **App:** Input = Mikro, Output = `CABLE Input` → ON.

---

## Factory Default Stufe 1 (`discord-gaming.json` als Referenz)

```json
{
  "version": 2,
  "name": "Discord Gaming Stufe1",
  "modules": {
    "hpf": { "freq": 100, "slope": 12, "bypass": false },
    "deesser": { "freq_low": 5000, "freq_high": 8000, "threshold": -20, "reduction": 6, "listen": false, "bypass": false },
    "limiter": { "ceiling": -1, "release": 50, "lookahead": 1, "true_peak": true, "bypass": false },
    "loudness": { "target_lufs": -16 }
  }
}
```

Stufe 2 Default erweitert um: `expander: {-40, 3, 10, 100, 6}`, `eq: {3500, 2, 1.5, bell}`, `compressor: {-18, 3, 15, 100, 6, makeup 3}`.

---

## Aufwand-Einschätzung (korrigiert)
- Alter Plan 8-10h galt für erfahrenen Rust+Tauri+DSP Dev. Realistisch für Erst-Build eher mehr, deshalb Stufen-Schnitt.
- Stufe 1 ist der testbare Meilenstein. Erst wenn HPF/DeEss/Lim offline + live sauber klingen, Stufe 2 starten.
