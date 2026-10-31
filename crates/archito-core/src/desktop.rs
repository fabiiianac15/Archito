//! El entorno en el que corre Archito: escritorio, sesión, distro y ayudante
//! del AUR. Con eso cada tarea de escritorio (capturas, bloqueo, brillo,
//! volumen) usa la herramienta que de verdad existe en este equipo, y lo que
//! Archito enseña es el comando de *tu* escritorio, no uno genérico.
//!
//! Solo lee variables de entorno, `/etc/os-release`, `/proc` y `/sys`.

use std::fs;
use std::sync::OnceLock;

use crate::exec::has_tool;
use crate::memory::home_dir;

/// Familias de escritorio que se comportan distinto para Archito.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Family {
    Hyprland,
    Sway,
    Niri,
    /// Otros compositores Wayland con layer-shell: river, wayfire, labwc…
    Wlroots,
    Kde,
    Gnome,
    Cosmic,
    Xfce,
    /// Cinnamon, MATE, LXQt, Budgie: escritorios completos con su propio bloqueo.
    Classic,
    /// Gestores de ventanas de X11: i3, bspwm, openbox, awesome…
    X11Wm,
    Unknown,
}

pub struct Desktop {
    pub family: Family,
    /// Nombre para mostrar: "Hyprland", "KDE Plasma"…
    pub name: String,
    pub wayland: bool,
}

pub struct Distro {
    /// `ID` de os-release: arch, cachyos, endeavouros, manjaro…
    pub id: String,
    pub name: String,
    /// Usa systemd. Artix no, y ahí `systemctl` no existe.
    pub systemd: bool,
}

fn env(key: &str) -> Option<String> {
    std::env::var(key).ok().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

fn read(path: &str) -> Option<String> {
    fs::read_to_string(path).ok()
}

/// El escritorio actual. Se detecta una vez por proceso.
pub fn desktop() -> &'static Desktop {
    static CACHE: OnceLock<Desktop> = OnceLock::new();
    CACHE.get_or_init(detect_desktop)
}

fn detect_desktop() -> Desktop {
    let wayland = env("WAYLAND_DISPLAY").is_some() || env("XDG_SESSION_TYPE").is_some_and(|t| t == "wayland");
    // Los sockets de cada compositor son más fiables que XDG_CURRENT_DESKTOP,
    // que algunos inicios de sesión no rellenan.
    let by_socket = if env("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
        Some(Family::Hyprland)
    } else if env("NIRI_SOCKET").is_some() {
        Some(Family::Niri)
    } else if env("SWAYSOCK").is_some() {
        Some(Family::Sway)
    } else {
        None
    };
    let raw = env("XDG_CURRENT_DESKTOP").or_else(|| env("XDG_SESSION_DESKTOP")).or_else(|| env("DESKTOP_SESSION")).unwrap_or_default();
    let lower = raw.to_lowercase();
    let has = |name: &str| lower.split([':', ';']).any(|part| part.trim() == name);
    let family = by_socket.unwrap_or_else(|| {
        if has("hyprland") {
            Family::Hyprland
        } else if has("sway") {
            Family::Sway
        } else if has("niri") {
            Family::Niri
        } else if has("kde") || has("plasma") {
            Family::Kde
        } else if has("gnome") || has("ubuntu") {
            Family::Gnome
        } else if has("cosmic") {
            Family::Cosmic
        } else if has("xfce") {
            Family::Xfce
        } else if ["x-cinnamon", "cinnamon", "mate", "lxqt", "budgie", "budgie-desktop", "lxde"].iter().any(|d| has(d)) {
            Family::Classic
        } else if ["river", "wayfire", "labwc", "hikari", "dwl", "qtile"].iter().any(|d| has(d)) && wayland {
            Family::Wlroots
        } else if ["i3", "bspwm", "openbox", "awesome", "dwm", "herbstluftwm", "qtile", "xmonad", "fluxbox"].iter().any(|d| has(d)) {
            Family::X11Wm
        } else {
            Family::Unknown
        }
    });
    let name = match family {
        Family::Hyprland => "Hyprland".to_string(),
        Family::Sway => "Sway".to_string(),
        Family::Niri => "niri".to_string(),
        Family::Kde => "KDE Plasma".to_string(),
        Family::Gnome => "GNOME".to_string(),
        Family::Cosmic => "COSMIC".to_string(),
        Family::Xfce => "Xfce".to_string(),
        _ if !raw.is_empty() => raw.split(':').next().unwrap_or(&raw).to_string(),
        _ => "un escritorio que no reconozco".to_string(),
    };
    Desktop { family, name, wayland }
}

