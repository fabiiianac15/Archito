//! El camino: misiones cortas que el usuario hace en SU terminal y que
//! Archito comprueba mirando el sistema real, no preguntando "¿ya lo hiciste?".
//!
//! Qué se mira para comprobar:
//!   * los comandos recientes que apunta el hook de la shell (si está puesto);
//!   * lo que se añadió al historial de fish, zsh o bash desde que empezó la misión;
//!   * lo que se añadió a `/var/log/pacman.log` desde entonces.
//! Solo se busca el comando de la misión. Nada de esto sale del equipo.

use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

use serde::Serialize;

use crate::memory::{data_dir, home_dir, now, runtime_dir, Memory, Started};

const PACMAN_LOG: &str = "/var/log/pacman.log";

enum Check {
    /// El usuario ejecutó un comando que empieza por alguno de estos textos.
    Ran(&'static [&'static str]),
    /// Apareció este texto en el registro de pacman.
    PacmanLog(&'static str),
}

pub struct Mission {
    pub id: &'static str,
    pub stage: usize,
    pub title: &'static str,
    /// Por qué vale la pena aprender esto.
    pub why: &'static str,
    /// Qué tiene que hacer, paso a paso.
    pub steps: &'static [&'static str],
    /// El comando central de la misión, para el botón "Explícame".
    pub command: &'static str,
    /// Lo que Archito dice cuando la misión queda hecha.
    pub done_text: &'static str,
    pub xp: u32,
    check: Check,
}

pub const STAGES: &[&str] = &["Primer contacto", "Paquetes sin miedo", "Bajo el capó", "Tu sistema, tus reglas"];

