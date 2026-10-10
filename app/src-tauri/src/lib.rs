//! La isla de Archito: el panel de escritorio.
//!
//! Esta capa solo conecta la ventana con `archito-core`: toda la lógica vive
//! en el núcleo. La interfaz nunca envía un comando para ejecutar, solo el
//! número de una acción que el núcleo propuso antes.

mod layer;

use std::sync::Mutex;

use archito_core::lessons::{CheckResult, PathView};
use archito_core::{Archito, ExecResult, LogEntry, Response};
use tauri::{Emitter, Manager, State, WebviewWindow};

struct Core(Mutex<Archito>);

type Bot<'a> = State<'a, Core>;

fn with<T>(core: &Bot, f: impl FnOnce(&mut Archito) -> T) -> T {
    // Si un comando anterior entró en pánico, el estado sigue siendo usable.
    let mut guard = core.0.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    f(&mut guard)
}

// Los comandos marcados `async` corren fuera del hilo de la ventana: una sonda
// lenta (checkupdates, ping) no congela el panel.

#[tauri::command(async)]
fn welcome(core: Bot) -> Response {
    with(&core, |a| a.welcome())
}

#[tauri::command(async)]
fn ask(core: Bot, text: String) -> Response {
    with(&core, |a| a.process(&text))
}

#[tauri::command(async)]
fn explain(core: Bot, command: String) -> Response {
    with(&core, |a| a.explain(&command))
}

#[tauri::command(async)]
fn why(core: Bot) -> Response {
    with(&core, |a| a.why())
}

#[tauri::command(async)]
fn run_action(core: Bot, id: u64) -> ExecResult {
    with(&core, |a| a.execute(id))
}

#[tauri::command(async)]
fn cancel_action(core: Bot, id: u64) {
    with(&core, |a| a.cancel(id))
}

#[tauri::command(async)]
fn path(core: Bot) -> PathView {
    with(&core, |a| a.path())
}

#[tauri::command(async)]
fn start_mission(core: Bot, id: String) -> PathView {
    with(&core, |a| {
        a.start_mission(&id);
        a.path()
    })
}

#[tauri::command(async)]
fn check_mission(core: Bot, id: String) -> CheckResult {
    with(&core, |a| a.check_mission(&id))
}

#[tauri::command(async)]
fn logbook(core: Bot) -> Vec<LogEntry> {
    with(&core, |a| a.logbook())
}

/// Lo que se pidió al lanzar el panel: `archito-panel --camino` o
/// `archito-panel cuánta ram tengo`.
#[derive(serde::Serialize, Clone, Default)]
struct Launch {
    view: &'static str,
    text: String,
}

fn parse_launch(args: &[String]) -> Launch {
    let mut launch = Launch::default();
    let mut words: Vec<&str> = Vec::new();
    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--camino" => launch.view = "camino",
            "--bitacora" => launch.view = "bitacora",
            other if other.starts_with("--") => {}
            other => words.push(other),
        }
    }
    launch.text = words.join(" ");
    launch
}

#[tauri::command]
fn launch_request() -> Launch {
    parse_launch(&std::env::args().collect::<Vec<_>>())
}

/// La página dibuja cuadros de verdad.
#[tauri::command]
fn frame_painted() {
    layer::frame_painted();
}

/// La página informa de la forma de la isla (píxeles lógicos de la ventana).
#[tauri::command]
fn set_region(window: WebviewWindow, x: f64, y: f64, w: f64, h: f64) {
    layer::set_region(&window, (x.floor() as i32, y.floor() as i32, w.ceil() as i32, h.ceil() as i32));
}

#[tauri::command]
fn set_expanded(window: WebviewWindow, expanded: bool) {
    layer::set_expanded(&window, expanded);
    if expanded {
        let _ = window.set_focus();
    }
}

pub fn run() {
    tauri::Builder::default()
        // Lanzar `archito-panel` otra vez (el atajo SUPER+Espacio) abre o
        // recoge la isla. Con argumentos, la abre con esa pregunta o vista.
        // La isla nunca se oculta: abrir y recoger lo anima la página.
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            let Some(win) = app.get_webview_window("panel") else { return };
            let launch = parse_launch(&args);
            if launch.view.is_empty() && launch.text.is_empty() {
                let _ = win.emit("toggle", ());
            } else {
                let _ = win.emit("launch", launch);
            }
        }))
        .manage(Core(Mutex::new(Archito::new())))
        .invoke_handler(tauri::generate_handler![
            welcome,
            ask,
            explain,
            why,
            run_action,
            cancel_action,
            path,
            start_mission,
            check_mission,
            logbook,
            launch_request,
            frame_painted,
            set_region,
            set_expanded
        ])
        .setup(|app| {
            if let Some(win) = app.get_webview_window("panel") {
                layer::attach(&win);
                let _ = win.show();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("no se pudo iniciar el panel de Archito");
}
