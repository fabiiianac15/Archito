//! El intérprete: recibe texto, decide la intención, consulta el sistema real
//! y devuelve una respuesta estructurada. No sabe nada de ventanas.

use std::collections::BTreeMap;
use std::fs;

use serde::Serialize;

use crate::errors::{self, ErrorHelp};
use crate::exec;
use crate::explain;
use crate::glossary;
use crate::lessons;
use crate::memory::{self, Memory};
use crate::model::{Action, ExecResult, Level, Response, Status};
use crate::probe;
use crate::safety::classify;
use crate::text::{norm, phrase_match, strip_any};

type Handler = fn(&mut Archito, &str) -> Response;

/// Intenciones por palabras clave. Gana la frase más larga; en un empate, la
/// que está antes en la lista. Por eso las guías de problemas van primero.
const INTENTS: &[(&[&str], Handler)] = &[
    // ── Cuando algo no funciona ──
    (&["no arranca", "no enciende", "no inicia", "no prende", "no bootea", "pantalla negra", "grub", "arranque", "problema de arranque", "no carga el sistema"], h_boot),
    (&["sonido", "audio", "no suena", "no se escucha", "no escucho", "parlantes", "altavoces", "audifonos", "volumen"], h_sound),
    (&["internet", "wifi", "wi-fi", "conexion", "la red", "sin red", "no tengo red", "no conecta", "se cae la red"], h_network),
    (&["lento", "lenta", "va lento", "rendimiento", "se traba", "se congela", "se pega", "diagnostico"], h_slow),
    (&["por que fallo", "porque fallo", "que salio mal", "ultimo error", "que paso con el comando"], h_why),
    // ── Aprender ──
    (&["camino", "mision", "misiones", "quiero aprender", "ensename", "por donde empiezo", "que sigue", "siguiente mision", "leccion*", "mi progreso", "mi nivel"], h_path),
    (&["comprobar", "comprueba", "ya lo hice", "ya la hice", "revisa mi mision"], h_check),
    (&["bitacora", "comandos aprendidos", "que he aprendido", "mis comandos"], h_logbook),
    (&["como instal*", "como se instala*", "instalar programas", "instalar aplicaciones", "donde descargo", "descargar programas", "tienda de aplicaciones"], h_how_install),
    (&["modo profesor", "explicame todo", "activa las explicaciones", "modo principiante"], h_teach_on),
    (&["modo experto", "sin explicaciones", "desactiva las explicaciones"], h_teach_off),
    (&["consejo", "tip", "truco", "dato curioso"], h_tip),
    (&["hook", "integracion con la terminal", "conectar la terminal"], h_hook),
    // ── Conversación ──
    (&["ayuda", "help", "que puedes hacer", "que sabes hacer"], h_help),
    (&["quien eres", "que eres", "como te llamas"], h_identity),
    (&["gracias", "muchas gracias", "thanks"], h_thanks),
    (&["adios", "chao", "hasta luego", "bye", "nos vemos"], h_bye),
    (&["hola", "holi", "buenas", "hey", "buenos dias", "buenas tardes", "buenas noches", "saludos"], h_hello),
    // ── Sistema ──
    (&["info del sistema", "informacion del sistema", "resumen del sistema", "info sistema", "mi equipo", "mi pc", "mi computador", "mi maquina"], h_system),
    (&["salud", "chequeo", "revisa mi sistema", "estado del sistema", "como esta mi sistema"], h_health),
    (&["kernel", "nucleo"], h_kernel),
    (&["distro", "distribucion", "que linux tengo", "que sistema tengo"], h_distro),
    (&["temperatura", "caliente", "se calienta", "calor"], h_temp),
    (&["procesos", "que consume", "uso de cpu", "quien usa la ram", "que esta abierto"], h_processes),
    (&["cpu", "procesador"], h_cpu),
    (&["ram", "memoria", "swap"], h_ram),
    (&["disco", "espacio", "almacenamiento", "espacio libre"], h_disk),
    (&["bateria"], h_battery),
    (&["uptime", "tiempo encendido", "cuanto lleva encendido", "cuanto lleva prendido"], h_uptime),
    (&["mi ip", "direccion ip"], h_ip),
    // ── Paquetes ──
    (&["actualiza el sistema", "actualizar el sistema", "actualiza todo", "actualiza", "actualizar", "update", "pacman -syu", "pon al dia"], a_update),
    (&["actualizaciones", "hay actualizaciones", "updates pendientes"], h_updates),
    (&["huerfano*", "paquetes sin uso"], h_orphans),
    (&["cache", "limpia la cache", "limpiar cache", "liberar espacio"], h_cache),
    (&["cuantos paquetes", "paquetes instalados", "que tengo instalado"], h_packages),
    // ── Servicios y registro ──
    (&["servicios fallidos", "servicios caidos", "servicios", "systemd"], h_failed),
    (&["errores recientes", "errores del sistema", "errores", "logs", "journal", "registro"], h_errors),
    (&["mi actividad", "que hice", "historial de acciones"], h_activity),
];

const TIPS: &[&str] = &[
    "**man comando** abre el manual de cualquier comando. Vive en tu disco y funciona sin Internet.",
    "La tecla **Tab** completa nombres de archivos y comandos. Dos veces Tab muestra las opciones.",
    "**Ctrl+R** busca en tu historial mientras escribes. Sirve para recuperar ese comando largo de la semana pasada.",
    "En Arch se actualiza completo y seguido: **sudo pacman -Syu**. Usar -Sy sin la u es la forma clásica de romper dependencias.",
    "**Ctrl+C** interrumpe lo que esté corriendo en la terminal. No rompe nada.",
    "Antes de copiar un comando de Internet, entiéndelo. Dime \"explica\" y el comando, y lo desarmo.",
    "**journalctl -p err -b** muestra solo los errores desde que encendiste.",
    "La Arch Wiki es la mejor documentación de Linux que existe. Casi cualquier duda tiene su página.",
    "Si pacman te pregunta algo y no sabes, la opción por defecto (la letra en mayúscula) suele ser la segura.",
    "**pacman -Qo /ruta/archivo** te dice qué paquete puso ese archivo en tu sistema.",
];

/// Entrada de la bitácora de comandos.
#[derive(Serialize, Clone, Debug)]
pub struct LogEntry {
    pub program: String,
    pub doc: String,
    pub times: u32,
}

pub struct Archito {
    pub memory: Memory,
    pending: BTreeMap<u64, Action>,
    next_id: u64,
    tip: usize,
}

impl Default for Archito {
    fn default() -> Self {
        Self::new()
    }
}

impl Archito {
    pub fn new() -> Self {
        trim_recent();
        Self::with_memory(Memory::load())
    }

    pub fn with_memory(memory: Memory) -> Self {
        Archito { memory, pending: BTreeMap::new(), next_id: 1, tip: memory::now() as usize }
    }

    // ── API pública ──────────────────────────────────────────────────────────

    pub fn welcome(&self) -> Response {
        let name = self.memory.user_name.as_deref().map(|n| format!(", {n}")).unwrap_or_default();
        let next = match lessons::next(&self.memory) {
            Some(m) if self.memory.missions_done.is_empty() => {
                format!("\n\nSi estás empezando, tengo un camino de misiones cortas para que le pierdas el miedo a la terminal. La primera: **{}**.", m.title)
            }
            Some(m) => format!("\n\nTu siguiente misión: **{}**.", m.title),
            None => "\n\nYa terminaste el camino. Ahora la terminal es tuya.".to_string(),
        };
        Response::new(format!(
            "¡Hola{name}! Soy **Archito**.\nTe acompaño mientras aprendes Arch: te explico cada comando, traduzco los errores y reviso tu sistema contigo.{next}"
        ))
        .chips(&["camino", "info del sistema", "ayuda"])
    }

