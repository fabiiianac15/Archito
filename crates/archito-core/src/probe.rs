//! Sondas de solo lectura: leen `/proc`, `/sys` y herramientas del sistema.
//! Ninguna función de este archivo cambia nada en el equipo.

use std::fs;
use std::time::Duration;

use crate::exec::{capture, capture_status, has_tool};

const SHORT: Duration = Duration::from_secs(4);
const LONG: Duration = Duration::from_secs(25);

fn read(path: &str) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

pub fn kernel() -> Option<String> {
    read("/proc/sys/kernel/osrelease")
}

pub fn distro() -> Option<String> {
    let text = read("/etc/os-release")?;
    text.lines()
        .find_map(|l| l.strip_prefix("PRETTY_NAME="))
        .map(|v| v.trim_matches('"').to_string())
}

pub fn hostname() -> Option<String> {
    read("/proc/sys/kernel/hostname")
}

pub fn user() -> String {
    std::env::var("USER").unwrap_or_else(|_| "tú".into())
}

pub fn desktop() -> Option<String> {
    std::env::var("XDG_CURRENT_DESKTOP").ok().filter(|s| !s.is_empty())
}

pub fn shell() -> Option<String> {
    let sh = std::env::var("SHELL").ok()?;
    sh.rsplit('/').next().map(str::to_string)
}

pub fn uptime() -> Option<String> {
    let secs = read("/proc/uptime")?.split_whitespace().next()?.parse::<f64>().ok()? as u64;
    let (d, h, m) = (secs / 86400, secs % 86400 / 3600, secs % 3600 / 60);
    Some(match (d, h) {
        (0, 0) => format!("{m} min"),
        (0, _) => format!("{h} h {m} min"),
        _ => format!("{d} d {h} h"),
    })
}

pub fn cpu_model() -> Option<String> {
    let text = read("/proc/cpuinfo")?;
    text.lines()
        .find(|l| l.starts_with("model name"))
        .and_then(|l| l.split_once(':'))
        .map(|(_, v)| v.trim().to_string())
}

pub fn cpu_threads() -> usize {
    std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1)
}

fn cpu_times() -> Option<(u64, u64)> {
    let text = read("/proc/stat")?;
    let nums: Vec<u64> = text.lines().next()?.split_whitespace().skip(1).filter_map(|n| n.parse().ok()).collect();
    let idle = nums.get(3)? + nums.get(4).copied().unwrap_or(0);
    Some((idle, nums.iter().sum()))
}

/// Uso de CPU en porcentaje, medido durante un cuarto de segundo.
pub fn cpu_usage() -> Option<f32> {
    let (idle_a, total_a) = cpu_times()?;
    std::thread::sleep(Duration::from_millis(250));
    let (idle_b, total_b) = cpu_times()?;
    let total = total_b.checked_sub(total_a)? as f32;
    (total > 0.0).then(|| 100.0 * (1.0 - (idle_b - idle_a) as f32 / total))
}

pub struct Mem {
    pub total_gb: f32,
    pub used_gb: f32,
    pub pct: f32,
    pub swap_pct: Option<f32>,
}

pub fn memory() -> Option<Mem> {
    let text = read("/proc/meminfo")?;
    let kb = |key: &str| {
        text.lines()
            .find(|l| l.starts_with(key))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|v| v.parse::<f32>().ok())
    };
    let total = kb("MemTotal:")?;
    let avail = kb("MemAvailable:")?;
    let used = total - avail;
    let swap_pct = match (kb("SwapTotal:"), kb("SwapFree:")) {
        (Some(t), Some(f)) if t > 0.0 => Some(100.0 * (t - f) / t),
        _ => None,
    };
    Some(Mem { total_gb: total / 1048576.0, used_gb: used / 1048576.0, pct: 100.0 * used / total, swap_pct })
}

pub struct Disk {
    pub total_gb: f32,
    pub free_gb: f32,
    pub pct: f32,
}

pub fn disk(path: &str) -> Option<Disk> {
    let out = capture(&["df", "-B1", "--output=size,avail", path], SHORT)?;
    let mut nums = out.lines().nth(1)?.split_whitespace().filter_map(|n| n.parse::<f64>().ok());
    let (size, avail) = (nums.next()?, nums.next()?);
    let gb = 1024f64.powi(3);
    Some(Disk { total_gb: (size / gb) as f32, free_gb: (avail / gb) as f32, pct: (100.0 * (size - avail) / size) as f32 })
}

