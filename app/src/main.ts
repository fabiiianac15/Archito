import { listen } from "@tauri-apps/api/event";

import { api, type Action, type Launch, type Mission, type PathView, type Reply } from "./api";
import { add, button, copy, el, rich } from "./dom";
import { farewell, introduce } from "./intro";
import { Island } from "./island";
import { createMascot, pose, startLife, type Pose } from "./mascot";
import { followDesktop } from "./theme";

// Cualquier error de la página llega al registro de ARCHITO_DEBUG=1.
window.addEventListener("error", (e) => void api.logPage(`${e.message} (${e.filename}:${e.lineno})`));
window.addEventListener("unhandledrejection", (e) => void api.logPage(`promesa: ${String(e.reason)}`));

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;

const messages = $<HTMLDivElement>("messages");
const chips = $<HTMLDivElement>("chips");
const input = $<HTMLInputElement>("input");
const pill = $<HTMLSpanElement>("pill");
const subtitle = $<HTMLParagraphElement>("subtitle");
const viewCamino = $<HTMLElement>("view-camino");
const viewBitacora = $<HTMLElement>("view-bitacora");

const island = new Island($<HTMLElement>("island"));
const compactText = $<HTMLSpanElement>("compact-text");

// Tres Archito: la cara en la píldora, la cara en la cabecera y el cuerpo
// entero en el inicio. Los tres cambian de pose a la vez.
const faceSmall = createMascot("mascot-pill", true);
const faceHeader = createMascot("mascot-brand", true);
const buddy = createMascot("mascot-home");
$("compact-avatar").append(faceSmall);
$("brand-avatar").append(faceHeader);
$("home-buddy").append(buddy);
const mascots = [faceSmall, faceHeader, buddy];

// Parpadeos y miradas, solo en los Archito que se ven ahora.
startLife(() => (island.isOpen ? (document.querySelector("#view-inicio.is-on") ? [faceHeader, buddy] : [faceHeader]) : [faceSmall]));

function poseAll(next: Pose, back?: Pose, ms = 0) {
  for (const m of mascots) pose(m, next, back, ms);
}

let busy = false;
/** Misión que se queda desplegada al volver a pintar el camino. */
let openMission: string | null = null;
const history: string[] = [];
let historyAt = 0;

// ── Estado visible ────────────────────────────────────────────────────────────

type Mood = "idle" | "busy" | "warn" | "error" | "happy";

/** Mientras dura el saludo de entrada, la píldora no cambia de texto. */
let greetingUntil = 0;

const MOOD_POSE: Record<Mood, Pose> = {
  idle: "idle",
  busy: "think",
  warn: "worried",
  error: "error",
  happy: "happy",
};

const COMPACT_TEXT: Record<Mood, string> = {
  idle: "Archito",
  busy: "pensando",
  warn: "mira esto",
  error: "algo falló",
  happy: "¡bien hecho!",
};

function setMood(mood: Mood, text: string) {
  pill.className = `pill pill-${mood}`;
  pill.textContent = text;
  poseAll(MOOD_POSE[mood], mood === "happy" ? "idle" : undefined, 2400);
  // La isla recogida cuenta lo mismo con color y una palabra.
  island.root.dataset.mood = mood;
  if (performance.now() > greetingUntil) compactText.textContent = COMPACT_TEXT[mood];
}

function celebrate() {
  poseAll("happy", "idle", 2400);
  island.bump();
}

// ── Sueño ─────────────────────────────────────────────────────────────────────
// Recogida y sin que nadie le hable durante un rato, Archito se duerme. Abrir
// la isla o cualquier respuesta lo despierta.

const SLEEP_AFTER = 3 * 60 * 1000;
let sleepTimer = 0;
let asleep = false;

function wake() {
  window.clearTimeout(sleepTimer);
  if (asleep) {
    asleep = false;
    poseAll("idle");
    compactText.textContent = COMPACT_TEXT.idle;
  }
  sleepTimer = window.setTimeout(() => {
    if (island.isOpen || busy || island.root.dataset.mood !== "idle") return wake();
    asleep = true;
    poseAll("sleep");
    compactText.textContent = "zzz…";
  }, SLEEP_AFTER);
}