impl Desktop {
    pub fn session(&self) -> &'static str {
        if self.wayland {
            "Wayland"
        } else {
            "X11"
        }
    }

    /// Puede anclar la isla como superficie layer-shell. GNOME y X11 no.
    pub fn layer_shell(&self) -> bool {
        self.wayland && !matches!(self.family, Family::Gnome | Family::Xfce | Family::Classic | Family::X11Wm)
    }

    /// Paquetes que, si se actualizan, cambian el propio escritorio: conviene
    /// cerrar sesión o reiniciar después.
    pub fn core_packages(&self) -> &'static [&'static str] {
        match self.family {
            Family::Hyprland => &["hyprland", "aquamarine", "hyprutils"],
            Family::Sway => &["sway", "wlroots", "wlroots0.18", "wlroots0.19"],
            Family::Niri => &["niri"],
            Family::Kde => &["plasma-workspace", "kwin", "plasma-desktop", "qt6-base"],
            Family::Gnome => &["gnome-shell", "mutter", "gdm"],
            Family::Cosmic => &["cosmic-session", "cosmic-comp"],
            Family::Xfce => &["xfwm4", "xfce4-session"],
            Family::Classic | Family::X11Wm | Family::Unknown | Family::Wlroots => &[],
        }
    }

    /// Cómo se pone un atajo para abrir la isla en este escritorio: (dónde, qué).
    pub fn shortcut_hint(&self, program: &str) -> (String, String) {
        match self.family {
            // Hyprland reciente puede configurarse en Lua; si existe
            // hyprland.lua, es el que manda.
            Family::Hyprland if hypr_lua() => {
                ("En tu configuración Lua de Hyprland (~/.config/hypr/hyprland.lua o un archivo que cargue):".into(), format!("hl.bind(\"SUPER + space\", hl.dsp.exec_cmd(\"{program}\"))"))
            }
            Family::Hyprland => ("En ~/.config/hypr/hyprland.conf:".into(), format!("bind = SUPER, space, exec, {program}")),
            Family::Sway => ("En ~/.config/sway/config:".into(), format!("bindsym $mod+space exec {program}")),
            Family::Niri => ("Dentro del bloque binds de ~/.config/niri/config.kdl:".into(), format!("Mod+Space {{ spawn \"{program}\"; }}")),
            Family::X11Wm => ("En la configuración de tu gestor de ventanas (en i3, ~/.config/i3/config):".into(), format!("bindsym $mod+space exec {program}")),
            Family::Kde => ("Configuración del sistema → Teclado → Atajos → Añadir nuevo → Orden, con esta orden:".into(), program.into()),
            Family::Gnome => ("Configuración → Teclado → Ver y personalizar atajos → Atajos personalizados, con esta orden:".into(), program.into()),
            Family::Cosmic => ("Configuración → Entrada → Teclado → Atajos de teclado → Atajos personalizados, con esta orden:".into(), program.into()),
            Family::Xfce => ("Configuración → Teclado → Atajos de aplicaciones → Añadir, con esta orden:".into(), program.into()),
            _ => ("En los atajos de teclado de tu escritorio, con esta orden:".into(), program.into()),
        }
    }
}

/// Hyprland configurado en Lua (`hyprland.lua`) en vez de `hyprland.conf`.
pub fn hypr_lua() -> bool {
    let config = env("XDG_CONFIG_HOME").map(std::path::PathBuf::from).unwrap_or_else(|| home_dir().join(".config"));
    config.join("hypr/hyprland.lua").is_file()
}

/// La distro, leída de `/etc/os-release`. Se detecta una vez por proceso.
pub fn distro() -> &'static Distro {
    static CACHE: OnceLock<Distro> = OnceLock::new();
    CACHE.get_or_init(|| {
        let text = read("/etc/os-release").unwrap_or_default();
        let field = |key: &str| {
            text.lines().find_map(|l| l.strip_prefix(key).and_then(|v| v.strip_prefix('='))).map(|v| v.trim_matches('"').to_string())
        };
        let id = field("ID").unwrap_or_else(|| "arch".into());
        let name = field("PRETTY_NAME").or_else(|| field("NAME")).unwrap_or_else(|| "Arch Linux".into());
        Distro { systemd: id != "artix" && has_tool("systemctl"), id, name }
    })
}