    pub fn process(&mut self, raw: &str) -> Response {
        let n = norm(raw);
        if n.is_empty() {
            return Response::new("No me llegó nada. ¿Me lo repites?");
        }
        let mut resp = self.route(&n, raw);

        // Modo profesor: toda acción llega con su comando desarmado.
        if self.memory.teach && resp.steps.is_empty() {
            if let Some(action) = resp.actions.first() {
                resp.steps = explain::explain(&action.command);
            }
        }
        // Bitácora: se apunta cada programa que el usuario ve por primera vez.
        let mut shown: Vec<String> = Vec::new();
        shown.extend(resp.learn.iter().filter_map(|l| explain::main_program(&l.command)));
        shown.extend(resp.actions.iter().filter_map(|a| explain::main_program(&a.command)));
        if let Some(first) = resp.steps.iter().find(|s| s.token != "sudo") {
            shown.push(first.token.rsplit('/').next().unwrap_or(&first.token).to_string());
        }
        let mut changed = false;
        for program in shown {
            if explain::known(&program).is_some() || exec::has_tool(&program) {
                self.memory.see(&program);
                changed = true;
            }
        }
        if changed {
            self.memory.save();
        }
        resp
    }

    /// Desarma un comando pieza por pieza y lo apunta en la bitácora.
    pub fn explain(&mut self, command: &str) -> Response {
        let resp = self.explain_only(command);
        if let Some(program) = explain::main_program(command).filter(|_| !resp.steps.is_empty()) {
            self.memory.see(&program);
            self.memory.save();
        }
        resp
    }

    /// El desglose sin tocar la bitácora: `process` ya apunta lo que muestra.
    fn explain_only(&mut self, command: &str) -> Response {
        let steps = explain::explain(command);
        if steps.is_empty() {
            return Response::new("Escribe un comando después de \"explica\". Por ejemplo: explica sudo pacman -Syu");
        }
        let level = classify(command);
        let verdict = match level {
            Level::Safe => "Solo lee información: no cambia nada en tu sistema.",
            Level::Confirm => "Cambia algo en tu sistema. Léelo completo antes de ejecutarlo.",
            Level::Danger => "Cuidado: puede borrar datos o dejar el sistema sin funcionar. No lo ejecutes si no sabes exactamente qué hace.",
        };
        let mut resp = Response::new(format!("Así se lee `{command}`:\n\n> {verdict}")).steps(steps).chips(&["bitácora", "consejo"]);
        if level == Level::Danger {
            resp.status = Status::Warn;
        }
        resp
    }

    /// Explica el último comando que falló en la terminal (necesita el hook).
    pub fn why(&mut self) -> Response {
        let Some((status, command)) = last_command() else {
            return Response::new(
                "Todavía no veo tu terminal. Para que pueda explicarte el último comando que falló, instala el hook de la shell:\n\n> Apunta cada comando y su resultado en una carpeta temporal de tu sesión. Se borra al cerrar sesión y no sale del equipo.",
            )
            .code("archito hook --install")
            .chips(&["hook", "ayuda"]);
        };
        if status == 0 {
            return Response::new(format!("Tu último comando terminó bien (código 0):\n\n`{command}`\n\nSi viste un mensaje raro de todas formas, pégamelo y lo reviso."))
                .chips(&[&format!("explica {command}")]);
        }
        let help = errors::from_failure(&command, status);
        let mut resp = self.error_response(help);
        resp.text = format!("Tu último comando falló con el código **{status}**:\n\n`{command}`\n\n{}", resp.text);
        resp.chips.insert(0, format!("explica {command}"));
        resp
    }

    pub fn path(&self) -> lessons::PathView {
        lessons::view(&self.memory)
    }

    pub fn start_mission(&mut self, id: &str) {
        lessons::start(&mut self.memory, id);
    }

    pub fn check_mission(&mut self, id: &str) -> lessons::CheckResult {
        lessons::check(&mut self.memory, id)
    }

    pub fn logbook(&self) -> Vec<LogEntry> {
        self.memory
            .seen
            .iter()
            .map(|(program, times)| LogEntry {
                program: program.clone(),
                doc: explain::known(program).map(str::to_string).or_else(|| probe::whatis(program)).unwrap_or_else(|| "Sin descripción todavía.".into()),
                times: *times,
            })
            .collect()
    }

    /// Ejecuta una acción que Archito propuso antes. La interfaz solo manda el
    /// número: el comando que se ejecuta es el que guardó el núcleo.
    pub fn execute(&mut self, id: u64) -> ExecResult {
        let Some(action) = self.pending.remove(&id) else {
            return ExecResult { ok: false, mode: "inline", output: "Esa acción ya no está disponible. Pídemela otra vez.".into() };
        };
        if action.level == Level::Safe {
            let result = exec::run_inline(&action.command);
            memory::log_activity(&action.command, if result.is_ok() { "ejecutado" } else { "falló" });
            return match result {
                Ok(output) => ExecResult { ok: true, mode: "inline", output },
                Err(output) => ExecResult { ok: false, mode: "inline", output },
            };
        }
        let opened = exec::open_in_terminal(&action.command);
        memory::log_activity(&action.command, if opened { "enviado a terminal" } else { "sin terminal" });
        ExecResult {
            ok: opened,
            mode: "terminal",
            output: if opened {
                "Abrí una terminal con el comando. Ahí ves todo lo que pasa y escribes tu contraseña si la pide.".into()
            } else {
                format!("No encontré un emulador de terminal. Copia el comando y ejecútalo tú:\n{}", action.command)
            },
        }
    }

    /// Saca una acción pendiente para que la ejecute quien llama (la CLI).
    pub fn take_action(&mut self, id: u64) -> Option<Action> {
        self.pending.remove(&id)
    }

    pub fn cancel(&mut self, id: u64) {
        if let Some(action) = self.pending.remove(&id) {
            memory::log_activity(&action.command, "cancelado");
        }
    }

    // ── Internos ─────────────────────────────────────────────────────────────

    /// Prepara una acción. El nivel lo pone el clasificador, no quien la pide.
    fn propose(&mut self, label: &str, command: &str, description: &str) -> Action {
        let level = classify(command);
        let action = Action {
            id: self.next_id,
            label: label.to_string(),
            command: command.to_string(),
            level,
            description: description.to_string(),
            danger_note: if level == Level::Danger {
                "Este comando borra o quita cosas. Lee cada parte antes de confirmar.".into()
            } else {
                String::new()
            },
        };
        self.next_id += 1;
        self.pending.insert(action.id, action.clone());
        // Las acciones viejas caducan: no se acumulan sin límite.
        while self.pending.len() > 40 {
            self.pending.pop_first();
        }
        action
    }

    fn error_response(&mut self, help: ErrorHelp) -> Response {
        let fixes: Vec<String> = help.fix.iter().map(|f| format!("• {f}")).collect();
        let mut resp = Response::new(format!("**{}**\n\n{}\n\n**Qué hacer:**\n{}", help.title, help.why, fixes.join("\n"))).status(Status::Warn);
        if let Some(command) = help.command {
            let label = if classify(&command) == Level::Safe { "Verlo ahora" } else { "Abrir en terminal" };
            let action = self.propose(label, &command, "El paso que suele resolver este error.");
            resp.chips.push(format!("explica {command}"));
            resp.actions.push(action);
        }
        resp
    }