async function expand() {
  wake();
  if (island.isOpen) return;
  greetingUntil = 0;
  compactText.textContent = COMPACT_TEXT[(island.root.dataset.mood as Mood) ?? "idle"];
  await island.go("open");
  input.focus();
}

function collapse() {
  if (!island.isOpen) return;
  input.blur();
  disarmQuit();
  void island.go("pill");
  wake();
}

// ── Cerrar el programa ────────────────────────────────────────────────────────
// Dos clics, como lo peligroso: el primero pregunta, el segundo se despide.

const quitButton = $<HTMLButtonElement>("quit");
let quitTimer = 0;

function disarmQuit() {
  window.clearTimeout(quitTimer);
  quitButton.classList.remove("is-armed");
}

async function quit() {
  disarmQuit();
  input.blur();
  if (island.isOpen) await island.go("pill");
  compactText.textContent = "¡Hasta luego!";
  greetingUntil = Number.POSITIVE_INFINITY;
  await farewell(island, faceSmall);
  await api.quit();
}

quitButton.addEventListener("click", () => {
  if (!quitButton.classList.contains("is-armed")) {
    quitButton.classList.add("is-armed");
    quitTimer = window.setTimeout(disarmQuit, 3200);
    return;
  }
  void quit();
});

function scrollDown() {
  requestAnimationFrame(() => (messages.scrollTop = messages.scrollHeight));
}

type ViewName = "inicio" | "chat" | "camino" | "bitacora";

function showView(name: ViewName) {
  for (const tab of document.querySelectorAll<HTMLButtonElement>(".tab")) {
    tab.classList.toggle("is-on", tab.dataset.view === name);
  }
  for (const view of document.querySelectorAll<HTMLElement>(".view")) {
    view.classList.toggle("is-on", view.id === `view-${name}`);
  }
  if (name === "inicio") void refreshSubtitle();
  if (name === "camino") void renderCamino();
  if (name === "bitacora") void renderBitacora();
  if (name === "chat") scrollDown();
}

// ── Chat ──────────────────────────────────────────────────────────────────────

function addUser(text: string) {
  messages.append(el("div", "msg msg-user", text));
  scrollDown();
}

function setChips(list: string[]) {
  chips.replaceChildren(...list.map((chip) => button(chip, "chip", () => void send(chip))));
}

function statBar(stat: Reply["stats"][number]): HTMLElement {
  const fill = el("div", `bar-fill sev-${stat.severity}`);
  fill.style.width = `${Math.max(2, Math.min(100, stat.pct))}%`;
  return el("div", "stat", el("div", "stat-head", el("span", "", stat.label), el("span", "stat-value", stat.value)), el("div", "bar", fill));
}

function stepsBlock(steps: Reply["steps"]): HTMLElement {
  const list = el("div", "steps");
  for (const step of steps) {
    list.append(el("div", "step", el("code", "step-token", step.token), el("div", "step-doc", rich(step.doc))));
  }
  return list;
}

/** "Así lo harías tú": el comando real detrás de la respuesta. */
function learnCard(learn: NonNullable<Reply["learn"]>): HTMLElement {
  const actions = el(
    "div",
    "row",
    button("Explícame", "btn btn-ghost", () => void explain(learn.command)),
    button("Copiar", "btn btn-ghost", (btn) => void copy(learn.command, btn)),
  );
  return el("div", "learn", el("div", "learn-title", "Así lo harías tú"), el("code", "cmd", learn.command), el("p", "learn-note", learn.note), actions);
}

const LEVEL_TAG: Record<Action["level"], string> = {
  safe: "solo lee",
  confirm: "cambia el sistema",
  danger: "peligroso",
};

