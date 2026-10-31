//! Los colores del escritorio, para que la isla se vista igual que él.
//!
//! Se buscan, por orden de fiabilidad, las paletas que generan los temas
//! dinámicos (matugen para Quickshell, Noctalia o Caelestia; pywal y wallust),
//! los colores de KDE, el GTK que generan esos mismos temas y el acento de
//! GNOME. Entre las paletas generadas gana la más reciente: es la que está
//! usando tu barra ahora. Si no hay nada, queda el azul de Arch.
//!
//! Solo lee archivos y `gsettings`.

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use serde::Serialize;
use serde_json::Value;

use crate::exec::capture;
use crate::memory::home_dir;

/// Colores en `#rrggbb`. La interfaz deriva de aquí transparencias y bordes.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Palette {
    /// De dónde salió: "matugen (Quickshell)", "KDE", "Arch"…
    pub source: String,
    pub dark: bool,
    /// Fondo de la isla.
    pub bg: String,
    /// Tarjetas y cajas sobre el fondo.
    pub surface: String,
    /// Tarjeta resaltada, botón al pasar el ratón.
    pub surface_high: String,
    pub text: String,
    pub dim: String,
    pub line: String,
    pub accent: String,
    pub on_accent: String,
    /// Segundo color del tema, para detalles.
    pub accent2: String,
    pub ok: String,
    pub warn: String,
    pub danger: String,
}

fn config_dir() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|| home_dir().join(".config"))
}

fn state_dir() -> PathBuf {
    std::env::var_os("XDG_STATE_HOME").map(PathBuf::from).unwrap_or_else(|| home_dir().join(".local/state"))
}

fn cache_dir() -> PathBuf {
    std::env::var_os("XDG_CACHE_HOME").map(PathBuf::from).unwrap_or_else(|| home_dir().join(".cache"))
}

/// Los archivos de los que puede salir la paleta. La isla los vigila: si
/// cambia uno (otro fondo de pantalla), vuelve a pedir los colores.
pub fn watched_files() -> Vec<PathBuf> {
    vec![
        state_dir().join("quickshell/user/generated/colors.json"),
        config_dir().join("noctalia/colors.json"),
        state_dir().join("caelestia/scheme.json"),
        cache_dir().join("wal/colors.json"),
        config_dir().join("kdeglobals"),
        config_dir().join("gtk-4.0/gtk.css"),
        config_dir().join("gtk-3.0/gtk.css"),
    ]
}

/// Huella de los archivos vigilados: cambia cuando cambia alguno.
pub fn fingerprint() -> Vec<Option<SystemTime>> {
    watched_files().iter().map(|p| fs::metadata(p).and_then(|m| m.modified()).ok()).collect()
}

// ── Color ────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
struct Rgb(f32, f32, f32);

impl Rgb {
    fn parse(text: &str) -> Option<Rgb> {
        let t = text.trim().trim_matches('"').trim_start_matches('#');
        if t.len() >= 6 && t.is_char_boundary(6) && t[..6].chars().all(|c| c.is_ascii_hexdigit()) {
            let n = u32::from_str_radix(&t[..6], 16).ok()?;
            return Some(Rgb(((n >> 16) & 255) as f32, ((n >> 8) & 255) as f32, (n & 255) as f32));
        }
        // "r,g,b" de kdeglobals
        let parts: Vec<f32> = t.split(',').filter_map(|p| p.trim().parse().ok()).collect();
        (parts.len() >= 3).then(|| Rgb(parts[0], parts[1], parts[2]))
    }

    fn hex(self) -> String {
        let c = |v: f32| v.round().clamp(0.0, 255.0) as u8;
        format!("#{:02x}{:02x}{:02x}", c(self.0), c(self.1), c(self.2))
    }