    fn route(&mut self, n: &str, raw: &str) -> Response {
        // 1. ¿Pegó un mensaje de error?
        if let Some(help) = errors::from_text(raw) {
            return self.error_response(help);
        }

        // 2. Nombre
        if let Some(name) = strip_any(n, &["me llamo", "mi nombre es"]) {
            if !name.is_empty() && name.split(' ').count() <= 2 {
                let name: String = name.split(' ').map(capitalize).collect::<Vec<_>>().join(" ");
                self.memory.user_name = Some(name.clone());
                self.memory.save();
                return Response::new(format!("Encantado, **{name}**. Lo guardé en tu equipo (~/.config/archito/) y en ningún otro sitio."))
                    .chips(&["camino", "info del sistema"]);
            }
        }

        // 3. Explicar un comando
        if let Some(rest) = strip_any(n, &["explica", "explicame", "que hace", "para que sirve", "como funciona", "como se usa"]) {
            let rest = strip_any(rest, &["el comando", "este comando", "el"]).unwrap_or(rest);
            let original = original_tail(raw, rest);
            if looks_like_command(&original) {
                return self.explain_only(&original);
            }
        }

        // 4. Glosario
        if let Some(rest) = strip_any(n, &["que es", "que son", "que significa", "que quiere decir", "explica", "explicame"]) {
            let term = strip_any(rest, &["un", "una", "el", "la", "los", "las", "eso de", "eso del"]).unwrap_or(rest);
            if let Some(entry) = glossary::lookup(term) {
                return self.glossary_response(entry);
            }
            if !term.contains(' ') && looks_like_command(term) {
                return self.explain_only(term);
            }
        }

        // 5. Instalar, quitar y buscar
        if let Some(rest) = strip_any(n, &["instala", "instalar", "instalame", "quiero instalar", "necesito instalar"]) {
            let pkg = strip_any(rest, &["el paquete", "el programa", "la aplicacion", "el", "la"]).unwrap_or(rest);
            if pkg.is_empty() || ["un programa", "programas", "algo", "una aplicacion", "aplicaciones", "un paquete", "paquetes", "cosas"].contains(&pkg) {
                return h_how_install(self, n);
            }
            return self.install(pkg);
        }
        if let Some(rest) = strip_any(n, &["desinstala", "desinstalar", "elimina", "eliminar", "quita", "quitar", "borra el paquete"]) {
            let pkg = strip_any(rest, &["el paquete", "el programa", "la aplicacion", "el", "la"]).unwrap_or(rest);
            if valid_package(pkg) {
                return self.remove(pkg);
            }
        }
        if let Some(rest) = strip_any(n, &["busca", "buscar", "buscame", "encuentra"]) {
            let term = strip_any(rest, &["el paquete", "un paquete de", "paquetes de", "un", "una", "el", "la"]).unwrap_or(rest);
            if !term.is_empty() {
                return self.search(term);
            }
        }

        // 6. Intenciones por palabras completas
        let words: Vec<&str> = n.split(' ').collect();
        let mut best: Option<(usize, Handler)> = None;
        for (keywords, handler) in INTENTS {
            for kw in *keywords {
                if let Some(len) = phrase_match(&words, kw) {
                    if best.is_none_or(|(b, _)| len > b) {
                        best = Some((len, *handler));
                    }
                }
            }
        }
        if let Some((_, handler)) = best {
            return handler(self, n);
        }

        // 7. Una sola palabra que es un término o un programa
        if words.len() == 1 {
            if let Some(entry) = glossary::lookup(n) {
                return self.glossary_response(entry);
            }
            if looks_like_command(n) {
                return self.explain_only(n);
            }
        }

        Response::new(
            "Eso todavía no lo sé responder, y prefiero decírtelo antes que inventar.\n\nLo que sí puedo hacer ahora: explicarte un comando, traducir un error que pegues aquí, revisar tu sistema o seguir con tu camino.",
        )
        .chips(&["ayuda", "camino", "salud"])
    }

    fn glossary_response(&mut self, entry: &glossary::Term) -> Response {
        let mut resp = Response::new(format!("**{}**\n\n{}", entry.title, entry.text)).chips(&["camino", "consejo"]);
        if let Some(cmd) = entry.try_it {
            let action = self.propose("Verlo en mi equipo", cmd, "Una lectura: no cambia nada.");
            resp.actions.push(action);
        }
        resp
    }

    fn install(&mut self, pkg: &str) -> Response {
        if !valid_package(pkg) {
            return self.search(pkg);
        }
        if probe::is_installed(pkg) {
            return Response::new(format!("**{pkg}** ya está instalado en tu equipo."))
                .learn(&format!("pacman -Qi {pkg}"), "Así ves la ficha de un paquete que ya tienes.")
                .chips(&[&format!("explica pacman -Qi {pkg}")]);
        }
        if !probe::package_exists(pkg) {
            let found = probe::search_packages(pkg, 5).unwrap_or_default();
            if found.is_empty() {
                return Response::new(format!(
                    "No hay ningún paquete llamado **{pkg}** en los repositorios oficiales, y tampoco encontré parecidos.\n\nPuede estar en el AUR o llamarse de otra forma."
                ))
                .status(Status::Warn)
                .chips(&["qué es el AUR", "cómo instalo un programa"]);
            }
            let code: Vec<String> = found.iter().map(|(name, desc)| format!("{name}\n    {desc}")).collect();
            let chips: Vec<String> = found.iter().take(3).map(|(name, _)| format!("instala {}", name.rsplit('/').next().unwrap_or(name))).collect();
            let mut resp = Response::new(format!("No existe un paquete que se llame exactamente **{pkg}**. Encontré estos, mira si alguno es el que buscas:"))
                .code(code.join("\n"))
                .learn(&format!("pacman -Ss {pkg}"), "Así se busca en los repositorios. No necesita sudo porque solo lee.");
            resp.chips = chips;
            return resp;
        }
        let command = format!("sudo pacman -S --needed {pkg}");
        let action = self.propose(&format!("Instalar {pkg}"), &command, "Descarga el paquete de los repositorios oficiales y lo instala. Pacman te mostrará qué va a instalar y cuánto ocupa antes de empezar.");
        let mut resp = Response::new(format!("**{pkg}** existe en los repositorios oficiales. Este es el comando para instalarlo:"));
        resp.actions.push(action);
        resp.chips(&[&format!("busca {pkg}"), "qué es un paquete"])
    }

    fn remove(&mut self, pkg: &str) -> Response {
        if !probe::is_installed(pkg) {
            return Response::new(format!("No tienes instalado ningún paquete llamado **{pkg}**."))
                .learn("pacman -Qe", "Lista los paquetes que instalaste tú, para encontrar el nombre exacto.");
        }
        let command = format!("sudo pacman -Rns {pkg}");
        let action = self.propose(
            &format!("Quitar {pkg}"),
            &command,
            "Quita el paquete, su configuración y las dependencias que queden sin uso. Pacman cancela todo si otro programa lo necesita.",
        );
        let mut resp = Response::new(format!("Para quitar **{pkg}** se usa este comando. Antes de confirmar, lee la lista que te muestre pacman.")).status(Status::Warn);
        resp.actions.push(action);
        resp.chips(&[&format!("explica pacman -Qi {pkg}")])
    }

    fn search(&mut self, term: &str) -> Response {
        let term = term.split(' ').next().unwrap_or(term);
        if !valid_package(term) {
            return Response::new("Dame una sola palabra para buscar, por ejemplo: busca navegador");
        }
        let learn_cmd = format!("pacman -Ss {term}");
        match probe::search_packages(term, 6) {
            None => Response::new("No pude consultar pacman.").status(Status::Error),
            Some(found) if found.is_empty() => Response::new(format!(
                "No encontré nada con «{term}» en los repositorios oficiales. Pacman busca en nombres y descripciones, que están en inglés: prueba con la palabra en inglés."
            ))
            .learn(&learn_cmd, "Así se busca en los repositorios."),
            Some(found) => {
                let code: Vec<String> = found.iter().map(|(name, desc)| format!("{name}\n    {desc}")).collect();
                let chips: Vec<String> = found.iter().take(3).map(|(name, _)| format!("instala {}", name.rsplit('/').next().unwrap_or(name))).collect();
                let mut resp = Response::new(format!("Esto encontré para «{term}». Lo que va antes de la barra es el repositorio; lo de después, el nombre que se instala."))
                    .code(code.join("\n"))
                    .learn(&learn_cmd, "Así lo buscarías tú. No necesita sudo porque solo lee.");
                resp.chips = chips;
                resp
            }
        }
    }
}

// ── Utilidades ───────────────────────────────────────────────────────────────

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    chars.next().map(|c| c.to_uppercase().collect::<String>() + chars.as_str()).unwrap_or_default()
}

fn valid_package(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 60
        && name.chars().next().is_some_and(|c| c.is_ascii_alphanumeric())
        && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '@' | '_' | '+' | '-'))
}

/// Recupera del texto original las últimas palabras que corresponden a
/// `rest`: los flags distinguen mayúsculas (-Syu no es -syu).
fn original_tail(raw: &str, rest: &str) -> String {
    let count = rest.split_whitespace().count();
    let words: Vec<&str> = raw.split_whitespace().collect();
    let start = words.len().saturating_sub(count);
    words[start..].join(" ").trim_end_matches(['?', '!', '.']).trim_matches('`').to_string()
}

