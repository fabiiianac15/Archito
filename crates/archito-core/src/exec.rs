//! Ejecución de comandos: lecturas dentro del chat y cambios en una terminal
//! visible, donde el usuario ve todo lo que pasa y escribe su contraseña.

use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::model::Level;
use crate::safety::classify;

const MAX_LINES: usize = 40;

/// Quita variables que rompen a los procesos hijos (preload de layer-shell).
pub fn clean(cmd: &mut Command) -> &mut Command {
    cmd.env_remove("LD_PRELOAD").env_remove("GTK_MODULES").env_remove("GDK_BACKEND")
}

pub fn find_on_path(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).map(|d| d.join(name)).find(|p| p.is_file())
}

pub fn has_tool(name: &str) -> bool {
    find_on_path(name).is_some()
}

/// Ejecuta `argv` sin shell y devuelve su salida estándar, o `None` si falta
/// el programa, tarda demasiado o no imprime nada. Los mensajes de error del
/// programa nunca se devuelven como si fueran datos.
pub fn capture(argv: &[&str], timeout: Duration) -> Option<String> {
    let (out, _ok) = capture_status(argv, timeout)?;
    let out = out.trim().to_string();
    (!out.is_empty()).then_some(out)
}

/// Como `capture`, pero devuelve también si el programa terminó bien.
pub fn capture_status(argv: &[&str], timeout: Duration) -> Option<(String, bool)> {
    let (prog, args) = argv.split_first()?;
    let mut cmd = Command::new(prog);
    cmd.args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    // Salida en inglés estable para poder leerla; los textos al usuario son nuestros.
    cmd.env("LC_ALL", "C");
    let mut child = clean(&mut cmd).spawn().ok()?;
    let mut stdout = child.stdout.take()?;
    let mut stderr = child.stderr.take()?;
    let reader = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = stdout.read_to_string(&mut s);
        s
    });
    let err_reader = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = stderr.read_to_string(&mut s);
        s
    });
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait().ok()? {
            Some(status) => break status,
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            None => std::thread::sleep(Duration::from_millis(15)),
        }
    };
    let out = reader.join().ok()?;
    // stderr se lee para que el programa no se bloquee al escribir, y se descarta.
    let _ = err_reader.join();
    Some((out, status.success()))
}

fn trim_lines(text: &str) -> String {
    let lines: Vec<&str> = text.trim_end().lines().collect();
    if lines.len() <= MAX_LINES {
        return lines.join("\n");
    }
    let hidden = lines.len() - (MAX_LINES - 1);
    format!("{}\n… (+{hidden} líneas más; ejecútalo en tu terminal para verlo entero)", lines[..MAX_LINES - 1].join("\n"))
}

/// Ejecuta un comando dentro del chat. Solo acepta los de nivel `Safe`.
pub fn run_inline(command: &str) -> Result<String, String> {
    if classify(command) != Level::Safe {
        return Err("Ese comando no es de solo lectura: va por terminal y con confirmación.".into());
    }
    let argv: Vec<&str> = command.split_whitespace().collect();
    let Some(prog) = argv.first() else { return Err("Comando vacío.".into()) };
    if !has_tool(prog) {
        return Err(format!("No tienes instalado `{prog}`."));
    }
    let mut cmd = Command::new(prog);
    cmd.args(&argv[1..]).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let child = clean(&mut cmd).spawn().map_err(|e| e.to_string())?;
    // Las lecturas de la lista son cortas; un `wait` simple basta.
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    if out.status.success() {
        let text = trim_lines(&stdout);
        Ok(if text.is_empty() { "(sin salida)".into() } else { text })
    } else {
        Err(trim_lines(if stderr.trim().is_empty() { &stdout } else { &stderr }))
    }
}

const TERMINALS: &[(&str, &[&str])] = &[
    ("kitty", &["--title", "Archito"]),
    ("ghostty", &["--title=Archito", "-e"]),
    ("alacritty", &["--title", "Archito", "-e"]),
    ("foot", &["--title", "Archito"]),
    ("wezterm", &["start", "--"]),
    ("konsole", &["-e"]),
    ("gnome-terminal", &["--"]),
    ("xfce4-terminal", &["-x"]),
    ("xterm", &["-e"]),
];

fn find_terminal() -> Option<(String, Vec<String>)> {
    if let Ok(pref) = std::env::var("TERMINAL") {
        let name = pref.split_whitespace().next().unwrap_or("").to_string();
        if has_tool(&name) {
            let args = TERMINALS.iter().find(|(t, _)| *t == name).map(|(_, a)| a.to_vec()).unwrap_or(vec!["-e"]);
            return Some((name, args.iter().map(|s| s.to_string()).collect()));
        }
    }
    TERMINALS
        .iter()
        .find(|(t, _)| has_tool(t))
        .map(|(t, a)| (t.to_string(), a.iter().map(|s| s.to_string()).collect()))
}

/// Guion de la terminal. El comando llega como `$1`, nunca pegado dentro del
/// guion, así un `%` o unas comillas no cambian lo que se imprime.
const TERMINAL_SCRIPT: &str = r#"printf '\033[1;36m$ %s\033[0m\n\n' "$1"
eval "$1"
status=$?
echo
if [ "$status" -ne 0 ]; then
  printf '\033[1;31m── Terminó con error (%s) ──\033[0m\n' "$status"
  printf 'Escribe \033[1marchito porque\033[0m y te explico qué pasó.\n'
else
  printf '\033[1;32m── Listo ──\033[0m\n'
fi
printf 'Enter para seguir en esta terminal…'
read -r _ || true
exec "${SHELL:-bash}""#;

/// Abre el comando en una terminal externa. Devuelve `false` si no hay ninguna.
pub fn open_in_terminal(command: &str) -> bool {
    let Some((term, prefix)) = find_terminal() else { return false };
    let mut cmd = Command::new(term);
    cmd.args(prefix).args(["bash", "-c", TERMINAL_SCRIPT, "archito", command]);
    cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    clean(&mut cmd).spawn().is_ok()
}

/// Ejecuta el comando en la terminal actual, con su entrada y salida. Lo usa
/// la CLI después de que el usuario confirma. Devuelve el código de salida.
pub fn run_here(command: &str) -> i32 {
    let mut cmd = Command::new("bash");
    cmd.args(["-c", command]);
    clean(&mut cmd).status().ok().and_then(|s| s.code()).unwrap_or(1)
}