pub const MISSIONS: &[Mission] = &[
    Mission {
        id: "kernel",
        stage: 0,
        title: "Saluda a tu kernel",
        why: "El kernel es el programa que habla con tu hardware. Saber qué versión corres es lo primero que te preguntarán si pides ayuda en un foro.",
        steps: &["Abre una terminal (en Hyprland suele ser SUPER + Enter o SUPER + Q).", "Escribe `uname -r` y pulsa Enter.", "Vuelve aquí y pulsa Comprobar."],
        command: "uname -r",
        done_text: "Ya ejecutaste tu primer comando. Lo que viste es la versión del kernel de Linux que mueve tu equipo ahora mismo.",
        xp: 10,
        check: Check::Ran(&["uname"]),
    },
    Mission {
        id: "explorar",
        stage: 0,
        title: "Mira dónde estás",
        why: "La terminal siempre está \"parada\" en una carpeta. Casi todos los errores de principiante vienen de no saber en cuál.",
        steps: &["Escribe `pwd` para ver en qué carpeta estás.", "Escribe `ls -la` para ver todo lo que hay, incluidos los archivos ocultos.", "Entra a una carpeta con `cd nombre` y vuelve con `cd ..`."],
        command: "ls -la",
        done_text: "Ya sabes ubicarte. Los archivos que empiezan por punto son los ocultos: ahí viven casi todas tus configuraciones.",
        xp: 10,
        check: Check::Ran(&["ls -"]),
    },
    Mission {
        id: "manual",
        stage: 0,
        title: "Abre el manual",
        why: "Arch viene con el manual de cada comando instalado. Quien sabe leer `man` no depende de buscar en Internet.",
        steps: &["Escribe `man pacman`.", "Muévete con las flechas o con espacio. Busca una palabra con `/palabra` y Enter.", "Sal con la tecla `q`."],
        command: "man pacman",
        done_text: "Eso que abriste está en tu disco y funciona sin Internet. Cada vez que yo te explique un comando, el detalle completo está ahí.",
        xp: 15,
        check: Check::Ran(&["man "]),
    },
    Mission {
        id: "buscar",
        stage: 1,
        title: "Busca un programa",
        why: "En Arch los programas se instalan desde los repositorios, que son más seguros que descargar instaladores de páginas web.",
        steps: &["Piensa en algo que quieras: un reproductor, un editor, un juego.", "Búscalo con `pacman -Ss palabra`, por ejemplo `pacman -Ss reproductor`.", "Fíjate en el nombre que aparece después de la barra: ese es el que se instala."],
        command: "pacman -Ss reproductor",
        done_text: "Buscar no necesita sudo porque solo lee. La parte antes de la barra (core, extra) es el repositorio de donde viene el paquete.",
        xp: 15,
        check: Check::Ran(&["pacman -Ss", "yay -Ss", "paru -Ss"]),
    },
    Mission {
        id: "instalar",
        stage: 1,
        title: "Instala tu primer paquete",
        why: "Instalar con pacman es la operación que más vas a repetir. Una vez la haces a mano, deja de dar miedo.",
        steps: &["Elige un paquete pequeño. Sugerencia: `sl`, un tren que cruza tu terminal.", "Instálalo con `sudo pacman -S sl`.", "Lee lo que pacman te muestra antes de aceptar: qué instala y cuánto ocupa.", "Pruébalo escribiendo `sl`."],
        command: "sudo pacman -S sl",
        done_text: "Vi en el registro de pacman que instalaste un paquete. Con sudo pediste permiso de administrador porque instalar cambia archivos del sistema.",
        xp: 25,
        check: Check::PacmanLog("[ALPM] installed "),
    },
    Mission {
        id: "ficha",
        stage: 1,
        title: "Lee la ficha de un paquete",
        why: "Antes de quitar algo conviene saber qué es, cuánto ocupa y quién lo necesita.",
        steps: &["Escribe `pacman -Qi` seguido de un paquete instalado, por ejemplo `pacman -Qi bash`.", "Busca las líneas \"Depende de\" y \"Exigido por\"."],
        command: "pacman -Qi bash",
        done_text: "\"Exigido por\" te dice qué otros paquetes dejarían de funcionar si quitaras este. Mirarlo antes de desinstalar evita sustos.",
        xp: 15,
        check: Check::Ran(&["pacman -Qi"]),
    },
    Mission {
        id: "servicio",
        stage: 2,
        title: "Mira un servicio por dentro",
        why: "La red, el sonido y el inicio de sesión son servicios que maneja systemd. Cuando algo falla, casi siempre se ve aquí.",
        steps: &["Escribe `systemctl status NetworkManager` (o cualquier otro servicio).", "Busca la línea \"Active\": dice si está corriendo y desde cuándo.", "Sal con `q` si se queda la pantalla abierta."],
        command: "systemctl status NetworkManager",
        done_text: "Las últimas líneas de esa pantalla son el registro del servicio. Cuando algo no arranca, el motivo suele estar escrito ahí.",
        xp: 20,
        check: Check::Ran(&["systemctl status", "systemctl --user status"]),
    },
    Mission {
        id: "registro",
        stage: 2,
        title: "Lee el registro del sistema",
        why: "Linux anota todo lo que pasa. Leer los errores del arranque actual te deja diagnosticar sin adivinar.",
        steps: &["Escribe `journalctl -p err -b`.", "Verás solo los errores desde que encendiste. Es normal que haya algunos.", "Sal con `q`."],
        command: "journalctl -p err -b",
        done_text: "Ya sabes dónde mirar cuando algo va mal. Si un mensaje no se entiende, pégamelo y lo traduzco.",
        xp: 20,
        check: Check::Ran(&["journalctl"]),
    },
    Mission {
        id: "actualizar",
        stage: 3,
        title: "Actualiza tu sistema",
        why: "Arch se actualiza de forma continua. Hacerlo completo y seguido es lo que lo mantiene estable.",
        steps: &["Pídeme \"actualiza el sistema\" para ver qué viene y qué riesgo tiene, o escribe `sudo pacman -Syu`.", "Lee la lista antes de aceptar.", "Si se actualizó el kernel, reinicia cuando puedas."],
        command: "sudo pacman -Syu",
        done_text: "Sistema al día. Nunca uses -Sy sin la u: instalar con la lista nueva sobre un sistema viejo es la forma clásica de romper Arch.",
        xp: 30,
        check: Check::PacmanLog("starting full system upgrade"),
    },
    Mission {
        id: "huerfanos",
        stage: 3,
        title: "Busca paquetes huérfanos",
        why: "Al desinstalar programas quedan dependencias que ya nadie usa. Saber listarlas es mantener tu sistema limpio a propósito.",
        steps: &["Escribe `pacman -Qtdq`.", "Si no sale nada, tu sistema ya está limpio.", "Si sale una lista, mira qué es cada cosa con `pacman -Qi` antes de quitar nada."],
        command: "pacman -Qtdq",
        done_text: "Con esto ya haces el mantenimiento básico de Arch por tu cuenta: actualizar, revisar servicios, leer el registro y limpiar.",
        xp: 20,
        check: Check::Ran(&["pacman -Qtdq", "pacman -Qdtq", "pacman -Qdt", "pacman -Qtd"]),
    },
];

