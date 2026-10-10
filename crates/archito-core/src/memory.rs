//! Memoria local: preferencias, progreso y bitácora. Todo vive en
//! `~/.config/archito/` y nunca sale del equipo.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn home() -> PathBuf {
    PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
}

fn xdg(var: &str, fallback: &str) -> PathBuf {
    std::env::var_os(var).map(PathBuf::from).filter(|p| p.is_absolute()).unwrap_or_else(|| home().join(fallback))
}

/// `~/.config/archito`, o `ARCHITO_CONFIG_DIR` (pruebas e instalaciones portables).
pub fn config_dir() -> PathBuf {
    std::env::var_os("ARCHITO_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| xdg("XDG_CONFIG_HOME", ".config").join("archito"))
}

/// `$XDG_RUNTIME_DIR/archito`: lo que deja el hook de la shell. Vive en
/// memoria y se borra al cerrar la sesión.
pub fn runtime_dir() -> PathBuf {
    std::env::var_os("ARCHITO_RUNTIME_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("XDG_RUNTIME_DIR").map(|d| PathBuf::from(d).join("archito")))
        .unwrap_or_else(|| config_dir().join("run"))
}

pub fn home_dir() -> PathBuf {
    home()
}

pub fn data_dir() -> PathBuf {
    xdg("XDG_DATA_HOME", ".local/share")
}

/// Punto de partida de una misión: solo cuenta lo que pase después.
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct Started {
    pub at: u64,
    /// Tamaño de cada archivo vigilado al empezar (historial, pacman.log).
    pub offsets: BTreeMap<String, u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Memory {
    pub user_name: Option<String>,
    /// Modo profesor: desarma cada comando antes de ejecutarlo.
    pub teach: bool,
    /// Programas que el usuario ya vio con Archito, con las veces.
    pub seen: BTreeMap<String, u32>,
    pub missions_done: Vec<String>,
    pub missions_started: BTreeMap<String, Started>,
    #[serde(skip)]
    path: Option<PathBuf>,
}

impl Default for Memory {
    fn default() -> Self {
        Memory { user_name: None, teach: true, seen: BTreeMap::new(), missions_done: Vec::new(), missions_started: BTreeMap::new(), path: None }
    }
}

impl Memory {
    pub fn load() -> Self {
        let path = config_dir().join("estado.json");
        let mut mem: Memory = fs::read_to_string(&path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
        mem.path = Some(path);
        mem
    }

    /// Memoria que no toca el disco, para pruebas.
    pub fn ephemeral() -> Self {
        Memory::default()
    }

    pub fn save(&self) {
        let Some(path) = &self.path else { return };
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Ok(text) = serde_json::to_string_pretty(self) {
            // Escribir aparte y renombrar: un corte de luz no deja el archivo a medias.
            let tmp = path.with_extension("json.tmp");
            if fs::write(&tmp, text).is_ok() {
                let _ = fs::rename(&tmp, path);
            }
        }
    }

    /// Apunta un programa en la bitácora. Devuelve `true` si es nuevo.
    pub fn see(&mut self, program: &str) -> bool {
        let count = self.seen.entry(program.to_string()).or_insert(0);
        *count += 1;
        *count == 1
    }
}

/// Deja constancia de una acción confirmada o cancelada. No guarda el chat.
pub fn log_activity(command: &str, outcome: &str) {
    let dir = config_dir();
    let _ = fs::create_dir_all(&dir);
    let line = serde_json::json!({ "at": now(), "command": command, "outcome": outcome }).to_string();
    use std::io::Write;
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(dir.join("actividad.jsonl")) {
        let _ = writeln!(f, "{line}");
    }
}

/// Últimas acciones registradas: (segundos desde 1970, comando, resultado).
pub fn recent_activity(limit: usize) -> Vec<(u64, String, String)> {
    let Ok(text) = fs::read_to_string(config_dir().join("actividad.jsonl")) else { return Vec::new() };
    let mut rows: Vec<(u64, String, String)> = text
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .map(|v| (v["at"].as_u64().unwrap_or(0), v["command"].as_str().unwrap_or("").to_string(), v["outcome"].as_str().unwrap_or("").to_string()))
        .collect();
    let skip = rows.len().saturating_sub(limit);
    rows.drain(..skip);
    rows
}