fn looks_like_command(text: &str) -> bool {
    let tokens = explain::tokenize(text);
    let Some(program) = tokens.iter().find(|t| *t != "sudo") else { return false };
    let program = program.rsplit('/').next().unwrap_or(program);
    explain::known(program).is_some()
        || tokens.iter().any(|t| t.starts_with('-') && t.len() > 1)
        || (program.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')) && exec::has_tool(program))
}

fn recent_path() -> std::path::PathBuf {
    memory::runtime_dir().join("recientes")
}

/// Último comando de la terminal según el hook: (código de salida, comando).
fn last_command() -> Option<(i32, String)> {
    let text = fs::read_to_string(recent_path()).ok()?;
    let line = text.lines().rev().find(|l| !l.trim().is_empty())?;
    let mut parts = line.splitn(3, '\t');
    let (_at, status, command) = (parts.next()?, parts.next()?, parts.next()?);
    Some((status.parse().ok()?, command.to_string()))
}

/// El archivo del hook no crece sin fin: se queda con las últimas líneas.
fn trim_recent() {
    let path = recent_path();
    let Ok(text) = fs::read_to_string(&path) else { return };
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() > 400 {
        let _ = fs::write(&path, lines[lines.len() - 150..].join("\n") + "\n");
    }
}

fn severity(pct: f32, warn: f32, danger: f32) -> &'static str {
    if pct >= danger {
        "danger"
    } else if pct >= warn {
        "warn"
    } else {
        "good"
    }
}

fn mark(ok: bool) -> &'static str {
    if ok {
        "✔"
    } else {
        "✘"
    }
}

// ── Conversación ─────────────────────────────────────────────────────────────

fn h_hello(a: &mut Archito, _: &str) -> Response {
    let greeting = match probe::local_hour() {
        0..=11 => "Buenos días",
        12..=18 => "Buenas tardes",
        _ => "Buenas noches",
    };
    let name = a.memory.user_name.as_deref().map(|n| format!(", {n}")).unwrap_or_default();
    let hint = if a.memory.user_name.is_none() { " Si me dices \"me llamo …\", lo recuerdo." } else { "" };
    Response::new(format!("{greeting}{name}. ¿Qué hacemos hoy con tu sistema?{hint}")).chips(&["camino", "salud", "consejo"])
}

fn h_help(_: &mut Archito, _: &str) -> Response {
    Response::new(
        "**Esto hago hoy:**\n\n\
         • **Enseñarte:** camino (misiones) · explica <comando> · qué es <palabra> · bitácora · consejo\n\
         • **Traducir errores:** pega aquí el mensaje, o escribe \"por qué falló\" tras un comando\n\
         • **Cuando algo falla:** no tengo sonido · no tengo internet · va lento · no arranca\n\
         • **Tu sistema:** info del sistema · salud · cpu · ram · disco · temperatura · procesos · batería\n\
         • **Paquetes:** busca <algo> · instala <paquete> · quita <paquete> · actualiza · huérfanos · caché\n\
         • **Registro:** servicios · errores · mi actividad\n\n\
         > Todo corre en tu equipo. Ningún comando que cambie algo se ejecuta sin que lo veas y lo confirmes.",
    )
    .chips(&["camino", "salud", "explica sudo pacman -Syu"])
}

fn h_identity(_: &mut Archito, _: &str) -> Response {
    Response::new(
        "Soy **Archito**, un asistente local para Arch Linux.\n\nVivo en tu equipo, no en la nube: lo que te digo de tu sistema lo leo en el momento, y nada sale de aquí.\n\n> Mi meta es que cada vez me necesites menos. Por eso te muestro siempre el comando real que hay detrás de cada respuesta.",
    )
    .chips(&["camino", "ayuda"])
}

fn h_thanks(a: &mut Archito, _: &str) -> Response {
    let name = a.memory.user_name.as_deref().map(|n| format!(", {n}")).unwrap_or_default();
    Response::new(format!("Con gusto{name}. Aquí sigo cuando me necesites."))
}

fn h_bye(_: &mut Archito, _: &str) -> Response {
    Response::new("Hasta luego. Que tu uptime sea largo y tus logs cortos.").status(Status::Bye)
}

fn h_tip(a: &mut Archito, _: &str) -> Response {
    a.tip = a.tip.wrapping_add(1);
    Response::new(TIPS[a.tip % TIPS.len()]).chips(&["otro consejo", "camino"])
}

fn h_teach_on(a: &mut Archito, _: &str) -> Response {
    a.memory.teach = true;
    a.memory.save();
    Response::new("Modo profesor activado: cada comando que te proponga llegará desarmado, pieza por pieza.")
}

fn h_teach_off(a: &mut Archito, _: &str) -> Response {
    a.memory.teach = false;
    a.memory.save();
    Response::new("Modo experto: ya no desarmo los comandos por mi cuenta. El botón Explicar sigue ahí cuando lo quieras.")
}

fn h_hook(_: &mut Archito, _: &str) -> Response {
    let active = recent_path().exists();
    let state = if active { "Ya está funcionando en esta sesión." } else { "Todavía no está instalado." };
    Response::new(format!(
        "**El hook de la terminal**\n\n{state}\n\nEs una función pequeña en tu shell (fish, zsh o bash) que apunta cada comando y si terminó bien o mal. Con eso puedo explicarte el último error cuando escribes `archito porque`, y comprobar tus misiones al instante.\n\n> Se guarda en una carpeta temporal de tu sesión, se borra al cerrarla y nunca sale del equipo."
    ))
    .code("archito hook --install")
    .chips(&["por qué falló", "camino"])
}

// ── Aprender ─────────────────────────────────────────────────────────────────

fn h_path(a: &mut Archito, _: &str) -> Response {
    let view = a.path();
    let text = match lessons::next(&a.memory) {
        Some(m) => {
            lessons::start(&mut a.memory, m.id);
            let steps: Vec<String> = m.steps.iter().enumerate().map(|(i, s)| format!("{}. {s}", i + 1)).collect();
            format!(
                "**Nivel {} · {}** ({} de {} misiones)\n\nTu siguiente misión: **{}**\n\n{}\n\n{}\n\n> Cuando la hagas en tu terminal, dime \"comprobar\" o usa el botón del camino.",
                view.level,
                view.level_name,
                view.done,
                view.total,
                m.title,
                m.why,
                steps.join("\n")
            )
        }
        None => format!("**Nivel {} · {}**\n\nCompletaste las {} misiones del camino. Ya haces el mantenimiento de Arch por tu cuenta.", view.level, view.level_name, view.total),
    };
    let command = lessons::next(&a.memory).map(|m| m.command);
    let mut resp = Response::new(text).view("camino");
    if let Some(command) = command {
        resp.chips = vec!["comprobar".into(), format!("explica {command}")];
    }
    resp
}

fn h_check(a: &mut Archito, _: &str) -> Response {
    let Some(mission) = lessons::next(&a.memory) else {
        return Response::new("No queda ninguna misión por comprobar: terminaste el camino.").view("camino");
    };
    let result = a.check_mission(mission.id);
    if !result.done {
        return Response::new(result.message).status(Status::Warn).view("camino").chips(&["camino", &format!("explica {}", mission.command)]);
    }
    let level = result.level_up.map(|name| format!("\n\n**Subiste de nivel: {name}.**")).unwrap_or_default();
    let next = lessons::next(&a.memory).map(|m| format!("\n\nLa siguiente: **{}**.", m.title)).unwrap_or_default();
    Response::new(format!("**Misión cumplida: {}** (+{} XP)\n\n{}{level}{next}", mission.title, result.xp, result.message)).view("camino").chips(&["camino", "bitácora"])
}

