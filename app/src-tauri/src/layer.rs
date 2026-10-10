//! La isla como superficie layer-shell de Wayland.
//!
//! Tauri crea una ventana GTK normal. Antes de que se muestre, se la entrega a
//! gtk-layer-shell para que el compositor (Hyprland, Sway, KDE, COSMIC) la
//! ancle arriba y al centro, sobre todo lo demás.
//!
//! La ventana es transparente y de tamaño fijo, más grande que la isla. La
//! isla se dibuja y se anima dentro con CSS; lo que cambia es la *región de
//! entrada*: solo el rectángulo de la isla recibe clics, y el resto de la
//! ventana los deja pasar a lo que haya debajo. Así la animación no depende de
//! redimensionar una ventana cuadro a cuadro.
//!
//! La técnica (llamar a libgtk-layer-shell sobre la ventana de Tauri, la
//! región de entrada y el remapeo inicial) viene de Coucou
//! (github.com/Louis-CFM/coucou, licencia MIT).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use gtk::glib::translate::ToGlibPtr;
use gtk::prelude::*;
use tauri::WebviewWindow;

mod ffi {
    use gtk::ffi::GtkWindow;
    use std::os::raw::{c_char, c_int};

    pub const LAYER_OVERLAY: c_int = 3;
    pub const EDGE_TOP: c_int = 2;
    pub const KEYBOARD_NONE: c_int = 0;
    pub const KEYBOARD_ON_DEMAND: c_int = 2;

    #[link(name = "gtk-layer-shell")]
    extern "C" {
        pub fn gtk_layer_is_supported() -> c_int;
        pub fn gtk_layer_init_for_window(window: *mut GtkWindow);
        pub fn gtk_layer_set_namespace(window: *mut GtkWindow, name_space: *const c_char);
        pub fn gtk_layer_set_layer(window: *mut GtkWindow, layer: c_int);
        pub fn gtk_layer_set_anchor(window: *mut GtkWindow, edge: c_int, anchor: c_int);
        pub fn gtk_layer_set_exclusive_zone(window: *mut GtkWindow, zone: c_int);
        pub fn gtk_layer_set_keyboard_mode(window: *mut GtkWindow, mode: c_int);
    }
}

static IS_LAYER: AtomicBool = AtomicBool::new(false);
static PAINTED: AtomicBool = AtomicBool::new(false);

/// Rectángulo que recibe clics, en píxeles lógicos de la ventana. Arranca
/// vacío: hasta que la página informa de la forma de la isla, nada atrapa el
/// ratón. GTK lo reinicia en cada `map`, así que se guarda y se reaplica.
static REGION: Mutex<(i32, i32, i32, i32)> = Mutex::new((0, 0, 0, 0));

fn window_ptr(gw: &gtk::ApplicationWindow) -> *mut gtk::ffi::GtkWindow {
    let window: &gtk::Window = gw.upcast_ref();
    window.to_glib_none().0
}

/// Ancla la isla arriba al centro. Devuelve `false` si el compositor no tiene
/// layer-shell (GNOME, X11) o si `ARCHITO_LAYER_SHELL=0`: en ese caso queda
/// como una ventana normal sin bordes.
pub fn attach(win: &WebviewWindow) -> bool {
    let Ok(gw) = win.gtk_window() else { return false };
    let wanted = std::env::var("ARCHITO_LAYER_SHELL").map(|v| v != "0").unwrap_or(true);
    let supported = unsafe { ffi::gtk_layer_is_supported() } != 0;
    if !wanted || !supported || gw.is_realized() {
        gw.set_keep_above(true);
        return false;
    }
    // Una superficie layer no lleva barra de título; con ella GTK recalcula
    // una región de entrada propia en cada `map`, encima de la nuestra.
    gw.set_titlebar(None::<&gtk::Widget>);
    let ptr = window_ptr(&gw);
    unsafe {
        ffi::gtk_layer_init_for_window(ptr);
        ffi::gtk_layer_set_namespace(ptr, c"archito".as_ptr());
        ffi::gtk_layer_set_layer(ptr, ffi::LAYER_OVERLAY);
        // Solo el borde superior: el compositor la centra en horizontal.
        ffi::gtk_layer_set_anchor(ptr, ffi::EDGE_TOP, 1);
        // -1: pegada al borde de la pantalla, por encima de la barra si la hay.
        ffi::gtk_layer_set_exclusive_zone(ptr, -1);
        // Recogida no pide teclado; al abrirse, `set_expanded` lo activa.
        ffi::gtk_layer_set_keyboard_mode(ptr, ffi::KEYBOARD_NONE);
    }
    gw.connect_map_event(|w, _| {
        apply_region(w);
        gtk::glib::Propagation::Proceed
    });
    IS_LAYER.store(true, Ordering::Relaxed);
    debug("superficie layer-shell lista");
    keep_remapping_until_painted(gw);
    true
}

