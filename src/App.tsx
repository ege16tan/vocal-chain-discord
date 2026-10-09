import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open, save } from "@tauri-apps/plugin-dialog";

type ChainParams = {
  hpfHz: number;
  hpfSlopeDb: number;
  hpfBypass: boolean;
  deesserLowHz: number;
  deesserHighHz: number;
  deesserThresholdDb: number;
  deesserReductionDb: number;
  deesserListen: boolean;
  deesserBypass: boolean;
  limiterCeilingDb: number;
  limiterReleaseMs: number;
  limiterLookaheadMs: number;
  limiterBypass: boolean;
};

type ProcessReport = {
  samples: number;
  durationSeconds: number;
  inputPeakDbfs: number | null;
  outputPeakDbfs: number | null;
  integratedLufs: number | null;
};

type AudioDevice = { id: string; name: string };
type AudioDevices = { inputs: AudioDevice[]; outputs: AudioDevice[] };
type AudioStatus = {
  state: "running" | "stopped" | "reconnecting" | "error";
  message: string | null;
};
type AudioMetrics = {
  inputPeakDbfs: number | null;
  outputPeakDbfs: number | null;
  queueOverruns: number;
  queueUnderruns: number;
};

const defaults: ChainParams = {
  hpfHz: 100,
  hpfSlopeDb: 12,
  hpfBypass: false,
  deesserLowHz: 5_000,
  deesserHighHz: 8_000,
  deesserThresholdDb: -20,
  deesserReductionDb: 6,
  deesserListen: false,
  deesserBypass: false,
  limiterCeilingDb: -1,
  limiterReleaseMs: 50,
  limiterLookaheadMs: 1,
  limiterBypass: false,
};

function meterValue(dbfs: number | null) {
  return dbfs == null ? 0 : Math.max(0, Math.min(60, dbfs + 60));
}