/// Sensor más caliente: (grados, nombre).
pub fn temperature() -> Option<(f32, String)> {
    let mut best: Option<(f32, String)> = None;
    for dir in fs::read_dir("/sys/class/hwmon").ok()?.flatten() {
        let base = dir.path();
        let name = read(&base.join("name").to_string_lossy()).unwrap_or_else(|| "sensor".into());
        for entry in fs::read_dir(&base).ok()?.flatten() {
            let file = entry.file_name().to_string_lossy().to_string();
            if !(file.starts_with("temp") && file.ends_with("_input")) {
                continue;
            }
            let Some(milli) = read(&entry.path().to_string_lossy()).and_then(|v| v.parse::<f32>().ok()) else { continue };
            let c = milli / 1000.0;
            if (1.0..130.0).contains(&c) && best.as_ref().is_none_or(|(b, _)| c > *b) {
                best = Some((c, name.clone()));
            }
        }
    }
    best
}

pub struct Battery {
    pub percent: f32,
    pub status: String,
}

pub fn battery() -> Option<Battery> {
    for entry in fs::read_dir("/sys/class/power_supply").ok()?.flatten() {
        let base = entry.path();
        if !entry.file_name().to_string_lossy().starts_with("BAT") {
            continue;
        }
        let percent = read(&base.join("capacity").to_string_lossy())?.parse().ok()?;
        let status = read(&base.join("status").to_string_lossy()).unwrap_or_default();
        return Some(Battery { percent, status });
    }
    None
}

pub struct Proc {
    pub pid: String,
    pub name: String,
    pub cpu: String,
    pub mem: String,
}

pub fn top_processes(by_mem: bool, limit: usize) -> Vec<Proc> {
    let sort = if by_mem { "--sort=-%mem" } else { "--sort=-%cpu" };
    let Some(out) = capture(&["ps", "-eo", "pid,comm,%cpu,%mem", sort], SHORT) else { return Vec::new() };
    out.lines()
        .skip(1)
        .filter_map(|l| {
            let f: Vec<&str> = l.split_whitespace().collect();
            (f.len() >= 4 && f[1] != "ps").then(|| Proc {
                pid: f[0].into(),
                name: f[1..f.len() - 2].join(" "),
                cpu: f[f.len() - 2].into(),
                mem: f[f.len() - 1].into(),
            })
        })
        .take(limit)
        .collect()
}

pub fn package_count() -> Option<usize> {
    capture(&["pacman", "-Qq"], SHORT).map(|o| o.lines().count())
}

pub fn is_installed(pkg: &str) -> bool {
    capture_status(&["pacman", "-Qq", pkg], SHORT).is_some_and(|(_, ok)| ok)
}

/// `None` si falta `checkupdates`; lista vacía si todo está al día.
pub fn updates() -> Option<Vec<String>> {
    if !has_tool("checkupdates") {
        return None;
    }
    let out = capture(&["checkupdates"], LONG).unwrap_or_default();
    Some(out.lines().filter_map(|l| l.split_whitespace().next()).map(str::to_string).collect())
}

pub fn orphans() -> Vec<String> {
    capture(&["pacman", "-Qtdq"], SHORT).map(|o| o.lines().map(str::to_string).collect()).unwrap_or_default()
}

/// (repositorio/nombre, descripción)
pub fn search_packages(term: &str, limit: usize) -> Option<Vec<(String, String)>> {
    let (out, _) = capture_status(&["pacman", "-Ss", "--", term], SHORT)?;
    let lines: Vec<&str> = out.lines().collect();
    let mut found: Vec<(String, String)> = Vec::new();
    for pair in lines.chunks(2) {
        let (Some(head), Some(desc)) = (pair.first(), pair.get(1)) else { continue };
        if head.starts_with(' ') {
            continue;
        }
        let name = head.split_whitespace().next().unwrap_or("").to_string();
        found.push((name, desc.trim().to_string()));
    }
    // Primero el paquete que se llama exactamente como lo que se buscó.
    found.sort_by_key(|(name, _)| name.rsplit('/').next() != Some(term));
    found.truncate(limit);
    Some(found)
}

pub fn package_exists(pkg: &str) -> bool {
    capture_status(&["pacman", "-Si", "--", pkg], SHORT).is_some_and(|(_, ok)| ok)
}

