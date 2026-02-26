// main.js
// Lynx Installer Shell — UI logic
//
// Handles:
//   - Screen transitions (splash → install → complete)
//   - Tauri command invocations
//   - Progress event subscription + UI updates
//   - Window controls (minimize, close)

// ─────────────────────────────────────────────
//  Tauri bridge
//  When running in dev without Tauri, stub it out
//  so we can develop in a normal browser too.
// ─────────────────────────────────────────────

let __invoke, __listen;

async function setupTauri() {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    const { listen }  = await import("@tauri-apps/api/event");
    __invoke = invoke;
    __listen = listen;
    return true;
  } catch {
    // Running in browser dev mode — stub everything
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
      if (cmd === "start_install") {
        simulateProgressLocally();
      }
      return null;
    };
    __listen = async (event, handler) => {
      window.__lynxProgressHandler = handler;
      return () => {};
    };
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

let globalFraction = 0;
let currentStep = 0;
const TOTAL_STEPS = 3;

function onProgressEvent(event) {
  // event.payload when coming from Tauri, event directly in stub mode
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
      // Update step chips
      els.stepChips.forEach((chip, i) => {
        chip.classList.toggle("active", i === currentStep);
        chip.classList.toggle("done",   i < currentStep);
      });
      break;

    case "file_progress": {
      // Overall fraction = (completed steps + current step fraction) / total steps
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
    if (window.__lynxProgressHandler) {
      window.__lynxProgressHandler({ payload: data });
    }
  };

  const delay = (ms) => new Promise(r => setTimeout(r, ms));

  emit({ type: "started", app_name: "My Awesome App", app_version: "2.1.0", total_steps: 3 });
  await delay(300);

  const steps = ["Preparing installation...", "Installing application files...", "Creating shortcuts..."];

  for (let s = 0; s < steps.length; s++) {
    emit({ type: "step_begin", step_index: s, step_label: steps[s] });
    const files = 15;
    for (let f = 0; f < files; f++) {
      await delay(90);
      emit({
        type: "file_progress",
        step_index: s,
        file_name: `file_${String(f).padStart(3,"0")}.dat`,
        fraction: (f + 1) / files,
        bytes_written: (f + 1) * 1024 * 50,
        bytes_total: files * 1024 * 50
      });
    }
    emit({ type: "step_complete", step_index: s, step_label: steps[s] });
    await delay(200);
  }

  await delay(300);
  emit({ type: "complete", install_dir: "C:\\Program Files\\MyAwesomeApp", duration_ms: 5000 });
}

// ─────────────────────────────────────────────
//  Boot sequence
// ─────────────────────────────────────────────

async function boot() {
  const hasTauri = await setupTauri();

  // Load install config
  const config = await __invoke("get_install_config");
  if (config) applyConfig(config);

  // Subscribe to progress events
  await __listen("progress", onProgressEvent);

  // Wire up buttons
  els.btnStartInstall.addEventListener("click", async () => {
    const installDir = els.installDirInput.value;
    await __invoke("start_install", { installDir });
  });

  els.btnMinimize.addEventListener("click", () => __invoke("shell_minimize"));
  els.btnClose.addEventListener("click",    () => __invoke("shell_close"));

  els.btnFinish.addEventListener("click", () => __invoke("shell_close"));

  // Force a repaint so WebView2 triggers CSS animations correctly
  document.body.offsetHeight;
  document.querySelector(".loader-fill").style.willChange = "width";

  // Tell Tauri to show the window immediately — don't wait for splash
  // The window was hidden in tauri.conf.json to prevent white flash on load
  if (hasTauri) __invoke("shell_ready");

  // Splash → Install transition
  // Use a reliable timeout based on the CSS animation duration (1.6s)
  // plus padding for the fade-up animations and a beat at the end
  const SPLASH_DURATION = 2800;
  let transitioned = false;

  function transitionToInstall() {
    if (transitioned) return;
    transitioned = true;
    showScreen("install");
  }

  // Primary: fire off the CSS animation end
  const loaderFill = document.querySelector(".loader-fill");
  loaderFill.addEventListener("animationend", () => {
    setTimeout(transitionToInstall, 500);
  });

  // Fallback: hard timeout in case animationend doesn't fire
  setTimeout(transitionToInstall, SPLASH_DURATION);
}

boot();