pub const LEVELS: &[(u32, &str)] = &[(0, "Recién aterrizando"), (35, "Con manos en la terminal"), (90, "Entendiendo el sistema"), (150, "Ya es tu Arch")];

#[derive(Serialize, Clone, Debug)]
pub struct MissionView {
    pub id: &'static str,
    pub stage: &'static str,
    pub title: &'static str,
    pub why: &'static str,
    pub steps: &'static [&'static str],
    pub command: &'static str,
    pub xp: u32,
    /// done | active | open
    pub state: &'static str,
}

#[derive(Serialize, Clone, Debug)]
pub struct PathView {
    pub level: usize,
    pub level_name: &'static str,
    pub xp: u32,
    /// XP del siguiente nivel, o `None` en el último.
    pub next_xp: Option<u32>,
    pub done: usize,
    pub total: usize,
    pub missions: Vec<MissionView>,
    /// Si el hook de la shell está instalado y escribiendo.
    pub hook: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct CheckResult {
    pub done: bool,
    pub message: String,
    pub xp: u32,
    pub level_up: Option<&'static str>,
}

pub fn xp(mem: &Memory) -> u32 {
    MISSIONS.iter().filter(|m| mem.missions_done.iter().any(|d| d == m.id)).map(|m| m.xp).sum()
}

fn level_of(xp: u32) -> usize {
    LEVELS.iter().rposition(|(min, _)| xp >= *min).unwrap_or(0)
}

pub fn find(id: &str) -> Option<&'static Mission> {
    MISSIONS.iter().find(|m| m.id == id)
}

/// La siguiente misión sin hacer, en orden.
pub fn next(mem: &Memory) -> Option<&'static Mission> {
    MISSIONS.iter().find(|m| !mem.missions_done.iter().any(|d| d == m.id))
}

pub fn view(mem: &Memory) -> PathView {
    let xp = xp(mem);
    let level = level_of(xp);
    let missions: Vec<MissionView> = MISSIONS
        .iter()
        .map(|m| MissionView {
            id: m.id,
            stage: STAGES[m.stage],
            title: m.title,
            why: m.why,
            steps: m.steps,
            command: m.command,
            xp: m.xp,
            state: if mem.missions_done.iter().any(|d| d == m.id) {
                "done"
            } else if mem.missions_started.contains_key(m.id) {
                "active"
            } else {
                "open"
            },
        })
        .collect();
    PathView {
        level: level + 1,
        level_name: LEVELS[level].1,
        xp,
        next_xp: LEVELS.get(level + 1).map(|(min, _)| *min),
        done: mem.missions_done.len(),
        total: MISSIONS.len(),
        missions,
        hook: recent_file().exists(),
    }
}

fn recent_file() -> PathBuf {
    runtime_dir().join("recientes")
}

fn history_files() -> Vec<PathBuf> {
    let home = home_dir();
    let mut files = vec![data_dir().join("fish/fish_history"), home.join(".zsh_history"), home.join(".bash_history"), home.join(".local/share/zsh/history")];
    if let Some(custom) = std::env::var_os("HISTFILE") {
        files.push(PathBuf::from(custom));
    }
    files
}