function actionCard(action: Action): HTMLElement {
  const card = el("div", `action action-${action.level}`);
  const result = el("div", "action-result");
  const settle = (text: string, ok: boolean, output = "") => {
    buttons.remove();
    result.className = `action-result ${ok ? "is-ok" : "is-bad"}`;
    result.append(el("p", "", text));
    if (output) result.append(el("pre", "code", output));
    scrollDown();
  };

  let armed = false;
  const run = button(action.level === "safe" ? "Ver ahora" : "Ejecutar", `btn btn-run btn-${action.level}`, async (btn) => {
    // Lo peligroso pide dos clics: el primero solo arma el botón.
    if (action.level === "danger" && !armed) {
      armed = true;
      btn.textContent = "Sí, entiendo el riesgo: ejecutar";
      window.setTimeout(() => {
        armed = false;
        if (btn.isConnected) btn.textContent = "Ejecutar";
      }, 5000);
      return;
    }
    btn.disabled = true;
    setMood("busy", "ejecutando");
    try {
      // Las acciones de escritorio (una captura, el bloqueo) no deben tener
      // la isla delante: se recoge y espera a que termine de cerrarse.
      if (action.quick && island.isOpen) {
        collapse();
        await new Promise((done) => window.setTimeout(done, 450));
      }
      const out = await api.runAction(action.id);
      if (out.mode === "inline") {
        settle(out.ok ? "Esto respondió tu sistema:" : "El comando falló:", out.ok, out.output);
      } else {
        settle(out.output, out.ok);
      }
      setMood(out.ok ? "idle" : "warn", out.ok ? "listo" : "revisa");
    } catch (err) {
      settle(`No pude ejecutarlo: ${String(err)}`, false);
      setMood("error", "error");
    }
  });
  const buttons = el(
    "div",
    "row",
    run,
    button("Explicar", "btn btn-ghost", () => void explain(action.command)),
    button("Cancelar", "btn btn-ghost", () => {
      void api.cancelAction(action.id);
      settle("Cancelado. No se ejecutó nada.", true);
    }),
  );

  add(
    card,
    el("div", "action-head", el("strong", "", action.label), el("span", `tag tag-${action.level}`, action.quick ? "escritorio" : LEVEL_TAG[action.level])),
    el("code", "cmd", `$ ${action.command}`),
    action.description ? el("p", "action-desc", action.description) : null,
    action.danger_note ? el("p", "action-warn", action.danger_note) : null,
    buttons,
    result,
  );
  return card;
}

function addBot(reply: Reply) {
  const bubble = el("div", `msg msg-bot status-${reply.status}`);
  if (reply.text) bubble.append(el("div", "text", rich(reply.text)));
  for (const stat of reply.stats) bubble.append(statBar(stat));
  if (reply.code) bubble.append(el("pre", "code", reply.code));
  if (reply.steps.length) bubble.append(stepsBlock(reply.steps));
  if (reply.learn) bubble.append(learnCard(reply.learn));
  for (const action of reply.actions) bubble.append(actionCard(action));
  messages.append(bubble);
  setChips(reply.chips);
  wake();
  const mood: Mood = reply.status === "error" ? "error" : reply.status === "warn" ? "warn" : "idle";
  setMood(mood, mood === "idle" ? "listo" : mood === "warn" ? "atención" : "error");
  scrollDown();
}

async function withTyping(work: () => Promise<Reply>) {
  if (busy) return;
  busy = true;
  setMood("busy", "pensando");
  const typing = el("div", "msg msg-bot typing", el("span"), el("span"), el("span"));
  messages.append(typing);
  scrollDown();
  try {
    const reply = await work();
    typing.remove();
    addBot(reply);
    if (reply.view) void refreshSubtitle();
    if (reply.status === "bye") window.setTimeout(collapse, 900);
  } catch (err) {
    typing.remove();
    addBot({ text: `Algo falló dentro de mí: ${String(err)}`, code: "", steps: [], actions: [], chips: ["ayuda"], stats: [], learn: null, status: "error", view: "" });
  } finally {
    busy = false;
  }
}

async function send(text: string) {
  const clean = text.trim();
  if (!clean || busy) return;
  showView("chat");
  history.push(clean);
  historyAt = history.length;
  addUser(clean);
  await withTyping(() => api.ask(clean));
}

async function explain(command: string) {
  if (busy) return;
  showView("chat");
  addUser(`explica ${command}`);
  await withTyping(() => api.explain(command));
}

// ── Camino ────────────────────────────────────────────────────────────────────

