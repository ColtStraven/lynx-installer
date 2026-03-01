// main.js
// Lynx Installer Shell — UI logic

let __invoke, __listen, __open;

async function setupTauri() {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const { listen }  = await import("@tauri-apps/api/event");
    const { open }    = await import("@tauri-apps/plugin-dialog");
    __invoke = invoke;
    __listen = listen;
    __open   = open;
    return true;
  } catch {
    console.warn("[Lynx] Tauri not available, running in browser stub mode");
    __invoke = async (cmd, args) => {
      console.log("[Lynx stub] invoke:", cmd, args);
      if (cmd === "get_install_config") {
        return {
          app: { name: "My Awesome App", version: "2.1.0", publisher: "Acme Corp", description: "" },
          theme: { accent_color: "#FF6B35", background_color: "#1A1A2E" },
          default_install_dir: "C:\\Program Files\\MyAwesomeApp"
        };
      }
      if (cmd === "start_install") simulateProgressLocally();
      return null;
    };
    __listen = async (event, handler) => {
      window.__lynxProgressHandler = handler;
      return () => {};
    };
    __open = async () => null; // stub — no dialog in browser
    return false;
  }
}

// ─────────────────────────────────────────────
//  DOM refs
// ─────────────────────────────────────────────

const screens = {
  splash:  document.getElementById("screen-splash"),
  install: document.getElementById("screen-install"),
};

const els = {
  splashAppname:    document.getElementById("splash-appname"),
  splashVersion:    document.getElementById("splash-version"),
  installAppname:   document.getElementById("install-appname"),
  installVersion:   document.getElementById("install-version"),
  installPublisher: document.getElementById("install-publisher"),
  panelAppname:     document.getElementById("panel-appname-2"),
  installDirInput:  document.getElementById("install-dir-input"),

  btnStartInstall:  document.getElementById("btn-start-install"),
  btnBrowse:        document.getElementById("btn-browse"),
  btnMinimize:      document.getElementById("btn-minimize"),
  btnClose:         document.getElementById("btn-close"),
  btnFinish:        document.getElementById("btn-finish"),

  panelStep0:       document.getElementById("panel-step-0"),
  panelStep1:       document.getElementById("panel-step-1"),
  panelStep2:       document.getElementById("panel-step-2"),

  progressFill:     document.getElementById("progress-fill"),
  progressGlow:     document.getElementById("progress-glow"),
  progressPct:      document.getElementById("progress-pct"),
  progressFile:     document.getElementById("progress-file"),
  progressLabel:    document.getElementById("progress-step-label"),

  completePath:     document.getElementById("complete-path"),
  completeAppname:  document.getElementById("complete-appname"),
  chkLaunch:        document.getElementById("chk-launch"),

  sidebarSteps:     document.querySelectorAll(".step-item"),
  stepChips:        document.querySelectorAll(".step-chip"),
};

// ─────────────────────────────────────────────
//  Screen transitions
// ─────────────────────────────────────────────

function showScreen(name) {
  Object.entries(screens).forEach(([k, el]) => {
    el.classList.toggle("active", k === name);
  });
}

function showPanelStep(index) {
  [els.panelStep0, els.panelStep1, els.panelStep2].forEach((el, i) => {
    el.classList.toggle("hidden", i !== index);
  });
}

// ─────────────────────────────────────────────
//  Config population
// ─────────────────────────────────────────────

function applyConfig(config) {
  const { app, default_install_dir } = config;

  els.splashAppname.textContent    = app.name;
  els.splashVersion.textContent    = `v${app.version} · by ${app.publisher}`;
  els.installAppname.textContent   = app.name;
  els.installVersion.textContent   = `v${app.version}`;
  els.installPublisher.textContent = `by ${app.publisher}`;
  els.panelAppname.textContent     = app.name;
  els.completeAppname.textContent  = app.name;

  if (default_install_dir) {
    els.installDirInput.value = default_install_dir;
  }
}

// ─────────────────────────────────────────────
//  Sidebar step state
// ─────────────────────────────────────────────

function setSidebarStep(index) {
  els.sidebarSteps.forEach((el, i) => {
    el.classList.toggle("active", i === index);
    el.classList.toggle("done",   i < index);
  });
}

// ─────────────────────────────────────────────
//  Progress updates
// ─────────────────────────────────────────────

