//! La isla de Archito: el panel de escritorio.
//!
//! Esta capa solo conecta la ventana con `archito-core`: toda la lógica vive
//! en el núcleo. La interfaz nunca envía un comando para ejecutar, solo el
//! número de una acción que el núcleo propuso antes.

mod layer;

use archito_core::lessons::{CheckResult, PathView};
use archito_core::theme::{self, Palette};
use archito_core::{Archito, ExecResult, LogEntry, Response};
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};

type Bot<'a> = State<'a, Archito>;

fn with<T>(core: &Bot, f: impl FnOnce(&Archito) -> T) -> T {
    f(core.inner())
}

// Los comandos marcados `async` corren fuera del hilo de la ventana, y el
// núcleo no se bloquea entero mientras uno trabaja: una sonda lenta
// (checkupdates, ping) no congela el panel ni las demás preguntas.

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

/// Los colores del escritorio. Leer gsettings tarda un poco: va fuera del hilo
/// de la ventana.
#[tauri::command(async)]
fn palette() -> Palette {
    theme::detect()
}

/// Cierra Archito del todo. La página ya se despidió antes de llamarlo.
#[tauri::command]
fn quit(app: AppHandle) {
    app.exit(0);
}

/// Vigila los archivos de color del escritorio. Cuando cambian (otro fondo de
/// pantalla con matugen, otro esquema en KDE), avisa a la página con la paleta
/// nueva. Mirar la fecha de siete archivos cada dos segundos no cuesta nada.
fn watch_theme(app: AppHandle) {
    std::thread::spawn(move || {
        let mut seen = theme::fingerprint();
        let mut last = theme::detect();
        loop {
            std::thread::sleep(std::time::Duration::from_secs(2));
            let now = theme::fingerprint();
            if now == seen {
                continue;
            }
            seen = now;
            // Los generadores escriben varios archivos seguidos: se espera a
            // que terminen antes de leer.
            std::thread::sleep(std::time::Duration::from_millis(400));
            let palette = theme::detect();
            if palette != last {
                let _ = app.emit("theme", &palette);
                last = palette;
            }
        }
    });
}

/// Lo que se pidió al lanzar el panel: `archito-panel --camino` o
/// `archito-panel cuánta ram tengo`.
#[derive(serde::Serialize, Clone, Default)]
struct Launch {
    view: &'static str,
    text: String,
    /// `archito-panel --cerrar`: la isla se despide y el programa termina.
    quit: bool,
}

fn parse_launch(args: &[String]) -> Launch {
    let mut launch = Launch::default();
    let mut words: Vec<&str> = Vec::new();
    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--camino" => launch.view = "camino",
            "--bitacora" => launch.view = "bitacora",
            "--cerrar" | "--quit" => launch.quit = true,
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

/// Errores de la página, al registro de ARCHITO_DEBUG=1 (en release no hay
/// consola de WebKit donde verlos).
#[tauri::command]
fn log_page(message: String) {
    layer::debug(&format!("página: {message}"));
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
            if launch.view.is_empty() && launch.text.is_empty() && !launch.quit {
                let _ = win.emit("toggle", ());
            } else {
                let _ = win.emit("launch", launch);
            }
        }))
        .manage(Archito::new())
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
            palette,
            quit,
            frame_painted,
            log_page,
            set_region,
            set_expanded
        ])
        .setup(|app| {
            if let Some(win) = app.get_webview_window("panel") {
                layer::attach(&win);
                let _ = win.show();
            }
            watch_theme(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("no se pudo iniciar el panel de Archito");
}