/** Resumen del inicio: nivel, avance y la misión que toca. */
async function refreshSubtitle(view?: PathView) {
  const path = view ?? (await api.path());
  subtitle.textContent = `Nivel ${path.level} · ${path.level_name} · ${path.done} de ${path.total} misiones`;
  const goal = path.next_xp ?? path.xp;
  $<HTMLDivElement>("home-bar").style.width = `${goal ? Math.min(100, (path.xp / goal) * 100) : 100}%`;
  const next = path.missions.find((m) => m.state !== "done");
  $<HTMLButtonElement>("home-next").textContent = next ? `Siguiente misión: ${next.title} →` : "Camino completo. Ver mis misiones →";
}

function missionCard(mission: Mission, isNext: boolean): HTMLElement {
  const icon = mission.state === "done" ? "✔" : mission.state === "active" ? "▶" : "○";
  const card = el("article", `mission mission-${mission.state}`);
  const feedback = el("p", "mission-feedback");
  const body = el("div", "mission-body");
  const steps = el("ol", "mission-steps");
  for (const step of mission.steps) steps.append(el("li", "", rich(step)));

  const primary =
    mission.state === "done"
      ? null
      : mission.state === "open"
        ? button("Empezar misión", "btn btn-run btn-safe", async () => {
            openMission = mission.id;
            paintCamino(await api.startMission(mission.id));
          })
        : button("Comprobar", "btn btn-run btn-safe", async (btn) => {
            btn.disabled = true;
            btn.textContent = "Mirando tu sistema…";
            const result = await api.checkMission(mission.id);
            if (result.done) {
              celebrate();
              setMood("happy", `+${result.xp} XP`);
              openMission = null;
              await renderCamino(`Misión cumplida: ${mission.title}. ${result.message}${result.level_up ? ` Subiste de nivel: ${result.level_up}.` : ""}`);
            } else {
              btn.disabled = false;
              btn.textContent = "Comprobar";
              feedback.textContent = result.message;
            }
          });

  add(
    body,
    el("p", "mission-why", mission.why),
    steps,
    el("div", "row", el("code", "cmd", mission.command), button("Explícame", "btn btn-ghost", () => void explain(mission.command))),
    mission.state === "active" ? el("p", "mission-hint", "Hazlo en tu terminal. Yo no lo ejecuto por ti: la idea es que lo escribas tú.") : null,
    primary ? el("div", "row", primary) : null,
    feedback,
  );

  const head = button("", "mission-head", () => {
    openMission = card.classList.toggle("is-open") ? mission.id : null;
  });
  head.append(el("span", "mission-icon", icon), el("span", "mission-title", mission.title), el("span", "mission-xp", `+${mission.xp} XP`));
  card.append(head, body);
  if (openMission === mission.id || (openMission === null && isNext)) card.classList.add("is-open");
  return card;
}

function paintCamino(path: PathView, banner = "") {
  void refreshSubtitle(path);
  const goal = path.next_xp ?? path.xp;
  const fill = el("div", "bar-fill sev-good");
  fill.style.width = `${goal ? Math.min(100, (path.xp / goal) * 100) : 100}%`;
  const header = el(
    "div",
    "path-head",
    el("div", "path-level", el("strong", "", `Nivel ${path.level}`), el("span", "", path.level_name)),
    el("div", "bar", fill),
    el("p", "path-meta", `${path.xp} XP${path.next_xp ? ` de ${path.next_xp} para el siguiente nivel` : ""} · ${path.done} de ${path.total} misiones`),
  );
  const nodes: HTMLElement[] = [header];
  if (banner) nodes.push(el("div", "banner", banner));
  if (!path.hook) {
    nodes.push(
      el(
        "div",
        "hint",
        rich("Para que compruebe tus misiones al instante, conecta tu terminal una sola vez: escribe `archito hook --install` y abre una terminal nueva."),
      ),
    );
  }
  const next = path.missions.find((m) => m.state !== "done");
  let stage = "";
  for (const mission of path.missions) {
    if (mission.stage !== stage) {
      stage = mission.stage;
      nodes.push(el("h2", "stage", stage));
    }
    nodes.push(missionCard(mission, mission.id === next?.id));
  }
  viewCamino.replaceChildren(...nodes);
}