let currentStep = 0;
const TOTAL_STEPS = 3;

function onProgressEvent(event) {
  const data = event.payload ?? event;

  switch (data.type) {
    case "started":
      showPanelStep(1);
      setSidebarStep(1);
      els.progressLabel.textContent = "Starting installation...";
      break;

    case "step_begin":
      currentStep = data.step_index;
      els.progressLabel.textContent = data.step_label;
      els.stepChips.forEach((chip, i) => {
        chip.classList.toggle("active", i === currentStep);
        chip.classList.toggle("done",   i < currentStep);
      });
      break;

    case "file_progress": {
      const overall = (currentStep + data.fraction) / TOTAL_STEPS;
      setProgress(overall, data.file_name);
      break;
    }

    case "step_complete":
      els.stepChips.forEach((chip, i) => {
        if (i === data.step_index) chip.classList.add("done");
      });
      break;

    case "complete":
      setProgress(1.0, "Done");
      setTimeout(() => {
        els.completePath.textContent = data.install_dir;
        showPanelStep(2);
        setSidebarStep(2);
      }, 600);
      break;

    case "failed":
      els.progressLabel.textContent = `Error: ${data.error}`;
      els.progressLabel.style.color = "#ff5050";
      break;
  }
}

function setProgress(fraction, fileName) {
  const pct = Math.round(fraction * 100);
  els.progressFill.style.width = `${pct}%`;
  els.progressGlow.style.left  = `calc(${pct}% - 20px)`;
  els.progressPct.textContent  = `${pct}%`;
  if (fileName) els.progressFile.textContent = fileName;
}

// ─────────────────────────────────────────────
//  Browser stub — simulate progress without Tauri
// ─────────────────────────────────────────────

async function simulateProgressLocally() {
  const emit = (data) => {
    if (window.__lynxProgressHandler) window.__lynxProgressHandler({ payload: data });
  };
  const delay = (ms) => new Promise(r => setTimeout(r, ms));

  emit({ type: "started", app_name: "My Awesome App", total_steps: 3 });
  await delay(300);

  const steps = ["Preparing...", "Installing files...", "Creating shortcuts..."];
  for (let s = 0; s < steps.length; s++) {
    emit({ type: "step_begin", step_index: s, step_label: steps[s] });
    for (let f = 0; f < 15; f++) {
      await delay(90);
      emit({ type: "file_progress", step_index: s, file_name: `file_${f}.dat`, fraction: (f+1)/15, bytes_written: (f+1)*1024*50, bytes_total: 15*1024*50 });
    }
    emit({ type: "step_complete", step_index: s, step_label: steps[s] });
    await delay(200);
  }

  await delay(300);
  emit({ type: "complete", install_dir: els.installDirInput.value, duration_ms: 5000 });
}

// ─────────────────────────────────────────────
//  Boot sequence
// ─────────────────────────────────────────────

async function boot() {
  const hasTauri = await setupTauri();

  const config = await __invoke("get_install_config");
  if (config?.error) {
    throw new Error("Config error: " + config.error);
  }
  if (config) applyConfig(config);

  // Make the input editable so users can type a path directly too
  els.installDirInput.removeAttribute("readonly");

  await __listen("progress", onProgressEvent);

  // Browse button — opens native folder picker
  els.btnBrowse.addEventListener("click", async () => {
    const picked = await __open({ directory: true, multiple: false });
    if (picked) {
      els.installDirInput.value = picked;
    }
  });

  els.btnStartInstall.addEventListener("click", async () => {
    const installDir = els.installDirInput.value.trim();
    if (!installDir) return;
    await __invoke("start_install", { installDir });
  });

  els.btnMinimize.addEventListener("click", () => __invoke("shell_minimize"));
  els.btnClose.addEventListener("click",    () => __invoke("shell_close"));
  els.btnFinish.addEventListener("click",   () => __invoke("shell_close"));

  setTimeout(() => {
    showScreen("install");
    if (hasTauri) __invoke("shell_ready");
  }, 2200);
}

async function safeboot() {
  try {
    await boot();
  } catch (err) {
    const loaderText = document.querySelector(".loader-text");
    if (loaderText) {
      loaderText.textContent = "Error: " + (err?.message ?? String(err));
      loaderText.style.color = "#ff5050";
    }
    try { await __invoke?.("shell_ready"); } catch {}
  }
}

safeboot();