    fn luminance(self) -> f32 {
        let lin = |v: f32| {
            let v = v / 255.0;
            if v <= 0.03928 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * lin(self.0) + 0.7152 * lin(self.1) + 0.0722 * lin(self.2)
    }

    fn contrast(self, other: Rgb) -> f32 {
        let (a, b) = (self.luminance(), other.luminance());
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    fn mix(self, other: Rgb, t: f32) -> Rgb {
        Rgb(self.0 + (other.0 - self.0) * t, self.1 + (other.1 - self.1) * t, self.2 + (other.2 - self.2) * t)
    }

    fn saturation(self) -> f32 {
        let max = self.0.max(self.1).max(self.2);
        let min = self.0.min(self.1).min(self.2);
        if max == 0.0 {
            0.0
        } else {
            (max - min) / max
        }
    }
}

const WHITE: Rgb = Rgb(255.0, 255.0, 255.0);
const BLACK: Rgb = Rgb(0.0, 0.0, 0.0);

/// Texto legible sobre `bg`: blanco o casi negro, el que más contraste dé.
fn readable_on(bg: Rgb) -> Rgb {
    let ink = Rgb(18.0, 20.0, 26.0);
    if bg.contrast(WHITE) >= bg.contrast(ink) {
        WHITE
    } else {
        ink
    }
}

/// Lo mínimo que hace falta para armar la paleta: fondo, texto y acento.
struct Base {
    source: String,
    bg: Rgb,
    text: Rgb,
    accent: Rgb,
    accent2: Option<Rgb>,
    surface: Option<Rgb>,
    surface_high: Option<Rgb>,
    dim: Option<Rgb>,
    line: Option<Rgb>,
    on_accent: Option<Rgb>,
    danger: Option<Rgb>,
}

impl Base {
    fn new(source: &str, bg: Rgb, text: Rgb, accent: Rgb) -> Base {
        Base { source: source.into(), bg, text, accent, accent2: None, surface: None, surface_high: None, dim: None, line: None, on_accent: None, danger: None }
    }

    fn finish(self) -> Palette {
        let dark = self.bg.luminance() < 0.2;
        // El texto tiene que leerse sobre el fondo aunque el tema diga otra cosa.
        let text = if self.text.contrast(self.bg) < 4.5 { readable_on(self.bg) } else { self.text };
        let lift = if dark { WHITE } else { BLACK };
        let surface = self.surface.unwrap_or_else(|| self.bg.mix(lift, 0.06));
        let surface_high = self.surface_high.unwrap_or_else(|| self.bg.mix(lift, 0.12));
        // Un acento casi gris (temas monocromos) no sirve para marcar nada.
        let accent = if self.accent.saturation() < 0.08 && self.accent.contrast(self.bg) < 2.0 { Rgb(23.0, 147.0, 209.0) } else { self.accent };
        // Verde, ámbar y rojo de estado, ajustados a fondo claro u oscuro.
        let (ok, warn, danger) = if dark {
            (Rgb(126.0, 214.0, 145.0), Rgb(242.0, 196.0, 92.0), Rgb(255.0, 128.0, 140.0))
        } else {
            (Rgb(38.0, 128.0, 62.0), Rgb(166.0, 110.0, 0.0), Rgb(186.0, 26.0, 26.0))
        };
        Palette {
            source: self.source,
            dark,
            bg: self.bg.hex(),
            surface: surface.hex(),
            surface_high: surface_high.hex(),
            text: text.hex(),
            dim: self.dim.filter(|d| d.contrast(self.bg) >= 3.0).unwrap_or_else(|| text.mix(self.bg, 0.35)).hex(),
            line: self.line.unwrap_or_else(|| text.mix(self.bg, 0.82)).hex(),
            accent: accent.hex(),
            on_accent: self.on_accent.filter(|c| c.contrast(accent) >= 3.0).unwrap_or_else(|| readable_on(accent)).hex(),
            accent2: self.accent2.unwrap_or(accent).hex(),
            ok: ok.hex(),
            warn: warn.hex(),
            danger: self.danger.filter(|d| d.contrast(self.bg) >= 3.0).unwrap_or(danger).hex(),
        }
    }
}

// ── Fuentes ──────────────────────────────────────────────────────────────────

fn read_json(path: &PathBuf) -> Option<Value> {
    serde_json::from_str(&fs::read_to_string(path).ok()?).ok()
}

/// Paleta Material You con los nombres de matugen (`surface`, `primary`…).
fn material(source: &str, colors: &Value) -> Option<Base> {
    let get = |key: &str| colors.get(key).and_then(Value::as_str).and_then(Rgb::parse);
    let mut base = Base::new(source, get("surface_container_lowest").or_else(|| get("background")).or_else(|| get("surface"))?, get("on_surface")?, get("primary")?);
    base.surface = get("surface_container");
    base.surface_high = get("surface_container_highest").or_else(|| get("surface_container_high"));
    base.dim = get("on_surface_variant");
    base.line = get("outline_variant");
    base.on_accent = get("on_primary");
    base.accent2 = get("tertiary").or_else(|| get("secondary"));
    base.danger = get("error");
    Some(base)
}

fn quickshell() -> Option<Base> {
    material("Material You (Quickshell)", &read_json(&state_dir().join("quickshell/user/generated/colors.json"))?)
}

fn caelestia() -> Option<Base> {
    let json = read_json(&state_dir().join("caelestia/scheme.json"))?;
    material("Material You (Caelestia)", json.get("colours")?)
}

fn noctalia() -> Option<Base> {
    let json = read_json(&config_dir().join("noctalia/colors.json"))?;
    let get = |key: &str| json.get(key).and_then(Value::as_str).and_then(Rgb::parse);
    let mut base = Base::new("Noctalia", get("mSurface")?, get("mOnSurface")?, get("mPrimary")?);
    base.surface = get("mSurfaceVariant");
    base.dim = get("mOnSurfaceVariant");
    base.line = get("mOutline");
    base.on_accent = get("mOnPrimary");
    base.accent2 = get("mTertiary").or_else(|| get("mSecondary"));
    base.danger = get("mError");
    Some(base)
}

fn pywal() -> Option<Base> {
    let json = read_json(&cache_dir().join("wal/colors.json"))?;
    let special = json.get("special")?;
    let colors = json.get("colors")?;
    let get = |v: &Value, key: &str| v.get(key).and_then(Value::as_str).and_then(Rgb::parse);
    let bg = get(special, "background")?;
    // El acento es el color más vivo de los seis que pywal saca del fondo.
    let accent = (1..=6)
        .filter_map(|i| get(colors, &format!("color{i}")))
        .filter(|c| c.contrast(bg) >= 2.5)
        .max_by(|a, b| a.saturation().total_cmp(&b.saturation()))?;
    let mut base = Base::new("pywal", bg, get(special, "foreground")?, accent);
    base.accent2 = get(colors, "color5");
    Some(base)
}

/// Un valor de un archivo .ini sencillo: `[sección]` y `clave=valor`.
fn ini(text: &str, section: &str, key: &str) -> Option<String> {
    let mut inside = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            inside = line == format!("[{section}]");
        } else if inside {
            if let Some(value) = line.strip_prefix(key).and_then(|r| r.trim_start().strip_prefix('=')) {
                return Some(value.trim().to_string());
            }
        }
    }
    None
}