function App() {
  const [inputPath, setInputPath] = useState("");
  const [outputPath, setOutputPath] = useState("");
  const [params, setParams] = useState(defaults);
  const [report, setReport] = useState<ProcessReport | null>(null);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [devices, setDevices] = useState<AudioDevices>({ inputs: [], outputs: [] });
  const [inputDevice, setInputDevice] = useState("");
  const [outputDevice, setOutputDevice] = useState("");
  const [bufferFrames, setBufferFrames] = useState(256);
  const [audioStatus, setAudioStatus] = useState<AudioStatus>({
    state: "stopped",
    message: null,
  });
  const [audioMetrics, setAudioMetrics] = useState<AudioMetrics>({
    inputPeakDbfs: null,
    outputPeakDbfs: null,
    queueOverruns: 0,
    queueUnderruns: 0,
  });
  const audioActive = audioStatus.state === "running" || audioStatus.state === "reconnecting";

  useEffect(() => {
    let mounted = true;
    let unlisten: (() => void) | undefined;
    let unlistenMetrics: (() => void) | undefined;

    void listen<AudioStatus>("audio-status", (event) => {
      if (mounted) setAudioStatus(event.payload);
    }).then((stopListening) => {
      if (mounted) unlisten = stopListening;
      else stopListening();
    }).catch((cause: unknown) => {
      if (mounted) {
        setAudioStatus({
          state: "error",
          message: cause instanceof Error ? cause.message : String(cause),
        });
      }
    });
    void listen<AudioMetrics>("audio-metrics", (event) => {
      if (mounted) setAudioMetrics(event.payload);
    }).then((stopListening) => {
      if (mounted) unlistenMetrics = stopListening;
      else stopListening();
    }).catch((cause: unknown) => {
      if (mounted) {
        setAudioStatus({
          state: "error",
          message: cause instanceof Error ? cause.message : String(cause),
        });
      }
    });

    void refreshAudioDevices().catch((cause: unknown) => {
      if (mounted) {
        setAudioStatus({
          state: "error",
          message: cause instanceof Error ? cause.message : String(cause),
        });
      }
    });

    return () => {
      mounted = false;
      unlisten?.();
      unlistenMetrics?.();
    };
  }, []);

  async function refreshAudioDevices() {
    const listed = await invoke<AudioDevices>("list_audio_devices");
    setDevices(listed);
    setInputDevice((current) =>
      listed.inputs.some((device) => device.id === current)
        ? current
        : listed.inputs[0]?.id ?? "",
    );
    setOutputDevice((current) =>
      listed.outputs.some((device) => device.id === current)
        ? current
        : listed.outputs.find((device) => /cable input/i.test(device.name))?.id ?? "",
    );
  }

  async function startAudio() {
    setError("");
    setAudioStatus({ state: "reconnecting", message: "Audiostream wird gestartet." });
    try {
      await invoke("start_audio", {
        inputDevice,
        outputDevice,
        bufferFrames,
        params,
      });
    } catch (cause) {
      setAudioStatus({
        state: "error",
        message: cause instanceof Error ? cause.message : String(cause),
      });
    }
  }

  async function stopAudio() {
    try {
      await invoke("stop_audio");
    } catch (cause) {
      setAudioStatus({
        state: "error",
        message: cause instanceof Error ? cause.message : String(cause),
      });
    }
  }

  function update<K extends keyof ChainParams>(key: K, value: ChainParams[K]) {
    setParams((current) => ({ ...current, [key]: value }));
    setReport(null);
  }

  async function chooseInput() {
    const selected = await open({
      multiple: false,
      filters: [{ name: "WAV-Audio", extensions: ["wav"] }],
    });
    if (typeof selected === "string") setInputPath(selected);
  }

  async function chooseOutput() {
    const selected = await save({
      defaultPath: "vocal-chain-output.wav",
      filters: [{ name: "WAV-Audio", extensions: ["wav"] }],
    });
    if (selected) setOutputPath(selected);
  }

  async function processFile() {
    setBusy(true);
    setError("");
    setReport(null);
    try {
      const result = await invoke<ProcessReport>("process_wav", {
        inputPath: inputPath.trim(),
        outputPath: outputPath.trim(),
        params,
      });
      setReport(result);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="app-shell">
      <header className="topbar">
        <div className="brand-mark" aria-hidden="true">VC</div>
        <div>
          <p className="eyebrow">VOCAL CHAIN · STUFE 1</p>
          <h1>Discord Voice</h1>
        </div>
        <span className={`status-pill status-${audioStatus.state}`}><i /> {audioStatus.state === "running" ? "Audio aktiv" : audioStatus.state === "reconnecting" ? "Verbindung" : audioStatus.state === "error" ? "Audiofehler" : "Audio gestoppt"}</span>
      </header>

      <section className="intro">
        <div>
          <h2>Stimme formen. WAV prüfen.</h2>
          <p>HPF → De-Esser → Limiter. Die Einstellungen werden beim Start in den Live-Pfad übernommen.</p>
        </div>
        <div className="sample-rate"><strong>48</strong><span>kHz<br />Projektziel</span></div>
      </section>

      <section className="audio-panel" aria-label="Live-Audio-Verbindung">
        <div className="audio-panel-heading">
          <div>
            <p className="eyebrow">ENGINE · 48 kHz · LIVE-DSP</p>
            <h2>Mikrofon-Verbindung</h2>
          </div>
          <button type="button" className="refresh-button" onClick={() => void refreshAudioDevices()} disabled={audioActive}>Geräte aktualisieren</button>
        </div>
        <div className="audio-controls">
          <label className="audio-select">
            <span>Eingabe · Mikrofon</span>
            <select value={inputDevice} onChange={(event) => setInputDevice(event.target.value)} disabled={audioActive}>
              {devices.inputs.length === 0 && <option value="">Kein Mikrofon gefunden</option>}
              {devices.inputs.map((device) => <option key={device.id} value={device.id}>{device.name}</option>)}
            </select>
          </label>
          <label className="audio-select">
            <span>Ausgabe · VB-CABLE Input</span>
            <select value={outputDevice} onChange={(event) => setOutputDevice(event.target.value)} disabled={audioActive}>
              {devices.outputs.length === 0 && <option value="">Kein Ausgabegerät gefunden</option>}
              {devices.outputs.map((device) => <option key={device.id} value={device.id}>{device.name}</option>)}
            </select>
          </label>
          <label className="audio-select buffer-select">
            <span>Buffer · 48 kHz</span>
            <select value={bufferFrames} onChange={(event) => setBufferFrames(Number(event.target.value))} disabled={audioActive}>
              {[128, 256, 512, 1024].map((size) => <option key={size} value={size}>{size} Samples</option>)}
            </select>
          </label>
          {audioActive
            ? <button type="button" className="engine-button stop-button" onClick={() => void stopAudio()}>Audio stoppen</button>
            : <button type="button" className="engine-button" onClick={() => void startAudio()} disabled={!inputDevice || !outputDevice}>Audio starten</button>}
        </div>
        <p className={`audio-status-message ${audioStatus.state === "error" ? "error-message" : ""}`} aria-live="polite">
          {audioStatus.message ?? (audioStatus.state === "running" ? "48-kHz-Live-Verarbeitung aktiv. Eingestellte DSP-Werte gelten bis zum nächsten Neustart." : "Live-Pfad: HPF → De-Esser → Limiter. Pegel und Queue-Aussetzer erscheinen während der Verbindung.")}
        </p>
        <div className="live-meters" aria-label="Live-Pegel">
          <label><span>Eingang <b>{audioMetrics.inputPeakDbfs == null ? "−∞" : `${audioMetrics.inputPeakDbfs.toFixed(1)} dBFS`}</b></span><meter min="0" max="60" value={meterValue(audioMetrics.inputPeakDbfs)} /></label>
          <label><span>Ausgang <b>{audioMetrics.outputPeakDbfs == null ? "−∞" : `${audioMetrics.outputPeakDbfs.toFixed(1)} dBFS`}</b></span><meter min="0" max="60" value={meterValue(audioMetrics.outputPeakDbfs)} /></label>
          <span className="drop-count">Überläufe {audioMetrics.queueOverruns} · Unterläufe {audioMetrics.queueUnderruns}</span>
        </div>
        <p className="audio-safety-note">Für den Test Kopfhörer oder VB-CABLE verwenden; Mikrofon-Passthrough auf Lautsprecher kann Rückkopplung verursachen.</p>
      </section>

      <section className="file-panel">
        <div className="path-field">
          <label htmlFor="input-path">Eingabe · WAV-Datei</label>
          <div className="path-entry"><input id="input-path" value={inputPath} onChange={(event) => setInputPath(event.target.value)} placeholder="C:/Audio/aufnahme.wav" spellCheck={false} /><button type="button" onClick={chooseInput}>Datei wählen</button></div>
        </div>
        <div className="path-field">
          <label htmlFor="output-path">Ausgabe · neue Float-WAV</label>
          <div className="path-entry"><input id="output-path" value={outputPath} onChange={(event) => setOutputPath(event.target.value)} placeholder="C:/Audio/aufnahme-processed.wav" spellCheck={false} /><button type="button" onClick={chooseOutput}>Speicherort</button></div>
        </div>
      </section>

      <section className="module-grid" aria-label="Audiomodule">
        <article className="module-card">
          <div className="module-heading">
            <div><span className="module-index">01</span><h3>High-Pass</h3></div>
            <label className="toggle">            <input type="checkbox" checked={params.hpfBypass} onChange={(event) => update("hpfBypass", event.target.checked)} disabled={audioActive} /><span>Bypass</span></label>
          </div>
          <p className="module-description">Entfernt Rumpeln und Trittschall unterhalb der Stimme.</p>
          <label className="control">
            <span>Frequenz <b>{params.hpfHz} Hz</b></span>
            <input type="range" min="20" max="200" step="1" value={params.hpfHz} onChange={(event) => update("hpfHz", Number(event.target.value))} disabled={audioActive} />
          </label>
          <label className="control select-control">
            <span>Flanke</span>
            <select value={params.hpfSlopeDb} onChange={(event) => update("hpfSlopeDb", Number(event.target.value))} disabled={audioActive}><option value={12}>12 dB / Oktave</option><option value={24}>24 dB / Oktave</option></select>
          </label>
        </article>

        <article className="module-card">
          <div className="module-heading">
            <div><span className="module-index">02</span><h3>De-Esser</h3></div>
            <label className="toggle"><input type="checkbox" checked={params.deesserBypass} onChange={(event) => update("deesserBypass", event.target.checked)} disabled={audioActive} /><span>Bypass</span></label>
          </div>
          <p className="module-description">Zähmt Zischlaute im einstellbaren Sibilanzband.</p>
          <label className="control">
            <span>Untere Bandgrenze <b>{(params.deesserLowHz / 1000).toFixed(1)} kHz</b></span>
            <input type="range" min="3000" max={Math.min(9000, params.deesserHighHz - 100)} step="100" value={params.deesserLowHz} onChange={(event) => update("deesserLowHz", Number(event.target.value))} disabled={audioActive} />
          </label>
          <label className="control">
            <span>Obere Bandgrenze <b>{(params.deesserHighHz / 1000).toFixed(1)} kHz</b></span>
            <input type="range" min={Math.max(4000, params.deesserLowHz + 100)} max="12000" step="100" value={params.deesserHighHz} onChange={(event) => update("deesserHighHz", Number(event.target.value))} disabled={audioActive} />
          </label>
          <label className="control">
            <span>Schwelle <b>{params.deesserThresholdDb} dB</b></span>
            <input type="range" min="-60" max="0" step="1" value={params.deesserThresholdDb} onChange={(event) => update("deesserThresholdDb", Number(event.target.value))} disabled={audioActive} />
          </label>
          <label className="control">
            <span>Max. Reduktion <b>{params.deesserReductionDb} dB</b></span>
            <input type="range" min="0" max="24" step="1" value={params.deesserReductionDb} onChange={(event) => update("deesserReductionDb", Number(event.target.value))} disabled={audioActive} />
          </label>
          <label className="toggle listen-toggle"><input type="checkbox" checked={params.deesserListen} onChange={(event) => update("deesserListen", event.target.checked)} disabled={audioActive} /><span>Sibilanzband abhören</span></label>
        </article>

        <article className="module-card">
          <div className="module-heading">
            <div><span className="module-index">03</span><h3>Limiter</h3></div>
            <label className="toggle"><input type="checkbox" checked={params.limiterBypass} onChange={(event) => update("limiterBypass", event.target.checked)} disabled={audioActive} /><span>Bypass</span></label>
          </div>
          <p className="module-description">Fängt Spitzen mit kurzem Lookahead und 4× Peak-Schätzung ab.</p>
          <label className="control">
            <span>Ceiling <b>{params.limiterCeilingDb} dBTP</b></span>
            <input type="range" min="-6" max="0" step="0.1" value={params.limiterCeilingDb} onChange={(event) => update("limiterCeilingDb", Number(event.target.value))} disabled={audioActive} />
          </label>
          <label className="control">
            <span>Release <b>{params.limiterReleaseMs} ms</b></span>
            <input type="range" min="1" max="200" step="1" value={params.limiterReleaseMs} onChange={(event) => update("limiterReleaseMs", Number(event.target.value))} disabled={audioActive} />
          </label>
          <label className="control">
            <span>Lookahead <b>{params.limiterLookaheadMs.toFixed(1)} ms</b></span>
            <input type="range" min="0" max="10" step="0.1" value={params.limiterLookaheadMs} onChange={(event) => update("limiterLookaheadMs", Number(event.target.value))} disabled={audioActive} />
          </label>
          <div className="meter-reference"><span>Loudness-Referenz</span><strong>-16 LUFS</strong><small>Anzeige, keine automatische Verstärkung</small></div>
        </article>
      </section>

      <footer className="action-row">
        <div className="result" aria-live="polite">
          {error && <p className="error-message">{error}</p>}
          {report && <p><strong>{report.durationSeconds.toFixed(2)} s</strong> verarbeitet <span>·</span> Peak {report.outputPeakDbfs == null ? "−∞" : `${report.outputPeakDbfs.toFixed(1)} dBFS`} <span>·</span> {report.integratedLufs == null ? "LUFS nicht messbar" : `${report.integratedLufs.toFixed(1)} LUFS integriert`}</p>}
          {!error && !report && <p>Mono- und Stereo-WAV · Quelldatei bleibt unverändert</p>}
        </div>
        <button className="process-button" onClick={processFile} disabled={busy || !inputPath.trim() || !outputPath.trim()}>
          {busy ? "Wird verarbeitet …" : "WAV verarbeiten"}
          {!busy && <span aria-hidden="true">↗</span>}
        </button>
      </footer>
      <p className="scope-note">Live-DSP übernimmt die Einstellungen beim Start; zum Ändern Audio stoppen und erneut starten.</p>
    </main>
  );
}

export default App;