fn h_logbook(a: &mut Archito, _: &str) -> Response {
    let entries = a.logbook();
    if entries.is_empty() {
        return Response::new("Tu bitácora está vacía. Cada comando que veas conmigo se apunta aquí con su explicación, para que tengas tu propia chuleta.")
            .view("bitacora")
            .chips(&["camino", "explica sudo pacman -Syu"]);
    }
    let code: Vec<String> = entries.iter().map(|e| format!("{:<14} {}", e.program, e.doc)).collect();
    Response::new(format!("Ya conoces **{} comandos**. Esta es tu bitácora:", entries.len())).code(code.join("\n")).view("bitacora").chips(&["camino"])
}

fn h_how_install(a: &mut Archito, _: &str) -> Response {
    let aur = ["yay", "paru"].into_iter().find(|t| exec::has_tool(t));
    let aur_line = match aur {
        Some(tool) => format!("Tienes **{tool}** instalado: con `{tool} -S nombre` instalas también desde el AUR."),
        None => "Para el AUR hace falta un ayudante como yay o paru, que todavía no tienes.".to_string(),
    };
    let action = a.propose("Probar una búsqueda", "pacman -Ss firefox", "Busca firefox en los repositorios. Solo lee.");
    let mut resp = Response::new(format!(
        "**Cómo se instala un programa en Arch**\n\nAquí no se descargan instaladores de páginas web. Los programas vienen de los repositorios y los instala pacman:\n\n1. **Busca** el nombre exacto: `pacman -Ss palabra`\n2. **Instala** con ese nombre: `sudo pacman -S nombre`\n3. Si no está en los oficiales, mira en el **AUR**. {aur_line}\n\n> Dime \"busca\" y una palabra, y lo hago contigo."
    ));
    resp.actions.push(action);
    resp.chips(&["busca navegador", "qué es el AUR", "qué es un paquete"])
}

fn h_why(a: &mut Archito, _: &str) -> Response {
    a.why()
}

// ── Sistema ──────────────────────────────────────────────────────────────────

fn h_system(_: &mut Archito, _: &str) -> Response {
    Response::new("Este es tu equipo ahora mismo:")
        .code(probe::summary())
        .learn("fastfetch", "Muestra un resumen parecido en la terminal. Si no lo tienes: sudo pacman -S fastfetch")
        .chips(&["salud", "temperatura", "disco"])
}

fn h_kernel(_: &mut Archito, _: &str) -> Response {
    match probe::kernel() {
        Some(k) => Response::new(format!("Tu kernel es **{k}**.\n\nEl kernel es el programa que habla con tu hardware y reparte CPU y memoria entre todo lo demás."))
            .learn("uname -r", "Imprime la versión del kernel que está corriendo.")
            .chips(&["qué es el kernel", "distro"]),
        None => Response::new("No pude leer la versión del kernel.").status(Status::Error),
    }
}

fn h_distro(_: &mut Archito, _: &str) -> Response {
    match probe::distro() {
        Some(d) => Response::new(format!("Tu sistema es **{d}**.")).learn("cat /etc/os-release", "Ese archivo guarda el nombre y la versión de tu distribución.").chips(&["kernel", "info del sistema"]),
        None => Response::new("No pude leer tu distribución.").status(Status::Error),
    }
}

fn h_cpu(_: &mut Archito, _: &str) -> Response {
    let model = probe::cpu_model().unwrap_or_else(|| "CPU desconocida".into());
    let mut resp = Response::new(format!("Tu procesador es un **{model}** con {} hilos.", probe::cpu_threads()));
    if let Some(usage) = probe::cpu_usage() {
        resp = resp.stat("Uso de CPU", usage, format!("{usage:.0} %"), severity(usage, 70.0, 90.0));
    }
    resp.learn("lscpu", "Muestra todos los detalles del procesador.").chips(&["procesos", "temperatura"])
}

fn h_temp(_: &mut Archito, _: &str) -> Response {
    match probe::temperature() {
        None => Response::new("No encontré sensores de temperatura que pueda leer.")
            .status(Status::Warn)
            .learn("sensors", "Viene en el paquete lm_sensors: sudo pacman -S lm_sensors")
            .chips(&["instala lm_sensors"]),
        Some((c, name)) => {
            let verdict = match c as u32 {
                0..=49 => "Fresco.",
                50..=69 => "Temperatura normal de trabajo.",
                70..=84 => "Algo caliente. Vigílala si sigue subiendo.",
                _ => "Muy caliente. Revisa la ventilación y qué proceso está exigiendo tanto.",
            };
            Response::new(format!("El sensor más caliente marca **{c:.0} °C** ({name}). {verdict}"))
                .stat("Temperatura", c.min(100.0), format!("{c:.0} °C"), severity(c, 70.0, 85.0))
                .learn("sensors", "Lista todos los sensores de temperatura (paquete lm_sensors).")
                .chips(&["procesos", "va lento"])
        }
    }
}

fn h_ram(_: &mut Archito, _: &str) -> Response {
    let Some(m) = probe::memory() else { return Response::new("No pude leer la memoria.").status(Status::Error) };
    let note = if m.pct > 85.0 { " Vas justo: mira qué la está usando." } else { "" };
    let mut resp = Response::new(format!("Usas **{:.1} de {:.0} GB** de RAM.{note}", m.used_gb, m.total_gb))
        .stat("RAM", m.pct, format!("{:.1} / {:.0} GB", m.used_gb, m.total_gb), severity(m.pct, 80.0, 90.0));
    if let Some(swap) = m.swap_pct {
        resp = resp.stat("Swap", swap, format!("{swap:.0} %"), severity(swap, 50.0, 80.0));
    }
    resp.learn("free -h", "La columna \"available\" es la memoria que de verdad queda para programas nuevos.").chips(&["procesos por memoria", "qué es el swap"])
}

fn h_disk(_: &mut Archito, _: &str) -> Response {
    let Some(d) = probe::disk("/") else { return Response::new("No pude leer el disco.").status(Status::Error) };
    let note = match d.pct {
        p if p > 90.0 => " Queda muy poco: un disco lleno hace fallar actualizaciones.",
        p if p > 75.0 => " Empieza a llenarse.",
        _ => "",
    };
    Response::new(format!("En tu partición raíz quedan **{:.0} GB libres** de {:.0}.{note}", d.free_gb, d.total_gb))
        .stat("Disco /", d.pct, format!("{:.0} GB libres", d.free_gb), severity(d.pct, 75.0, 90.0))
        .learn("df -h", "Muestra el espacio de cada partición. La h pone los tamaños en GB en vez de en bloques.")
        .chips(&["caché", "huérfanos"])
}

fn h_processes(_: &mut Archito, n: &str) -> Response {
    let by_mem = n.contains("memoria") || n.contains("ram");
    let procs = probe::top_processes(by_mem, 6);
    if procs.is_empty() {
        return Response::new("No pude listar los procesos.").status(Status::Error);
    }
    let code: Vec<String> = procs.iter().map(|p| format!("{:<22} CPU {:>5} %   RAM {:>5} %   pid {}", p.name, p.cpu, p.mem, p.pid)).collect();
    let (title, cmd) = if by_mem { ("Los que más memoria usan:", "ps aux --sort=-%mem") } else { ("Los que más CPU usan:", "ps aux --sort=-%cpu") };
    Response::new(title).code(code.join("\n")).learn(cmd, "Para verlo en vivo usa htop o btop. Se sale con q.").chips(&["va lento", "ram"])
}

fn h_battery(_: &mut Archito, _: &str) -> Response {
    match probe::battery() {
        None => Response::new("No encontré batería. Parece un equipo de sobremesa."),
        Some(b) => {
            let state = match b.status.as_str() {
                "Charging" => "cargando",
                "Discharging" => "descargándose",
                "Full" => "llena",
                "Not charging" => "conectada sin cargar",
                other => other,
            };
            let sev = if b.percent <= 20.0 && b.status == "Discharging" { "danger" } else { "good" };
            Response::new(format!("Batería al **{:.0} %**, {state}.", b.percent)).stat("Batería", b.percent, state.to_string(), sev)
        }
    }
}