fn kde() -> Option<Base> {
    if crate::desktop::desktop().family != crate::desktop::Family::Kde {
        return None;
    }
    let text = fs::read_to_string(config_dir().join("kdeglobals")).ok()?;
    let get = |section: &str, key: &str| ini(&text, section, key).and_then(|v| Rgb::parse(&v));
    let bg = get("Colors:Window", "BackgroundNormal")?;
    let accent = get("General", "AccentColor").or_else(|| get("Colors:Selection", "BackgroundNormal"))?;
    let mut base = Base::new("KDE", bg, get("Colors:Window", "ForegroundNormal")?, accent);
    base.surface = get("Colors:View", "BackgroundNormal");
    base.dim = get("Colors:Window", "ForegroundInactive");
    base.danger = get("Colors:View", "ForegroundNegative");
    Some(base)
}

fn prefers_dark() -> bool {
    capture(&["gsettings", "get", "org.gnome.desktop.interface", "color-scheme"], Duration::from_secs(2)).is_none_or(|v| v.contains("dark"))
}

/// `@define-color` del gtk.css que generan matugen y compañía. Si trae
/// variantes clara y oscura, se usa la que pide el sistema.
fn gtk_css() -> Option<Base> {
    let text = [config_dir().join("gtk-4.0/gtk.css"), config_dir().join("gtk-3.0/gtk.css")].iter().find_map(|p| fs::read_to_string(p).ok())?;
    let wanted = if prefers_dark() { "dark" } else { "light" };
    let section = text.split("@media").find(|s| s.trim_start().starts_with(&format!("(prefers-color-scheme: {wanted})"))).unwrap_or(&text);
    let get = |name: &str| {
        section.lines().find_map(|l| l.trim().strip_prefix("@define-color ")?.strip_prefix(name)?.strip_prefix(' ').map(|v| v.trim_end_matches(';').trim().to_string())).and_then(|v| Rgb::parse(&v))
    };
    let mut base = Base::new("GTK", get("window_bg_color")?, get("window_fg_color")?, get("accent_bg_color").or_else(|| get("accent_color"))?);
    base.surface = get("card_bg_color");
    base.on_accent = get("accent_fg_color");
    base.danger = get("destructive_color");
    Some(base)
}

