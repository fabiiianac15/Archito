//! Archito en la terminal.
//!
//!     archito                     conversación
//!     archito cuánta ram tengo    una pregunta y sale
//!     archito porque              explica el último comando que falló
//!     archito camino              tus misiones
//!     archito comprobar           comprueba la misión en curso
//!     archito hook [--install]    integración con tu shell

use std::io::{self, BufRead, IsTerminal, Write};

use archito_core::lessons::PathView;
use archito_core::text::bar;
use archito_core::{exec, memory, Archito, Level, Response, Status};

const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";

/// **negrita** y `código` a colores de terminal.
fn inline(text: &str) -> String {
    let mut out = String::new();
    let mut bold = false;
    let mut code = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '*' if chars.peek() == Some(&'*') => {
                chars.next();
                bold = !bold;
                out.push_str(if bold { BOLD } else { RESET });
            }
            '`' => {
                code = !code;
                out.push_str(if code { CYAN } else { RESET });
            }
            c => out.push(c),
        }
    }
    out.push_str(RESET);
    out
}

fn print_response(resp: &Response) {
    let color = match resp.status {
        Status::Warn => YELLOW,
        Status::Error => RED,
        _ => "",
    };
    println!();
    for line in resp.text.lines() {
        match line.strip_prefix("> ") {
            Some(quote) => println!("  {DIM}│ {}{RESET}", inline(quote)),
            None => println!("  {color}{}{RESET}", inline(line)),
        }
    }
    for s in &resp.stats {
        let c = match s.severity {
            "danger" => RED,
            "warn" => YELLOW,
            _ => GREEN,
        };
        println!("\n  {:<12} {c}{}{RESET} {}", s.label, bar(s.pct, 20), s.value);
    }
    if !resp.code.is_empty() {
        println!();
        for line in resp.code.lines() {
            println!("    {DIM}{line}{RESET}");
        }
    }
    if !resp.steps.is_empty() {
        println!();
        for step in &resp.steps {
            println!("    {CYAN}{BOLD}{}{RESET}", step.token);
            println!("    {DIM}└─{RESET} {}", inline(&step.doc));
        }
    }
    if let Some(learn) = &resp.learn {
        println!("\n  {GREEN}Así lo harías tú:{RESET} {BOLD}{}{RESET}", learn.command);
        println!("  {DIM}{}{RESET}", learn.note);
    }
    println!();
}

fn ask(prompt: &str) -> String {
    print!("{prompt}");
    let _ = io::stdout().flush();
    let mut line = String::new();
    let _ = io::stdin().lock().read_line(&mut line);
    line.trim().to_lowercase()
}

/// Ofrece cada acción de la respuesta. Nada corre sin un "s" explícito.
fn offer_actions(bot: &mut Archito, resp: &Response) {
    let interactive = io::stdin().is_terminal() && io::stdout().is_terminal();
    for action in &resp.actions {
        let (tag, color) = match action.level {
            Level::Safe => ("solo lee", GREEN),
            Level::Confirm => ("cambia el sistema", YELLOW),
            Level::Danger => ("PELIGROSO", RED),
        };
        println!("  {BOLD}{}{RESET}  {color}[{tag}]{RESET}", action.label);
        println!("    {CYAN}$ {}{RESET}", action.command);
        if !action.description.is_empty() {
            println!("    {DIM}{}{RESET}", action.description);
        }
        if !action.danger_note.is_empty() {
            println!("    {RED}{}{RESET}", action.danger_note);
        }
        if !interactive {
            bot.cancel(action.id);
            println!();
            continue;
        }
        let answer = if action.level == Level::Danger {
            ask(&format!("  Para ejecutarlo escribe {BOLD}si, ejecutar{RESET} (cualquier otra cosa cancela): "))
        } else {
            ask(&format!("  ¿Lo ejecuto? [{BOLD}s{RESET}/N] "))
        };
        let yes = if action.level == Level::Danger { answer == "si, ejecutar" || answer == "sí, ejecutar" } else { matches!(answer.as_str(), "s" | "si" | "sí" | "y") };
        if !yes {
            bot.cancel(action.id);
            println!("  {DIM}Cancelado. No se ejecutó nada.{RESET}\n");
            continue;
        }
        let Some(taken) = bot.take_action(action.id) else { continue };
        println!();
        let code = exec::run_here(&taken.command);
        memory::log_activity(&taken.command, if code == 0 { "ejecutado" } else { "falló" });
        if code == 0 {
            println!("\n  {GREEN}Listo.{RESET}\n");
        } else {
            println!("\n  {RED}Terminó con el código {code}.{RESET}");
            let help = archito_core::errors::from_failure(&taken.command, code);
            println!("  {BOLD}{}{RESET}\n  {}\n", help.title, inline(&help.why));
        }
    }
}