fn apply_region(gw: &gtk::ApplicationWindow) {
    if !IS_LAYER.load(Ordering::Relaxed) {
        return;
    }
    let Some(gdk_window) = gw.window() else { return };
    let (x, y, w, h) = *REGION.lock().unwrap_or_else(|p| p.into_inner());
    let rect = gtk::cairo::RectangleInt::new(x, y, w.max(0), h.max(0));
    let region = gtk::cairo::Region::create_rectangle(&rect);
    gdk_window.input_shape_combine_region(&region, 0, 0);
}

/// La página informa de dónde está la isla: solo ese rectángulo recibe clics.
pub fn set_region(win: &WebviewWindow, rect: (i32, i32, i32, i32)) {
    *REGION.lock().unwrap_or_else(|p| p.into_inner()) = rect;
    let handle = win.clone();
    let _ = win.run_on_main_thread(move || {
        if let Ok(gw) = handle.gtk_window() {
            apply_region(&gw);
        }
    });
}

/// Abierta, la isla acepta teclado a demanda (tu terminal lo conserva mientras
/// sigues una misión con la isla abierta). Recogida, no lo pide nunca.
pub fn set_expanded(win: &WebviewWindow, expanded: bool) {
    if !IS_LAYER.load(Ordering::Relaxed) {
        return;
    }
    let handle = win.clone();
    let _ = win.run_on_main_thread(move || {
        if let Ok(gw) = handle.gtk_window() {
            let mode = if expanded { ffi::KEYBOARD_ON_DEMAND } else { ffi::KEYBOARD_NONE };
            unsafe { ffi::gtk_layer_set_keyboard_mode(window_ptr(&gw), mode) };
        }
    });
}

/// WebKitGTK a veces no pinta su primer cuadro en una superficie layer recién
/// mapeada, y la isla queda invisible. Ocultarla y mostrarla lo arregla, pero
/// solo si WebKit ya está listo, y eso tarda más con el equipo ocupado. Por eso
/// se repite cada poco hasta que la página confirma que dibuja cuadros
/// (`frame_painted`). Mientras no hay nada pintado, el parpadeo no se ve.
fn keep_remapping_until_painted(gw: gtk::ApplicationWindow) {
    let mut attempts = 0;
    gtk::glib::timeout_add_local(std::time::Duration::from_millis(600), move || {
        attempts += 1;
        if PAINTED.load(Ordering::Relaxed) || attempts > 60 {
            return gtk::glib::ControlFlow::Break;
        }
        if gw.is_visible() {
            debug(&format!("remapeo {attempts}"));
            gw.hide();
            gw.show_all();
        }
        gtk::glib::ControlFlow::Continue
    });
}

/// La página dibuja cuadros de verdad: ya no hace falta remapear.
pub fn frame_painted() {
    debug("la página confirmó cuadros pintados");
    PAINTED.store(true, Ordering::Relaxed);
}

/// Trazas de arranque con ARCHITO_DEBUG=1.
fn debug(message: &str) {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    let start = START.get_or_init(std::time::Instant::now);
    if std::env::var_os("ARCHITO_DEBUG").is_some() {
        eprintln!("[archito {:>6} ms] {message}", start.elapsed().as_millis());
    }
}