fn size(path: &PathBuf) -> u64 {
    fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

/// Lo que se añadió a un archivo desde `offset`. Si el archivo encogió (zsh
/// reescribe su historial), no hay forma de saber qué es nuevo: no se lee nada.
fn tail(path: &PathBuf, offset: u64) -> String {
    let Ok(mut file) = fs::File::open(path) else { return String::new() };
    let len = file.metadata().map(|m| m.len()).unwrap_or(0);
    if len <= offset || file.seek(SeekFrom::Start(offset)).is_err() {
        return String::new();
    }
    let mut bytes = Vec::new();
    let _ = file.take(2 * 1024 * 1024).read_to_end(&mut bytes);
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Marca el punto de partida de una misión.
pub fn start(mem: &mut Memory, id: &str) {
    if find(id).is_none() || mem.missions_done.iter().any(|d| d == id) || mem.missions_started.contains_key(id) {
        return;
    }
    let mut started = Started { at: now(), ..Default::default() };
    for file in history_files().into_iter().chain([PathBuf::from(PACMAN_LOG)]) {
        started.offsets.insert(file.to_string_lossy().into_owned(), size(&file));
    }
    mem.missions_started.insert(id.to_string(), started);
    mem.save();
}

/// Quita el formato de cada shell y deja el comando limpio, sin `sudo`.
fn clean_history_line(line: &str) -> &str {
    let line = line.trim();
    let line = line.strip_prefix("- cmd: ").unwrap_or(line); // fish
    let line = match line.strip_prefix(": ") {
        Some(rest) => rest.split_once(';').map(|(_, c)| c).unwrap_or(line), // zsh extendido
        None => line,
    };
    let line = line.trim();
    line.strip_prefix("sudo ").or_else(|| line.strip_prefix("doas ")).unwrap_or(line).trim()
}

fn ran_since(started: &Started, prefixes: &[&str]) -> bool {
    let hit = |cmd: &str| prefixes.iter().any(|p| cmd.starts_with(p));
    // Hook de la shell: "segundos<TAB>código<TAB>comando".
    if let Ok(text) = fs::read_to_string(recent_file()) {
        for line in text.lines() {
            let mut parts = line.splitn(3, '\t');
            let (Some(at), Some(_status), Some(cmd)) = (parts.next(), parts.next(), parts.next()) else { continue };
            if at.parse::<u64>().is_ok_and(|t| t + 2 >= started.at) && hit(clean_history_line(cmd)) {
                return true;
            }
        }
    }
    history_files().iter().any(|file| {
        let offset = started.offsets.get(file.to_string_lossy().as_ref()).copied().unwrap_or(u64::MAX);
        tail(file, offset).lines().any(|l| hit(clean_history_line(l)))
    })
}

fn logged_since(started: &Started, needle: &str) -> bool {
    let file = PathBuf::from(PACMAN_LOG);
    let offset = started.offsets.get(PACMAN_LOG).copied().unwrap_or(u64::MAX);
    tail(&file, offset).contains(needle)
}

/// Comprueba una misión contra el sistema y, si está hecha, la apunta.
pub fn check(mem: &mut Memory, id: &str) -> CheckResult {
    let Some(mission) = find(id) else {
        return CheckResult { done: false, message: "No conozco esa misión.".into(), xp: 0, level_up: None };
    };
    if mem.missions_done.iter().any(|d| d == id) {
        return CheckResult { done: true, message: "Esta ya la tenías hecha.".into(), xp: 0, level_up: None };
    }
    let Some(started) = mem.missions_started.get(id).cloned() else {
        start(mem, id);
        return CheckResult { done: false, message: "Misión empezada. Haz los pasos en tu terminal y vuelve a comprobar.".into(), xp: 0, level_up: None };
    };
    let ok = match mission.check {
        Check::Ran(prefixes) => ran_since(&started, prefixes),
        Check::PacmanLog(needle) => logged_since(&started, needle),
    };
    if !ok {
        let hint = match mission.check {
            Check::Ran(_) if !recent_file().exists() => {
                " Si usas zsh o bash, el historial se guarda al cerrar la terminal: ciérrala y comprueba otra vez, o instala el hook con `archito hook`."
            }
            _ => "",
        };
        return CheckResult {
            done: false,
            message: format!("Todavía no veo `{}` en tu terminal desde que empezaste la misión.{hint}", mission.command),
            xp: 0,
            level_up: None,
        };
    }
    let before = level_of(xp(mem));
    mem.missions_done.push(id.to_string());
    mem.missions_started.remove(id);
    mem.save();
    let after = level_of(xp(mem));
    CheckResult { done: true, message: mission.done_text.to_string(), xp: mission.xp, level_up: (after > before).then(|| LEVELS[after].1) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limpia_los_formatos_de_historial() {
        assert_eq!(clean_history_line("- cmd: uname -r"), "uname -r");
        assert_eq!(clean_history_line(": 1760000000:0;sudo pacman -Ss vlc"), "pacman -Ss vlc");
        assert_eq!(clean_history_line("man pacman"), "man pacman");
    }

    #[test]
    fn los_niveles_suben_con_la_experiencia() {
        assert_eq!(level_of(0), 0);
        assert_eq!(level_of(35), 1);
        assert_eq!(level_of(1000), LEVELS.len() - 1);
        let total: u32 = MISSIONS.iter().map(|m| m.xp).sum();
        assert!(total >= LEVELS.last().unwrap().0, "el último nivel tiene que ser alcanzable");
    }

    #[test]
    fn una_mision_sin_empezar_no_se_da_por_hecha() {
        let mut mem = Memory::ephemeral();
        let r = check(&mut mem, "kernel");
        assert!(!r.done);
        assert!(mem.missions_started.contains_key("kernel"));
    }
}