fn h_uptime(_: &mut Archito, _: &str) -> Response {
    match probe::uptime() {
        Some(up) => Response::new(format!("Llevas **{up}** con el equipo encendido.\n\nTras actualizar el kernel hace falta reiniciar para empezar a usar el nuevo."))
            .learn("uptime", "Muestra el tiempo encendido y la carga del sistema.")
            .chips(&["actualizaciones"]),
        None => Response::new("No pude leer el tiempo encendido.").status(Status::Error),
    }
}

fn h_ip(_: &mut Archito, _: &str) -> Response {
    match probe::local_ip() {
        Some(ip) => Response::new(format!("Tu IP en la red local es **{ip}**.\n\nEs la dirección dentro de tu casa u oficina, no la que ve Internet."))
            .learn("ip addr", "Muestra las direcciones de cada interfaz de red.")
            .chips(&["internet"]),
        None => Response::new("No tienes una dirección IP ahora mismo: no estás en ninguna red.").status(Status::Warn).chips(&["no tengo internet"]),
    }
}

fn h_health(_: &mut Archito, _: &str) -> Response {
    let mut score: i32 = 100;
    let mut findings: Vec<String> = Vec::new();
    let mut resp = Response::new("");
    let mut chips: Vec<&str> = Vec::new();
    if let Some(d) = probe::disk("/") {
        resp = resp.stat("Disco /", d.pct, format!("{:.0} GB libres", d.free_gb), severity(d.pct, 75.0, 90.0));
        if d.pct > 90.0 {
            score -= 25;
            findings.push("El disco está casi lleno.".into());
            chips.push("caché");
        } else if d.pct > 75.0 {
            score -= 8;
            findings.push("El disco empieza a llenarse.".into());
        }
    }
    if let Some(m) = probe::memory() {
        resp = resp.stat("RAM", m.pct, format!("{:.1} / {:.0} GB", m.used_gb, m.total_gb), severity(m.pct, 80.0, 90.0));
        if m.pct > 90.0 {
            score -= 15;
            findings.push("La memoria está casi llena.".into());
            chips.push("procesos por memoria");
        }
    }
    if let Some((c, _)) = probe::temperature() {
        if c >= 85.0 {
            score -= 15;
            findings.push(format!("El equipo está muy caliente ({c:.0} °C)."));
        }
    }
    let failed = probe::failed_services();
    if !failed.is_empty() {
        score -= 15;
        findings.push(format!("{} servicio(s) fallaron: {}.", failed.len(), failed.iter().take(3).cloned().collect::<Vec<_>>().join(", ")));
        chips.push("servicios");
    }
    match probe::updates() {
        Some(u) if u.len() > 150 => {
            score -= 15;
            findings.push(format!("Hay {} actualizaciones acumuladas. En Arch conviene no dejar pasar tanto.", u.len()));
            chips.push("actualiza");
        }
        Some(u) if !u.is_empty() => {
            score -= 3;
            findings.push(format!("Hay {} actualizaciones pendientes.", u.len()));
            chips.push("actualiza");
        }
        Some(_) => {}
        None => findings.push("No puedo ver las actualizaciones pendientes: falta el paquete pacman-contrib.".into()),
    }
    let orphans = probe::orphans();
    if !orphans.is_empty() {
        score -= 2;
        findings.push(format!("{} paquete(s) huérfano(s) ocupando espacio.", orphans.len()));
        chips.push("huérfanos");
    }
    let score = score.max(0);
    let body = if findings.is_empty() { "No encontré nada que atender.".to_string() } else { findings.iter().map(|f| format!("• {f}")).collect::<Vec<_>>().join("\n") };
    resp.text = format!("Salud del sistema: **{score} de 100**\n\n{body}\n\n> Solo leí datos de tu equipo. No cambié nada.");
    resp.status = if score < 80 { Status::Warn } else { Status::Ok };
    if chips.is_empty() {
        chips.push("camino");
    }
    resp.chips(&chips)
}

fn h_slow(_: &mut Archito, _: &str) -> Response {
    let mut findings: Vec<String> = Vec::new();
    let usage = probe::cpu_usage();
    if let Some(u) = usage.filter(|u| *u >= 70.0) {
        let top: Vec<String> = probe::top_processes(false, 2).into_iter().map(|p| p.name).collect();
        findings.push(format!("La CPU está al **{u:.0} %**. Los que más piden: {}.", top.join(", ")));
    }
    if let Some(m) = probe::memory() {
        if m.pct >= 85.0 {
            let top: Vec<String> = probe::top_processes(true, 2).into_iter().map(|p| p.name).collect();
            findings.push(format!("La RAM está al **{:.0} %**. Los que más ocupan: {}.", m.pct, top.join(", ")));
        }
        if m.swap_pct.is_some_and(|s| s >= 50.0) {
            findings.push("Se está usando bastante swap: el sistema guarda memoria en disco, y eso se nota lento.".into());
        }
    }
    if let Some(d) = probe::disk("/").filter(|d| d.pct >= 90.0) {
        findings.push(format!("El disco está al **{:.0} %**. Casi lleno, todo se vuelve lento.", d.pct));
    }
    if let Some((c, _)) = probe::temperature().filter(|(c, _)| *c >= 85.0) {
        findings.push(format!("La temperatura está en **{c:.0} °C**: el procesador baja su velocidad para protegerse."));
    }
    let failed = probe::failed_services();
    if !failed.is_empty() {
        findings.push(format!("Hay {} servicio(s) caído(s): {}.", failed.len(), failed.iter().take(3).cloned().collect::<Vec<_>>().join(", ")));
    }
    if findings.is_empty() {
        return Response::new(
            "Revisé CPU, memoria, swap, disco, temperatura y servicios, y **todo está en niveles normales** ahora mismo.\n\nSi la lentitud es de un programa concreto, ábrelo y pregúntame otra vez mientras ocurre.",
        )
        .learn("htop", "Monitor en vivo de procesos. Ordena por CPU con F6. Se sale con q.")
        .chips(&["procesos", "errores"]);
    }
    let body: Vec<String> = findings.iter().map(|f| format!("• {f}")).collect();
    Response::new(format!("Revisé tu sistema y esto es lo que puede estar frenándolo:\n\n{}", body.join("\n")))
        .status(Status::Warn)
        .learn("htop", "Monitor en vivo de procesos. Se sale con q.")
        .chips(&["procesos", "procesos por memoria", "errores"])
}

// ── Guías de problemas ───────────────────────────────────────────────────────

fn h_sound(a: &mut Archito, n: &str) -> Response {
    if !exec::has_tool("wpctl") {
        return Response::new("No encuentro PipeWire (el servidor de sonido) en tu sistema. Sin él no hay audio.")
            .status(Status::Warn)
            .chips(&["instala pipewire", "qué es pipewire"]);
    }
    let pipewire = probe::service_active("pipewire.service", true) || probe::service_active("pipewire.socket", true);
    let plumber = probe::service_active("wireplumber.service", true);
    let volume = probe::volume();
    let mut lines = vec![format!("{} PipeWire (servidor de sonido) corriendo", mark(pipewire)), format!("{} WirePlumber (elige el dispositivo) corriendo", mark(plumber))];
    match volume {
        Some((pct, muted)) => {
            lines.push(format!("{} Salida por defecto sin silenciar", mark(!muted)));
            lines.push(format!("{} Volumen al {pct} %", mark(pct > 5)));
        }
        None => lines.push("✘ No hay ninguna salida de audio por defecto".into()),
    }
    let only_asking = n.contains("volumen") && !n.contains("no ");
    let mut resp = Response::new("").code(lines.join("\n")).learn("wpctl status", "Lista tus dispositivos de sonido. El que tiene un asterisco es el que está en uso.");
    let (text, status) = if !pipewire || !plumber {
        let action = a.propose("Reiniciar el sonido", "systemctl --user restart pipewire wireplumber", "Reinicia los servicios de sonido de tu usuario. Se corta el audio un segundo y vuelve.");
        resp.actions.push(action);
        ("El servicio de sonido no está corriendo. Eso explica que no suene nada. Reiniciarlo suele bastar:", Status::Warn)
    } else if let Some((_, true)) = volume {
        let action = a.propose("Quitar el silencio", "wpctl set-mute @DEFAULT_AUDIO_SINK@ 0", "Quita el silencio de la salida de audio por defecto.");
        resp.actions.push(action);
        ("Encontré el motivo: la salida de audio está **silenciada**.", Status::Warn)
    } else if volume.is_some_and(|(p, _)| p <= 5) {
        let action = a.propose("Subir el volumen al 50 %", "wpctl set-volume @DEFAULT_AUDIO_SINK@ 0.5", "Pone el volumen de la salida por defecto a la mitad.");
        resp.actions.push(action);
        ("El volumen está casi en cero.", Status::Warn)
    } else if volume.is_none() {
        ("Los servicios corren, pero no hay ninguna salida seleccionada. Puede que el sistema no esté viendo tu tarjeta de sonido o tus audífonos.", Status::Warn)
    } else if only_asking {
        ("Así está tu sonido:", Status::Ok)
    } else {
        (
            "Los servicios de sonido corren, no está silenciado y hay volumen. Si aun así no suena, casi siempre es que el audio está saliendo por otro dispositivo (HDMI, audífonos Bluetooth). Míralo con el comando de abajo: el asterisco marca la salida en uso.",
            Status::Ok,
        )
    };
    resp.text = text.to_string();
    resp.status = status;
    if resp.actions.is_empty() {
        let action = a.propose("Ver mis dispositivos de sonido", "wpctl status", "Una lectura: no cambia nada.");
        resp.actions.push(action);
    }
    resp.chips(&["qué es pipewire", "errores"])
}