fn gnome() -> Option<Base> {
    let accent = capture(&["gsettings", "get", "org.gnome.desktop.interface", "accent-color"], Duration::from_secs(2))?;
    // Los acentos con nombre de GNOME 47 y su color.
    let hex = match accent.trim_matches('\'') {
        "blue" => "#3584e4",
        "teal" => "#2190a4",
        "green" => "#3a944a",
        "yellow" => "#c88800",
        "orange" => "#ed5b00",
        "red" => "#e62d42",
        "pink" => "#d56199",
        "purple" => "#9141ac",
        "slate" => "#6f8396",
        _ => return None,
    };
    let (bg, text) = if prefers_dark() { (Rgb(30.0, 30.0, 32.0), Rgb(240.0, 240.0, 242.0)) } else { (Rgb(250.0, 250.0, 251.0), Rgb(30.0, 30.0, 34.0)) };
    Some(Base::new("GNOME", bg, text, Rgb::parse(hex)?))
}

/// Archito Noche: el tema propio, cuando el escritorio no dice nada.
pub fn fallback() -> Palette {
    let mut base = Base::new("Archito", Rgb(13.0, 17.0, 27.0), Rgb(226.0, 233.0, 245.0), Rgb(23.0, 147.0, 209.0));
    base.accent2 = Some(Rgb(146.0, 120.0, 255.0));
    base.finish()
}

fn modified(path: PathBuf) -> SystemTime {
    fs::metadata(path).and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH)
}

/// Una fuente de colores: lee su archivo y arma la base, si existe.
type Source = fn() -> Option<Base>;

/// La paleta del escritorio actual.
pub fn detect() -> Palette {
    // Las paletas generadas por el fondo de pantalla: gana la más reciente.
    let mut generated: Vec<(SystemTime, Source)> = vec![
        (modified(state_dir().join("quickshell/user/generated/colors.json")), quickshell),
        (modified(config_dir().join("noctalia/colors.json")), noctalia),
        (modified(state_dir().join("caelestia/scheme.json")), caelestia),
        (modified(cache_dir().join("wal/colors.json")), pywal),
    ];
    generated.retain(|(t, _)| *t > SystemTime::UNIX_EPOCH);
    generated.sort_by_key(|(t, _)| std::cmp::Reverse(*t));
    generated
        .into_iter()
        .find_map(|(_, f)| f())
        .or_else(kde)
        .or_else(gtk_css)
        .or_else(gnome)
        .map(Base::finish)
        .unwrap_or_else(fallback)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lee_colores_de_varios_formatos() {
        assert_eq!(Rgb::parse("#1793d1"), Some(Rgb(23.0, 147.0, 209.0)));
        assert_eq!(Rgb::parse("1793d1ff"), Some(Rgb(23.0, 147.0, 209.0)));
        assert_eq!(Rgb::parse("23,147,209"), Some(Rgb(23.0, 147.0, 209.0)));
        assert_eq!(Rgb::parse("rojo"), None);
    }

    #[test]
    fn la_paleta_siempre_se_lee() {
        // Texto gris sobre gris: se corrige para que se lea.
        let p = Base::new("prueba", Rgb(40.0, 40.0, 40.0), Rgb(60.0, 60.0, 60.0), Rgb(23.0, 147.0, 209.0)).finish();
        assert!(p.dark);
        let (bg, text) = (Rgb::parse(&p.bg).unwrap(), Rgb::parse(&p.text).unwrap());
        assert!(text.contrast(bg) >= 4.5);
        let real = detect();
        assert!(Rgb::parse(&real.accent).is_some() && Rgb::parse(&real.bg).is_some(), "{real:?}");
    }

    #[test]
    fn material_you() {
        let json: Value = serde_json::from_str(r##"{"surface_container_lowest":"#100d0d","on_surface":"#e9e1df","primary":"#e0bfb9","on_primary":"#402b27","surface_container":"#221f1e","error":"#ffb4ab"}"##).unwrap();
        let p = material("m", &json).unwrap().finish();
        assert_eq!((p.bg.as_str(), p.accent.as_str(), p.on_accent.as_str()), ("#100d0d", "#e0bfb9", "#402b27"));
        assert!(p.dark);
    }
}