pub fn failed_services() -> Vec<String> {
    capture(&["systemctl", "--failed", "--no-legend", "--plain"], SHORT)
        .map(|o| o.lines().filter_map(|l| l.split_whitespace().next()).map(str::to_string).collect())
        .unwrap_or_default()
}

pub fn service_active(unit: &str, user: bool) -> bool {
    let mut argv = vec!["systemctl"];
    if user {
        argv.push("--user");
    }
    argv.extend(["is-active", "--quiet", unit]);
    capture_status(&argv, SHORT).is_some_and(|(_, ok)| ok)
}

pub fn recent_errors(limit: usize) -> Option<String> {
    let n = limit.to_string();
    capture(&["journalctl", "-p", "err", "-b", "-n", &n, "--no-pager", "-q"], SHORT)
}

pub fn local_ip() -> Option<String> {
    let out = capture(&["ip", "-4", "-o", "addr", "show", "scope", "global"], SHORT)?;
    let line = out.lines().next()?;
    let addr = line.split_whitespace().skip_while(|w| *w != "inet").nth(1)?;
    Some(addr.split('/').next()?.to_string())
}

/// Interfaces con su estado: (nombre, UP/DOWN).
pub fn interfaces() -> Vec<(String, String)> {
    let Some(out) = capture(&["ip", "-br", "link"], SHORT) else { return Vec::new() };
    out.lines()
        .filter_map(|l| {
            let mut f = l.split_whitespace();
            let (name, state) = (f.next()?, f.next()?);
            (name != "lo").then(|| (name.to_string(), state.to_string()))
        })
        .collect()
}

pub fn internet() -> bool {
    capture_status(&["ping", "-c", "1", "-W", "2", "1.1.1.1"], SHORT).is_some_and(|(_, ok)| ok)
}

pub fn dns_works() -> bool {
    capture_status(&["getent", "hosts", "archlinux.org"], SHORT).is_some_and(|(_, ok)| ok)
}

/// Volumen de la salida por defecto: (porcentaje, silenciado).
pub fn volume() -> Option<(u32, bool)> {
    let out = capture(&["wpctl", "get-volume", "@DEFAULT_AUDIO_SINK@"], SHORT)?;
    let value: f32 = out.split_whitespace().nth(1)?.parse().ok()?;
    Some(((value * 100.0).round() as u32, out.contains("MUTED")))
}

/// Descripción de una línea sacada de las páginas del manual instaladas.
pub fn whatis(command: &str) -> Option<String> {
    if !command.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '+')) {
        return None;
    }
    let (out, ok) = capture_status(&["whatis", "--", command], SHORT)?;
    if !ok {
        return None;
    }
    let line = out.lines().next()?;
    line.split_once(" - ").map(|(_, d)| d.trim().to_string())
}

/// Paquete que trae un programa que no está instalado.
pub fn package_providing(command: &str) -> Option<String> {
    if !command.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '+')) {
        return None;
    }
    // Solo vale un nombre de paquete bien formado de un programa que terminó bien.
    let first_package = |argv: &[&str]| {
        let (out, ok) = capture_status(argv, SHORT)?;
        let name = out.lines().next()?.trim().rsplit('/').next()?.to_string();
        let valid = !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '+' | '@'));
        (ok && valid).then_some(name)
    };
    let target = format!("/usr/bin/{command}");
    has_tool("pkgfile")
        .then(|| first_package(&["pkgfile", "-b", "--", command]))
        .flatten()
        .or_else(|| first_package(&["pacman", "-Fq", "--", &target]))
}

pub fn local_hour() -> u32 {
    capture(&["date", "+%H"], SHORT).and_then(|h| h.parse().ok()).unwrap_or(12)
}

pub fn summary() -> String {
    let mut lines = Vec::new();
    let mut push = |label: &str, value: Option<String>| {
        if let Some(v) = value {
            lines.push(format!("{label:<10} {v}"));
        }
    };
    push("Sistema", distro());
    push("Kernel", kernel());
    push("Equipo", hostname());
    push("Escritorio", desktop());
    push("Shell", shell());
    push("CPU", cpu_model().map(|m| format!("{m} ({} hilos)", cpu_threads())));
    push("RAM", memory().map(|m| format!("{:.1} de {:.0} GB en uso", m.used_gb, m.total_gb)));
    push("Disco /", disk("/").map(|d| format!("{:.0} GB libres de {:.0}", d.free_gb, d.total_gb)));
    push("Paquetes", package_count().map(|n| n.to_string()));
    push("Encendido", uptime());
    lines.join("\n")
}