fn h_network(a: &mut Archito, _: &str) -> Response {
    let ifaces = probe::interfaces();
    let up: Vec<&(String, String)> = ifaces.iter().filter(|(_, s)| s == "UP").collect();
    let ip = probe::local_ip();
    let ping = probe::internet();
    let dns = ping && probe::dns_works();
    let manager = ["NetworkManager.service", "iwd.service", "systemd-networkd.service"].into_iter().find(|u| probe::service_active(u, false));
    let lines = [
        format!("{} Servicio de red corriendo{}", mark(manager.is_some()), manager.map(|m| format!(" ({})", m.trim_end_matches(".service"))).unwrap_or_default()),
        format!("{} Interfaz conectada{}", mark(!up.is_empty()), up.first().map(|(n, _)| format!(" ({n})")).unwrap_or_default()),
        format!("{} Dirección IP{}", mark(ip.is_some()), ip.as_ref().map(|i| format!(" ({i})")).unwrap_or_default()),
        format!("{} Sale a Internet (ping a 1.1.1.1)", mark(ping)),
        format!("{} Traduce nombres (DNS)", mark(dns)),
    ];
    let mut resp = Response::new("").code(lines.join("\n")).learn("ping -c 3 archlinux.org", "Envía tres paquetes de prueba. Si responden, hay conexión y DNS.");
    let nm = exec::has_tool("nmcli");
    let (text, status) = if manager.is_none() {
        if nm {
            let action = a.propose("Arrancar NetworkManager", "sudo systemctl enable --now NetworkManager", "Arranca el servicio de red ahora y lo deja activado para los próximos inicios.");
            resp.actions.push(action);
        }
        ("No hay ningún servicio de red corriendo. Sin él, el equipo no se conecta a nada.", Status::Error)
    } else if up.is_empty() || ip.is_none() {
        if nm {
            let action = a.propose("Ver redes WiFi cercanas", "nmcli device wifi list", "Escanea y lista las redes. Solo lee.");
            resp.actions.push(action);
        }
        (
            "El servicio de red corre, pero no estás conectado a ninguna red. Mira las redes disponibles y conéctate con `nmcli device wifi connect \"nombre\" --ask` (te pedirá la clave).",
            Status::Warn,
        )
    } else if !ping {
        ("Estás conectado a la red local pero no sales a Internet. El problema suele estar en el router o en el proveedor: prueba reiniciar el router.", Status::Warn)
    } else if !dns {
        (
            "Hay Internet, pero falla el DNS: el equipo no logra traducir nombres como archlinux.org a direcciones. Por eso el navegador no abre páginas aunque haya conexión.",
            Status::Warn,
        )
    } else {
        ("Tu conexión funciona: hay red, hay salida a Internet y el DNS responde.", Status::Ok)
    };
    resp.text = text.to_string();
    resp.status = status;
    resp.chips(&["mi ip", "errores"])
}

fn h_boot(a: &mut Archito, _: &str) -> Response {
    // /boot suele ser ilegible sin root: primero se mira qué gestor está instalado.
    let by_package = [("limine", "Limine"), ("grub", "GRUB"), ("refind", "rEFInd")].into_iter().find(|(pkg, _)| probe::is_installed(pkg)).map(|(_, name)| name);
    let by_files = [("/boot/grub", "GRUB"), ("/boot/loader/entries", "systemd-boot"), ("/boot/limine.conf", "Limine"), ("/efi/loader/entries", "systemd-boot")]
        .into_iter()
        .find(|(path, _)| std::path::Path::new(path).exists())
        .map(|(_, name)| name);
    let loader = by_package.or(by_files);
    let loader_line = match loader {
        Some(name) => format!("Tu gestor de arranque es **{name}**."),
        None => "No pude identificar tu gestor de arranque: /boot solo se puede leer como administrador.".to_string(),
    };
    let action = a.propose("Ver errores del arranque anterior", "journalctl -b -1 -p err -n 25 --no-pager", "Lee los errores de la vez anterior que encendiste. No cambia nada.");
    let mut resp = Response::new(format!(
        "Si estás leyendo esto, el sistema arrancó ahora, así que lo podemos investigar con calma.\n\n{loader_line}\n\n**Si un día no arranca, este es el camino de rescate:**\n1. Arranca desde el USB de instalación de Arch.\n2. Monta tu partición raíz en /mnt (y la de arranque en /mnt/boot).\n3. Entra a tu sistema con `arch-chroot /mnt`.\n4. Desde ahí arregla lo que falló: reinstalar el kernel (`pacman -S linux`) y regenerar el arranque suelen bastar.\n\n> La causa más común es una actualización cortada a medias. Por eso conviene no apagar el equipo mientras pacman trabaja."
    ))
    .learn("journalctl -b -1 -p err", "El -1 significa \"el arranque anterior\". Ahí queda escrito qué falló.");
    resp.actions.push(action);
    resp.chips(&["qué es el gestor de arranque", "servicios", "explica arch-chroot /mnt"])
}

// ── Paquetes ─────────────────────────────────────────────────────────────────

fn h_packages(_: &mut Archito, _: &str) -> Response {
    match probe::package_count() {
        Some(n) => Response::new(format!("Tienes **{n} paquetes** instalados. La mayoría son librerías que otros programas necesitan."))
            .learn("pacman -Qe", "Lista solo los que instalaste tú, sin dependencias.")
            .chips(&["huérfanos", "qué es un paquete"]),
        None => Response::new("No pude consultar pacman.").status(Status::Error),
    }
}

fn h_updates(_: &mut Archito, _: &str) -> Response {
    match probe::updates() {
        None => Response::new("Para ver las actualizaciones sin tocar el sistema uso `checkupdates`, que viene en el paquete pacman-contrib y no lo tienes.")
            .status(Status::Warn)
            .chips(&["instala pacman-contrib"]),
        Some(u) if u.is_empty() => Response::new("Tu sistema está al día.").learn("checkupdates", "Lista las actualizaciones pendientes sin instalar nada."),
        Some(u) => {
            let shown: Vec<&str> = u.iter().take(12).map(String::as_str).collect();
            let more = if u.len() > 12 { format!("\n… y {} más", u.len() - 12) } else { String::new() };
            Response::new(format!("Hay **{} paquetes** con actualización disponible:", u.len()))
                .code(format!("{}{more}", shown.join("  ")))
                .learn("checkupdates", "Lista las actualizaciones pendientes sin instalar nada.")
                .chips(&["actualiza"])
        }
    }
}

const CORE_PACKAGES: &[&str] = &["linux", "linux-lts", "linux-zen", "linux-cachyos", "glibc", "systemd", "mesa", "nvidia", "grub", "pacman", "openssl", "mkinitcpio", "limine", "wayland", "hyprland"];