impl Distro {
    pub fn is_pure_arch(&self) -> bool {
        self.id == "arch"
    }

    /// Lo que conviene saber de esta distro en concreto, si no es Arch puro.
    pub fn note(&self) -> Option<&'static str> {
        Some(match self.id.as_str() {
            "arch" => return None,
            "cachyos" => "CachyOS es Arch con paquetes compilados para tu procesador y kernels propios (linux-cachyos). Casi todo lo de la Arch Wiki aplica igual; sus repositorios van por delante de los de Arch en unas pocas piezas.",
            "endeavouros" => "EndeavourOS es Arch con un instalador gráfico y unas pocas herramientas propias (eos-update, yay de serie). Usa los repositorios oficiales de Arch, así que la Arch Wiki aplica tal cual.",
            "manjaro" => "Manjaro **no** usa los repositorios de Arch: tiene los suyos, que llegan unas semanas tarde. Por eso mezclar paquetes del AUR puede fallar más que en Arch. Su herramienta propia es pamac.",
            "garuda" => "Garuda es Arch con el repositorio chaotic-aur activado y snapshots de Btrfs antes de cada actualización (puedes volver atrás desde el menú de arranque).",
            "artix" => "Artix es Arch **sin systemd**: usa OpenRC, runit, s6 o dinit. Los comandos con systemctl que veas en la Arch Wiki no te sirven tal cual.",
            "archcraft" | "rebornos" | "archlabs" | "parabola" => "Tu distro está basada en Arch: pacman y la Arch Wiki te sirven, pero puede traer repositorios y herramientas propias.",
            _ => return None,
        })
    }
}

/// El ayudante del AUR instalado, en orden de preferencia.
pub fn aur_helper() -> Option<&'static str> {
    ["paru", "yay", "pikaur", "aura", "trizen", "pamac"].into_iter().find(|t| has_tool(t))
}

/// ¿Hay un proceso con ese nombre? Lee `/proc/*/comm`.
pub fn process_running(name: &str) -> bool {
    let Ok(dir) = fs::read_dir("/proc") else { return false };
    dir.flatten()
        .filter(|e| e.file_name().to_string_lossy().bytes().all(|b| b.is_ascii_digit()))
        .any(|e| read(&e.path().join("comm").to_string_lossy()).is_some_and(|c| c.trim() == name))
}

// ── Herramientas por tarea ───────────────────────────────────────────────────

/// Un comando concreto para una tarea de escritorio.
pub struct Tool {
    pub program: &'static str,
    pub command: String,
}

/// Cuando no hay herramienta: qué paquete instalar y por qué ese.
pub struct Missing {
    pub package: &'static str,
    pub why: &'static str,
}

/// Carpeta de imágenes del usuario (`XDG_PICTURES_DIR`), o la casa si no hay.
pub fn pictures_dir() -> String {
    let home = home_dir();
    let configured = read(&home.join(".config/user-dirs.dirs").to_string_lossy()).and_then(|text| {
        text.lines()
            .find_map(|l| l.strip_prefix("XDG_PICTURES_DIR="))
            .map(|v| v.trim_matches('"').replace("$HOME", &home.to_string_lossy()))
    });
    configured
        .filter(|p| std::path::Path::new(p).is_dir())
        .or_else(|| Some(home.join("Pictures")).filter(|p| p.is_dir()).map(|p| p.to_string_lossy().to_string()))
        .unwrap_or_else(|| home.to_string_lossy().to_string())
}

fn screenshot_file() -> String {
    let stamp = crate::exec::capture(&["date", "+%Y%m%d-%H%M%S"], std::time::Duration::from_secs(2)).unwrap_or_else(|| "nueva".into());
    format!("{}/captura-{stamp}.png", pictures_dir())
}