fn respond(bot: &mut Archito, input: &str) -> bool {
    let resp = match archito_core::text::norm(input).as_str() {
        "porque" | "por que" | "why" => bot.why(),
        _ => bot.process(input),
    };
    print_response(&resp);
    offer_actions(bot, &resp);
    if !resp.chips.is_empty() && io::stdout().is_terminal() {
        println!("  {DIM}Puedes seguir con: {}{RESET}\n", resp.chips.join(" · "));
    }
    resp.status != Status::Bye
}

fn print_path(view: &PathView) {
    let next = view.next_xp.map(|n| format!(" de {n}")).unwrap_or_default();
    println!("\n  {BOLD}Nivel {} · {}{RESET}   {} XP{next}   {} de {} misiones\n", view.level, view.level_name, view.xp, view.done, view.total);
    let mut stage = "";
    for m in &view.missions {
        if m.stage != stage {
            stage = m.stage;
            println!("  {DIM}── {stage} ──{RESET}");
        }
        let (icon, color) = match m.state {
            "done" => ("✔", GREEN),
            "active" => ("▶", CYAN),
            _ => ("○", ""),
        };
        println!("  {color}{icon} {}{RESET}  {DIM}+{} XP{RESET}", m.title, m.xp);
    }
    if !view.hook {
        println!("\n  {DIM}Consejo: con `archito hook --install` compruebo tus misiones al instante.{RESET}");
    }
    println!("\n  {DIM}Escribe `archito camino` sin más para ver los pasos de la siguiente, y `archito comprobar` cuando la hagas.{RESET}\n");
}

fn hook(args: &[String]) {
    let shell = args.iter().find(|a| !a.starts_with('-')).cloned().or_else(|| std::env::var("SHELL").ok().and_then(|s| s.rsplit('/').next().map(str::to_string))).unwrap_or_default();
    let install = args.iter().any(|a| a == "--install");
    let script = match shell.as_str() {
        "fish" => archito_core::HOOK_FISH,
        "zsh" => archito_core::HOOK_ZSH,
        "bash" => archito_core::HOOK_BASH,
        other => {
            eprintln!("No tengo hook para la shell «{other}». Disponibles: fish, zsh, bash.");
            std::process::exit(2);
        }
    };
    if !install {
        print!("{script}");
        return;
    }
    let home = memory::home_dir();
    let (path, rc) = match shell.as_str() {
        "fish" => (home.join(".config/fish/conf.d/archito.fish"), None),
        "zsh" => (memory::config_dir().join("archito.zsh"), Some(home.join(".zshrc"))),
        _ => (memory::config_dir().join("archito.bash"), Some(home.join(".bashrc"))),
    };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Err(e) = std::fs::write(&path, script) {
        eprintln!("No pude escribir {}: {e}", path.display());
        std::process::exit(1);
    }
    println!("Hook de {shell} instalado en {}", path.display());
    match rc {
        None => println!("Abre una terminal nueva y ya queda activo."),
        // El archivo de inicio es del usuario: Archito no lo edita.
        Some(rc) => println!("Añade esta línea a {} y abre una terminal nueva:\n\n  source {}", rc.display(), path.display()),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--version" | "-V") => return println!("archito {}", archito_core::VERSION),
        Some("--help" | "-h") => {
            return println!(
                "Archito {}: tu compañero para aprender Arch Linux\n\n  archito                    conversación\n  archito <pregunta>         una pregunta y sale\n  archito explica <comando>  desarma un comando\n  archito porque             explica el último comando que falló\n  archito camino             tus misiones y tu nivel\n  archito comprobar          comprueba la misión en curso\n  archito hook [--install]   integración con fish, zsh o bash\n  archito --json <pregunta>  respuesta en JSON",
                archito_core::VERSION
            )
        }
        Some("hook") => return hook(&args[1..]),
        _ => {}
    }
    let mut bot = Archito::new();
    if args.first().is_some_and(|a| a == "--json") {
        let resp = bot.process(&args[1..].join(" "));
        return println!("{}", serde_json::to_string_pretty(&resp).unwrap_or_default());
    }
    if args.first().is_some_and(|a| a == "camino") && args.len() == 2 && args[1] == "todo" {
        return print_path(&bot.path());
    }
    if !args.is_empty() {
        respond(&mut bot, &args.join(" "));
        return;
    }
    let welcome = bot.welcome();
    print_response(&welcome);
    let stdin = io::stdin();
    loop {
        print!("{CYAN}archito ›{RESET} ");
        let _ = io::stdout().flush();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 {
            println!();
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if matches!(line, "salir" | "exit" | "q") || !respond(&mut bot, line) {
            break;
        }
    }
}