fn a_update(a: &mut Archito, _: &str) -> Response {
    let updates = probe::updates();
    let mut text = String::from("Actualizar Arch es una sola orden que sincroniza la lista de paquetes y pone todo al día.");
    let mut code = String::new();
    if let Some(list) = &updates {
        if list.is_empty() {
            return Response::new("Tu sistema ya está al día. No hay nada que actualizar.").learn("sudo pacman -Syu", "Es el comando de actualización. Ahora mismo no tendría nada que hacer.");
        }
        let core: Vec<&str> = list.iter().map(String::as_str).filter(|p| CORE_PACKAGES.iter().any(|c| p == c || p.starts_with(&format!("{c}-")))).collect();
        text.push_str(&format!("\n\nHay **{} paquetes** esperando.", list.len()));
        if core.is_empty() {
            text.push_str(" Ninguno es del núcleo del sistema: es una actualización tranquila.");
        } else {
            text.push_str(" Entre ellos hay piezas centrales, así que **reinicia al terminar** para empezar a usarlas:");
            code = core.join("  ");
        }
    }
    let action = a.propose("Actualizar el sistema", "sudo pacman -Syu", "Pacman te mostrará la lista completa y el espacio que ocupa antes de empezar. No apagues el equipo mientras trabaja.");
    let mut resp = Response::new(text).code(code);
    resp.actions.push(action);
    resp.chips(&["actualizaciones", "qué es rolling release"])
}

fn h_orphans(a: &mut Archito, _: &str) -> Response {
    let orphans = probe::orphans();
    if orphans.is_empty() {
        return Response::new("No tienes paquetes huérfanos: no hay dependencias sobrando.").learn("pacman -Qtdq", "Lista las dependencias que ya nadie usa. Sin salida significa que no hay ninguna.");
    }
    let action = a.propose(
        &format!("Quitar {} huérfano(s)", orphans.len()),
        &format!("sudo pacman -Rns {}", orphans.join(" ")),
        "Quita exactamente los paquetes de la lista. Si alguno te interesa, no confirmes: instálalo a mano y dejará de ser huérfano.",
    );
    let mut resp = Response::new(format!(
        "Tienes **{} paquete(s) huérfano(s)**: dependencias de programas que ya quitaste. No hacen daño, solo ocupan espacio.",
        orphans.len()
    ))
    .code(orphans.join("\n"))
    .learn("pacman -Qtdq", "Así los listas tú.");
    resp.actions.push(action);
    resp.chips(&["qué es un huérfano"])
}

fn h_cache(a: &mut Archito, _: &str) -> Response {
    let size = exec::capture(&["du", "-sh", "/var/cache/pacman/pkg"], std::time::Duration::from_secs(8)).and_then(|o| o.split_whitespace().next().map(str::to_string));
    let size_line = size.map(|s| format!("Ahora ocupa **{s}**.")).unwrap_or_default();
    let mut resp = Response::new(format!(
        "Pacman guarda cada paquete que descarga en su caché, por si necesitas volver a una versión anterior. Con el tiempo crece mucho. {size_line}"
    ))
    .learn("du -sh /var/cache/pacman/pkg", "Mide cuánto ocupa la caché de paquetes.");
    if exec::has_tool("paccache") {
        let action = a.propose("Limpiar versiones viejas", "sudo paccache -r", "Borra las versiones antiguas y conserva las tres más recientes de cada paquete, por si hay que volver atrás.");
        resp.actions.push(action);
    } else {
        resp.text.push_str("\n\nLa forma segura de limpiarla es `paccache`, que viene en el paquete pacman-contrib.");
        resp.chips.push("instala pacman-contrib".into());
    }
    resp
}

// ── Servicios y registro ─────────────────────────────────────────────────────

fn h_failed(a: &mut Archito, _: &str) -> Response {
    let failed = probe::failed_services();
    if failed.is_empty() {
        return Response::new("Todos los servicios están en pie. Systemd no reporta ninguno fallido.")
            .learn("systemctl --failed", "Lista los servicios que fallaron. Vacío es buena noticia.")
            .chips(&["errores", "qué es un servicio"]);
    }
    let first = failed[0].clone();
    let action = a.propose(&format!("Ver por qué falló {first}"), &format!("journalctl -b -n 30 --no-pager -u {first}"), "Lee las últimas líneas del registro de ese servicio. No cambia nada.");
    let mut resp = Response::new(format!("Hay **{} servicio(s) fallido(s)**. El motivo de cada uno queda en su registro:", failed.len()))
        .code(failed.join("\n"))
        .status(Status::Warn)
        .learn("systemctl --failed", "Así los listas tú.");
    resp.actions.push(action);
    resp.chips(&["errores", "qué es un servicio"])
}

fn h_errors(_: &mut Archito, _: &str) -> Response {
    match probe::recent_errors(10) {
        Some(out) => Response::new("Estos son los últimos errores desde que encendiste. Es normal que haya algunos; si uno no se entiende, pégamelo.")
            .code(out)
            .learn("journalctl -p err -b", "Muestra solo los errores del arranque actual.")
            .chips(&["servicios", "explica journalctl -p err -b"]),
        None => Response::new("No hay errores en el registro desde que encendiste (o tu usuario no tiene permiso para leerlo).")
            .learn("journalctl -p err -b", "Muestra solo los errores del arranque actual.")
            .chips(&["servicios"]),
    }
}

fn h_activity(_: &mut Archito, _: &str) -> Response {
    let rows = memory::recent_activity(10);
    if rows.is_empty() {
        return Response::new("Todavía no hay acciones registradas. Solo apunto lo que confirmas o cancelas; las conversaciones no se guardan.");
    }
    let code: Vec<String> = rows.iter().map(|(_, cmd, outcome)| format!("{outcome:<20} {cmd}")).collect();
    Response::new("Tus últimas acciones conmigo. Este registro vive solo en tu equipo.").code(code.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bot() -> Archito {
        Archito::with_memory(Memory::ephemeral())
    }

    /// Las frases que el motor anterior respondía mal por buscar subcadenas.
    #[test]
    fn frases_de_principiante() {
        let mut a = bot();
        assert!(a.process("mi sistema no arranca").text.contains("rescate"));
        assert!(a.process("como instalo un programa").text.contains("Cómo se instala"));
        assert!(a.process("se me rompio el grub").text.contains("rescate"));
        let r = a.process("ahora no tengo sonido");
        assert!(r.learn.is_some_and(|l| l.command == "wpctl status") || r.text.contains("PipeWire"));
        let r = a.process("no me funciona el wifi");
        assert!(r.code.contains("Interfaz conectada"));
    }

    #[test]
    fn explica_respetando_mayusculas() {
        let mut a = bot();
        let r = a.process("Explica sudo pacman -Syu");
        assert_eq!(r.steps.len(), 3);
        assert!(r.steps[2].doc.contains("actualiza todo"));
        // Un comando visto una vez cuenta una vez en la bitácora.
        assert_eq!(a.memory.seen.get("pacman"), Some(&1));
    }

    #[test]
    fn glosario_y_errores() {
        let mut a = bot();
        assert!(a.process("¿Qué es el AUR?").text.contains("comunidad"));
        let r = a.process("error: failed to init transaction (unable to lock database)");
        assert_eq!(r.status, Status::Warn);
        // El arreglo borra un archivo: tiene que llegar marcado como peligroso.
        assert_eq!(r.actions[0].level, Level::Danger);
    }

    #[test]
    fn las_acciones_se_ejecutan_por_numero() {
        let mut a = bot();
        let r = a.process("que es el kernel");
        let action = &r.actions[0];
        assert_eq!(action.level, Level::Safe);
        let out = a.execute(action.id);
        assert!(out.ok && out.mode == "inline", "{}", out.output);
        // Una vez usada, la acción deja de existir.
        assert!(!a.execute(action.id).ok);
        assert!(!a.execute(9999).ok);
    }

    #[test]
    fn lo_que_no_sabe_lo_dice() {
        let mut a = bot();
        assert!(a.process("cuéntame de la revolución francesa").text.contains("todavía no lo sé"));
    }
}