/// La captura según el escritorio. `region`: el usuario elige un recuadro.
pub fn screenshot(region: bool) -> Result<Tool, Missing> {
    let d = desktop();
    let file = screenshot_file();
    let dir = pictures_dir();
    let mut order: Vec<&str> = match d.family {
        Family::Hyprland => vec!["hyprshot", "grimblast", "grim"],
        Family::Niri => vec!["niri", "grim"],
        Family::Sway | Family::Wlroots => vec!["grimblast", "grim"],
        Family::Kde => vec!["spectacle"],
        Family::Gnome => vec!["gnome-screenshot"],
        Family::Cosmic => vec!["cosmic-screenshot"],
        Family::Xfce => vec!["xfce4-screenshooter"],
        _ => vec![],
    };
    order.extend(if d.wayland { ["grim", "flameshot", "spectacle"] } else { ["flameshot", "maim", "scrot"] });
    for program in order {
        // grim solo recorta con slurp; sin él, solo vale la pantalla entera.
        if !has_tool(program) || (program == "grim" && region && !has_tool("slurp")) {
            continue;
        }
        let command = match (program, region) {
            ("hyprshot", true) => format!("hyprshot -m region -o {dir}"),
            ("hyprshot", false) => format!("hyprshot -m active -m output -o {dir}"),
            ("grimblast", true) => format!("grimblast --notify save area {file}"),
            ("grimblast", false) => format!("grimblast --notify save output {file}"),
            ("grim", true) => format!("grim -g \"$(slurp)\" {file}"),
            ("grim", false) => format!("grim {file}"),
            ("niri", true) => "niri msg action screenshot".into(),
            ("niri", false) => "niri msg action screenshot-screen".into(),
            ("spectacle", true) => format!("spectacle -b -n -r -o {file}"),
            ("spectacle", false) => format!("spectacle -b -n -f -o {file}"),
            ("gnome-screenshot", true) => format!("gnome-screenshot -a -f {file}"),
            ("gnome-screenshot", false) => format!("gnome-screenshot -f {file}"),
            ("cosmic-screenshot", _) => format!("cosmic-screenshot --save-dir {dir}"),
            ("xfce4-screenshooter", true) => format!("xfce4-screenshooter -r -s {file}"),
            ("xfce4-screenshooter", false) => format!("xfce4-screenshooter -f -s {file}"),
            ("flameshot", true) => format!("flameshot gui -p {dir}"),
            ("flameshot", false) => format!("flameshot full -p {dir}"),
            ("maim", true) => format!("maim -s {file}"),
            ("maim", false) => format!("maim {file}"),
            ("scrot", true) => format!("scrot -s {file}"),
            ("scrot", false) => format!("scrot {file}"),
            _ => continue,
        };
        return Ok(Tool { program, command });
    }
    Err(match d.family {
        Family::Kde => Missing { package: "spectacle", why: "Es la herramienta de capturas de KDE." },
        Family::Gnome => Missing { package: "gnome-screenshot", why: "Hace capturas desde la terminal en GNOME." },
        Family::Xfce => Missing { package: "xfce4-screenshooter", why: "Es la herramienta de capturas de Xfce." },
        _ if d.wayland => Missing { package: "grim slurp", why: "grim captura la pantalla en Wayland y slurp te deja elegir un recuadro con el ratón." },
        _ => Missing { package: "maim", why: "Captura la pantalla en X11, entera o un recuadro." },
    })
}

/// Bloquear la sesión según el escritorio.
pub fn lock() -> Result<Tool, Missing> {
    let d = desktop();
    let candidates: &[&str] = match d.family {
        Family::Hyprland => &["hyprlock", "swaylock", "gtklock"],
        Family::Sway | Family::Niri | Family::Wlroots => &["swaylock", "gtklock", "hyprlock"],
        Family::Xfce => &["xflock4"],
        Family::X11Wm | Family::Unknown if !d.wayland => &["i3lock", "xsecurelock", "slock", "betterlockscreen"],
        _ => &[],
    };
    if let Some(program) = candidates.iter().copied().find(|t| has_tool(t)) {
        let command = match program {
            "swaylock" => "swaylock -f".to_string(),
            "i3lock" => "i3lock -c 000000".to_string(),
            "betterlockscreen" => "betterlockscreen -l".to_string(),
            other => other.to_string(),
        };
        return Ok(Tool { program, command });
    }
    // loginctl avisa a quien escuche la señal de bloqueo: el propio escritorio
    // en KDE, GNOME, COSMIC y compañía; hypridle o swayidle en los compositores.
    let listener = match d.family {
        Family::Hyprland => process_running("hypridle"),
        Family::Sway | Family::Niri | Family::Wlroots => process_running("swayidle"),
        Family::X11Wm | Family::Unknown => process_running("xss-lock"),
        _ => true,
    };
    if listener && has_tool("loginctl") {
        return Ok(Tool { program: "loginctl", command: "loginctl lock-session".into() });
    }
    Err(match d.family {
        Family::Hyprland => Missing { package: "hyprlock", why: "Es la pantalla de bloqueo de Hyprland." },
        Family::Sway | Family::Niri | Family::Wlroots => Missing { package: "swaylock", why: "Es la pantalla de bloqueo más usada en compositores Wayland." },
        _ => Missing { package: "i3lock", why: "Una pantalla de bloqueo sencilla para X11." },
    })
}