async function renderCamino(banner = "") {
  paintCamino(await api.path(), banner);
}

// ── Bitácora ──────────────────────────────────────────────────────────────────

async function renderBitacora() {
  const entries = await api.logbook();
  const nodes: HTMLElement[] = [
    el(
      "div",
      "path-head",
      el("div", "path-level", el("strong", "", `${entries.length} comandos`), el("span", "", "que ya viste conmigo")),
      el("p", "path-meta", "Tu chuleta personal. Se llena sola con cada comando que te explico o que ejecutamos."),
    ),
  ];
  if (!entries.length) {
    nodes.push(el("div", "hint", "Todavía está vacía. Pregúntame algo de tu sistema o pídeme que te explique un comando."));
  }
  for (const entry of entries) {
    const row = button("", "log-row", () => void explain(entry.program));
    row.append(el("code", "log-name", entry.program), el("span", "log-doc", entry.doc), el("span", "log-times", `×${entry.times}`));
    nodes.push(row);
  }
  viewBitacora.replaceChildren(...nodes);
}

// ── Arranque ──────────────────────────────────────────────────────────────────

for (const tab of document.querySelectorAll<HTMLButtonElement>(".tab")) {
  tab.addEventListener("click", () => showView(tab.dataset.view as ViewName));
}
for (const tile of document.querySelectorAll<HTMLButtonElement>(".tile")) {
  tile.addEventListener("click", () => void send(tile.dataset.ask ?? ""));
}
$<HTMLButtonElement>("home-next").addEventListener("click", () => showView("camino"));

$<HTMLFormElement>("composer").addEventListener("submit", (event) => {
  event.preventDefault();
  const text = input.value;
  input.value = "";
  void send(text);
});

input.addEventListener("keydown", (event) => {
  if (event.key === "ArrowUp" && historyAt > 0) {
    historyAt -= 1;
    input.value = history[historyAt];
    event.preventDefault();
  } else if (event.key === "ArrowDown" && historyAt < history.length) {
    historyAt += 1;
    input.value = history[historyAt] ?? "";
    event.preventDefault();
  }
});

$<HTMLButtonElement>("close").addEventListener("click", collapse);
$<HTMLButtonElement>("compact").addEventListener("click", () => void expand());
window.addEventListener("keydown", (event) => {
  if (event.key === "Escape") collapse();
});
void listen("toggle", () => (island.isOpen ? collapse() : void expand()));

function applyLaunch(launch: Launch) {
  if (launch.quit) return void quit();
  if (!launch.view && !launch.text) return;
  void expand();
  if (launch.view) showView(launch.view);
  if (launch.text) void send(launch.text);
}
void listen<Launch>("launch", (event) => applyLaunch(event.payload));

// Un cuadro suelto de requestAnimationFrame no prueba que el panel se vea
// (WebKit puede dar uno antes de presentar nada). Veinte seguidos sí: entonces
// se avisa al backend para que deje de remapear la capa.
let frames = 0;
const countFrame = () => {
  frames += 1;
  if (frames >= 20) void api.framePainted();
  else requestAnimationFrame(countFrame);
};
requestAnimationFrame(countFrame);

function greetingWord(): string {
  const hour = new Date().getHours();
  return hour < 12 ? "Buenos días" : hour < 19 ? "Buenas tardes" : "Buenas noches";
}

void (async () => {
  await followDesktop();
  $("home-hello").textContent = `${greetingWord()}.`;
  const launch = await api.launchRequest().catch((): Launch => ({ view: "", text: "", quit: false }));
  if (launch.quit) return void api.quit();
  // La entrada corre a la vez que se carga lo demás.
  const entrance = introduce(island, $("compact-avatar"), "¡Hola! Soy Archito");
  wake();
  try {
    addBot(await api.welcome());
    await refreshSubtitle();
    await entrance;
    applyLaunch(launch);
  } catch (err) {
    messages.append(el("div", "msg msg-bot status-error", `No pude hablar con el núcleo: ${String(err)}`));
  }
  input.focus();
})();