/// Pantalla con luz de fondo regulable: (actual, máximo).
pub fn backlight() -> Option<(u32, u32)> {
    for entry in fs::read_dir("/sys/class/backlight").ok()?.flatten() {
        let base = entry.path();
        let value = |f: &str| read(&base.join(f).to_string_lossy()).and_then(|v| v.trim().parse::<u32>().ok());
        if let (Some(now), Some(max)) = (value("brightness"), value("max_brightness")) {
            if max > 0 {
                return Some((now, max));
            }
        }
    }
    None
}

/// Cambiar el brillo. `step`: +10, -10; `exact`: poner un porcentaje.
pub fn brightness(step: i32, exact: Option<u32>) -> Result<Tool, Missing> {
    if let Some(program) = ["brightnessctl", "light"].into_iter().find(|t| has_tool(t)) {
        let command = match (program, exact) {
            ("brightnessctl", Some(p)) => format!("brightnessctl set {p}%"),
            ("brightnessctl", None) if step >= 0 => format!("brightnessctl set {step}%+"),
            ("brightnessctl", None) => format!("brightnessctl set {}%-", -step),
            (_, Some(p)) => format!("light -S {p}"),
            (_, None) if step >= 0 => format!("light -A {step}"),
            (_, None) => format!("light -U {}", -step),
        };
        return Ok(Tool { program, command });
    }
    Err(Missing { package: "brightnessctl", why: "Cambia el brillo sin necesidad de sudo y funciona en cualquier escritorio." })
}

/// Volumen de la salida por defecto. `step` en puntos; `exact` en porcentaje.
pub fn volume(step: i32, exact: Option<u32>) -> Result<Tool, Missing> {
    if has_tool("wpctl") {
        let command = match exact {
            Some(p) => format!("wpctl set-volume @DEFAULT_AUDIO_SINK@ {p}%"),
            // -l 1.0: subir nunca pasa del 100 %, que distorsiona.
            None if step >= 0 => format!("wpctl set-volume -l 1.0 @DEFAULT_AUDIO_SINK@ {step}%+"),
            None => format!("wpctl set-volume @DEFAULT_AUDIO_SINK@ {}%-", -step),
        };
        return Ok(Tool { program: "wpctl", command });
    }
    if has_tool("pactl") {
        let command = match exact {
            Some(p) => format!("pactl set-sink-volume @DEFAULT_SINK@ {p}%"),
            None if step >= 0 => format!("pactl set-sink-volume @DEFAULT_SINK@ +{step}%"),
            None => format!("pactl set-sink-volume @DEFAULT_SINK@ -{}%", -step),
        };
        return Ok(Tool { program: "pactl", command });
    }
    Err(Missing { package: "pipewire-pulse wireplumber", why: "PipeWire es el servidor de sonido actual de Arch; wireplumber trae wpctl para controlarlo." })
}

/// Silenciar (`true`) o quitar el silencio (`false`). Con el estado exacto en
/// vez de alternar: pedir "quita el silencio" nunca silencia.
pub fn mute(on: bool) -> Result<Tool, Missing> {
    let value = if on { 1 } else { 0 };
    if has_tool("wpctl") {
        return Ok(Tool { program: "wpctl", command: format!("wpctl set-mute @DEFAULT_AUDIO_SINK@ {value}") });
    }
    if has_tool("pactl") {
        return Ok(Tool { program: "pactl", command: format!("pactl set-sink-mute @DEFAULT_SINK@ {value}") });
    }
    Err(Missing { package: "pipewire-pulse wireplumber", why: "PipeWire es el servidor de sonido actual de Arch; wireplumber trae wpctl para controlarlo." })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn las_tareas_dan_comando_o_paquete() {
        for result in [screenshot(true), screenshot(false), lock(), brightness(10, None), volume(-5, None), mute(true)] {
            match result {
                Ok(tool) => assert!(tool.command.starts_with(tool.program), "{}", tool.command),
                Err(missing) => assert!(!missing.package.is_empty()),
            }
        }
    }
}
