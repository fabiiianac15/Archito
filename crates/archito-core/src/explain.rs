//! Archito Profesor: desarma un comando y explica cada pieza.
//!
//! Primero usa su propia base en español; si no conoce el programa, lee la
//! descripción de la página del manual instalada (`whatis`).

use crate::model::Step;
use crate::probe;

pub const COMMANDS: &[(&str, &str)] = &[
    ("sudo", "Ejecuta lo que sigue como administrador (root). Úsalo solo cuando haga falta: con sudo el sistema no te protege de ti."),
    ("pacman", "El gestor de paquetes de Arch. Instala, actualiza y quita programas del sistema."),
    ("yay", "Ayudante para el AUR: instala paquetes hechos por la comunidad, además de los oficiales."),
    ("paru", "Ayudante para el AUR, parecido a yay."),
    ("systemctl", "Controla los servicios de systemd: verlos, arrancarlos, pararlos o activarlos al inicio."),
    ("journalctl", "Lee el registro (los logs) del sistema que guarda systemd."),
    ("nmcli", "Maneja la red desde la terminal a través de NetworkManager."),
    ("ip", "Muestra y configura interfaces, direcciones y rutas de red."),
    ("ping", "Envía paquetes de prueba a un equipo para ver si responde."),
    ("lsblk", "Lista discos y particiones en forma de árbol."),
    ("df", "Muestra el espacio usado y libre de cada sistema de archivos montado."),
    ("du", "Calcula cuánto ocupa una carpeta."),
    ("free", "Muestra el uso de memoria RAM y swap."),
    ("ps", "Lista los procesos en ejecución."),
    ("top", "Monitor de procesos en tiempo real. Se sale con q."),
    ("htop", "Monitor de procesos más cómodo que top. Se sale con q."),
    ("btop", "Monitor de recursos con gráficas. Se sale con q."),
    ("uname", "Imprime datos del kernel que está corriendo."),
    ("uptime", "Tiempo que lleva encendido el equipo y su carga."),
    ("whoami", "Muestra con qué usuario estás trabajando."),
    ("hostname", "Muestra el nombre del equipo."),
    ("ls", "Lista los archivos y carpetas del directorio."),
    ("cd", "Cambia de directorio."),
    ("pwd", "Muestra en qué directorio estás."),
    ("mkdir", "Crea una carpeta."),
    ("touch", "Crea un archivo vacío o actualiza su fecha."),
    ("rm", "Borra archivos o carpetas. No hay papelera: lo borrado no vuelve."),
    ("cp", "Copia archivos o carpetas."),
    ("mv", "Mueve o renombra archivos y carpetas."),
    ("cat", "Imprime el contenido de un archivo."),
    ("less", "Muestra un archivo página por página. Se sale con q."),
    ("grep", "Busca texto dentro de archivos o de la salida de otro comando."),
    ("find", "Busca archivos por nombre, tamaño, fecha y más."),
    ("echo", "Imprime texto."),
    ("man", "Abre el manual de un comando. Se sale con q."),
    ("tldr", "Ejemplos cortos de uso de un comando."),
    ("chmod", "Cambia los permisos de un archivo o carpeta."),
    ("chown", "Cambia el dueño de un archivo o carpeta."),
    ("kill", "Termina un proceso por su número (PID)."),
    ("pkill", "Termina procesos por nombre."),
    ("hyprctl", "Controla Hyprland: ventanas, espacios de trabajo, monitores."),
    ("git", "Control de versiones: guarda el historial de cambios de un proyecto."),
    ("curl", "Descarga o envía datos por Internet."),
    ("wget", "Descarga archivos de Internet."),
    ("tar", "Empaqueta o desempaqueta archivos .tar."),
    ("date", "Muestra la fecha y la hora."),
    ("lspci", "Lista el hardware interno (tarjeta gráfica, red, sonido)."),
    ("lsusb", "Lista los dispositivos USB conectados."),
    ("sensors", "Muestra temperaturas y ventiladores (paquete lm_sensors)."),
    ("wpctl", "Controla el sonido de PipeWire: volumen, dispositivos, silencio."),
    ("bluetoothctl", "Maneja Bluetooth desde la terminal."),
    ("brightnessctl", "Cambia el brillo de la pantalla."),
    ("checkupdates", "Lista las actualizaciones pendientes sin tocar el sistema (paquete pacman-contrib)."),
    ("paccache", "Limpia versiones viejas de paquetes guardadas en la caché (paquete pacman-contrib)."),
    ("pacdiff", "Revisa los archivos .pacnew y te deja fusionarlos (paquete pacman-contrib)."),
    ("makepkg", "Construye un paquete a partir de un PKGBUILD. Así se instalan cosas del AUR a mano."),
    ("mkinitcpio", "Genera la imagen de arranque inicial (initramfs)."),
    ("grub-mkconfig", "Regenera el menú de arranque de GRUB."),
    ("arch-chroot", "Entra a un sistema Arch instalado desde un USB de rescate."),
    ("nano", "Editor de texto sencillo. Guarda con Ctrl+O y sale con Ctrl+X."),
    ("vim", "Editor de texto modal. Se sale con :q y Enter."),
    ("fastfetch", "Muestra un resumen del sistema con el logo de la distro."),
    ("systemd-analyze", "Mide cuánto tardó el arranque y qué lo retrasó."),
];

const PACMAN_FLAGS: &[(&str, &str)] = &[
    ("-S", "Sync: instala paquetes desde los repositorios."),
    ("-Syu", "Sincroniza la lista de paquetes (y) y actualiza todo el sistema (u). Es la forma correcta de actualizar Arch."),
    ("-Syyu", "Como -Syu, pero fuerza a descargar de nuevo la lista de paquetes."),
    ("-Sy", "Solo descarga la lista de paquetes. Usarlo sin la u puede dejar el sistema a medias: mejor -Syu."),
    ("-Ss", "Busca en los repositorios por nombre o descripción."),
    ("-Si", "Muestra la ficha de un paquete de los repositorios."),
    ("-Sc", "Borra de la caché los paquetes que ya no tienes instalados."),
    ("-Scc", "Vacía toda la caché de paquetes. Después no podrás volver a una versión anterior sin descargarla."),
    ("-Q", "Query: consulta los paquetes instalados."),
    ("-Qi", "Muestra la ficha de un paquete instalado."),
    ("-Ql", "Lista los archivos que instaló un paquete."),
    ("-Qo", "Dice a qué paquete pertenece un archivo."),
    ("-Qe", "Lista los paquetes que instalaste tú, sin las dependencias."),
    ("-Qq", "Lista los paquetes instalados, solo los nombres."),
    ("-Qtdq", "Lista los huérfanos: dependencias que ya nadie usa."),
    ("-Qdt", "Lista los huérfanos: dependencias que ya nadie usa."),
    ("-R", "Remove: quita un paquete."),
    ("-Rs", "Quita un paquete y las dependencias que queden sin uso."),
    ("-Rns", "Quita un paquete, sus dependencias sin uso (s) y sus archivos de configuración (n)."),
    ("-F", "Busca qué paquete trae un archivo o programa."),
    ("-U", "Instala un paquete desde un archivo local."),
    ("--needed", "No reinstala lo que ya está al día."),
    ("--noconfirm", "No pregunta antes de actuar. Mejor no usarlo mientras aprendes."),
];

const SYSTEMCTL_WORDS: &[(&str, &str)] = &[
    ("status", "Muestra el estado de un servicio y sus últimas líneas de log."),
    ("start", "Arranca un servicio ahora."),
    ("stop", "Detiene un servicio ahora."),
    ("restart", "Detiene y vuelve a arrancar un servicio."),
    ("reload", "Le pide al servicio releer su configuración sin detenerse."),
    ("enable", "Hace que el servicio arranque solo en cada inicio."),
    ("disable", "Quita el arranque automático del servicio."),
    ("--now", "Aplica el cambio también ahora mismo, no solo en el próximo arranque."),
    ("--user", "Trabaja con los servicios de tu usuario, no con los del sistema."),
    ("--failed", "Lista solo los servicios que fallaron."),
    ("list-units", "Lista las unidades cargadas."),
    ("is-active", "Responde si el servicio está corriendo."),
    ("daemon-reload", "Hace que systemd relea los archivos de unidad que cambiaste."),
];

const JOURNALCTL_FLAGS: &[(&str, &str)] = &[
    ("-p", "Filtra por prioridad. El valor que sigue (por ejemplo err) marca el mínimo."),
    ("-b", "Solo el arranque actual."),
    ("-u", "Solo los mensajes del servicio que se indica a continuación."),
    ("-f", "Se queda mostrando los mensajes nuevos en vivo. Se sale con Ctrl+C."),
    ("-n", "Muestra solo las últimas N líneas."),
    ("-e", "Salta al final del registro."),
    ("-x", "Añade explicaciones a los mensajes cuando existen."),
    ("-xe", "Salta al final (e) y añade explicaciones (x)."),
    ("-r", "Orden inverso: lo más nuevo primero."),
    ("--no-pager", "Imprime todo de una vez, sin el visor por páginas."),
    ("-k", "Solo los mensajes del kernel."),
];

const COMMON_FLAGS: &[(&str, &[(&str, &str)])] = &[
    ("ls", &[("-l", "Formato largo: permisos, dueño, tamaño y fecha."), ("-a", "Incluye los archivos ocultos (los que empiezan por punto)."), ("-h", "Tamaños legibles (K, M, G).")]),
    ("df", &[("-h", "Tamaños legibles (K, M, G)."), ("-T", "Añade el tipo de sistema de archivos.")]),
    ("du", &[("-s", "Solo el total, sin detallar cada subcarpeta."), ("-h", "Tamaños legibles (K, M, G).")]),
    ("free", &[("-h", "Tamaños legibles (M, G).")]),
    ("uname", &[("-r", "Solo la versión del kernel."), ("-a", "Toda la información disponible.")]),
    ("rm", &[("-r", "Recursivo: borra la carpeta con todo lo que tiene dentro."), ("-f", "Fuerza: no pregunta ni avisa de errores.")]),
    ("cp", &[("-r", "Recursivo: copia la carpeta con todo su contenido."), ("-a", "Copia conservando permisos, fechas y enlaces.")]),
    ("mkdir", &[("-p", "Crea también las carpetas intermedias que falten.")]),
    ("grep", &[("-i", "No distingue mayúsculas de minúsculas."), ("-r", "Busca en todos los archivos de la carpeta."), ("-n", "Muestra el número de línea.")]),
    ("ps", &[("aux", "Todos los procesos de todos los usuarios, con detalle.")]),
    ("ip", &[("addr", "Direcciones de cada interfaz de red."), ("a", "Direcciones de cada interfaz de red."), ("link", "Estado de las interfaces de red."), ("route", "Tabla de rutas: por dónde sale el tráfico.")]),
    ("chmod", &[("+x", "Da permiso de ejecución."), ("-R", "Recursivo: aplica a todo lo que haya dentro.")]),
    ("tar", &[("-xf", "Extrae (x) el archivo (f) que se indica."), ("-czf", "Crea (c) un archivo comprimido con gzip (z) con el nombre que sigue (f).")]),
    ("ping", &[("-c", "Número de paquetes a enviar antes de parar.")]),
];

const GENERIC_FLAGS: &[(&str, &str)] = &[
    ("-h", "Normalmente: ayuda corta, o tamaños legibles."),
    ("--help", "Muestra la ayuda del programa."),
    ("-v", "Normalmente: más detalle sobre lo que hace."),
    ("--version", "Muestra la versión del programa."),
];

const OPERATORS: &[(&str, &str)] = &[
    ("|", "Tubería: la salida del comando de la izquierda entra al de la derecha."),
    ("&&", "Ejecuta lo de la derecha solo si lo de la izquierda terminó bien."),
    ("||", "Ejecuta lo de la derecha solo si lo de la izquierda falló."),
    (";", "Separa dos comandos: el segundo corre pase lo que pase con el primero."),
    (">", "Guarda la salida en el archivo que sigue, borrando lo que tuviera."),
    (">>", "Añade la salida al final del archivo que sigue."),
    ("<", "Usa el archivo que sigue como entrada del comando."),
    ("&", "Deja el comando corriendo en segundo plano."),
];

fn lookup<'a>(table: &'a [(&str, &str)], key: &str) -> Option<&'a str> {
    table.iter().find(|(k, _)| *k == key).map(|(_, d)| *d)
}

/// Descripción en español de un programa, si Archito lo conoce.
pub fn known(program: &str) -> Option<&'static str> {
    lookup(COMMANDS, program)
}

/// Separa un comando en palabras respetando comillas y operadores.
pub fn tokenize(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), c) => cur.push(c),
            (None, '"' | '\'') => quote = Some(c),
            (None, c) if c.is_whitespace() => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            (None, '|' | '&' | '>' | ';' | '<') => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                let mut op = c.to_string();
                if matches!(c, '|' | '&' | '>') && chars.peek() == Some(&c) {
                    op.push(chars.next().unwrap());
                }
                out.push(op);
            }
            (None, c) => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Programa principal de un comando, saltando `sudo`.
pub fn main_program(line: &str) -> Option<String> {
    tokenize(line.trim_start_matches('$').trim())
        .into_iter()
        .find(|t| t != "sudo" && t != "doas")
        .map(|t| t.rsplit('/').next().unwrap_or(&t).to_string())
}

fn flag_doc(program: &str, flag: &str) -> Option<String> {
    let specific = match program {
        "pacman" | "yay" | "paru" => lookup(PACMAN_FLAGS, flag),
        "systemctl" => lookup(SYSTEMCTL_WORDS, flag),
        "journalctl" => lookup(JOURNALCTL_FLAGS, flag),
        _ => COMMON_FLAGS.iter().find(|(p, _)| *p == program).and_then(|(_, t)| lookup(t, flag)),
    };
    if let Some(doc) = specific.or_else(|| lookup(GENERIC_FLAGS, flag)) {
        return Some(doc.to_string());
    }
    // -la → -l y -a, cuando se conoce cada letra.
    if flag.starts_with('-') && !flag.starts_with("--") && flag.len() > 2 && !matches!(program, "pacman" | "yay" | "paru") {
        let parts: Vec<String> = flag[1..]
            .chars()
            .filter_map(|c| {
                let single = format!("-{c}");
                flag_doc(program, &single).map(|d| format!("{single}: {d}"))
            })
            .collect();
        if parts.len() == flag.len() - 1 {
            return Some(format!("Varias opciones juntas. {}", parts.join(" ")));
        }
    }
    None
}

/// Desglose educativo de una línea de comandos.
pub fn explain(line: &str) -> Vec<Step> {
    let tokens = tokenize(line.trim().trim_start_matches('$').trim());
    let mut steps = Vec::new();
    let mut program = String::new();
    let mut expect_program = true;
    let mut prev = String::new();
    for tok in tokens {
        let doc = if let Some(op) = lookup(OPERATORS, &tok) {
            expect_program = !matches!(tok.as_str(), ">" | ">>" | "<");
            op.to_string()
        } else if expect_program {
            let base = tok.rsplit('/').next().unwrap_or(&tok).to_string();
            let doc = match known(&base) {
                Some(d) => d.to_string(),
                None => match probe::whatis(&base) {
                    Some(d) => format!("Según su manual: {d}. Más detalle con `man {base}`."),
                    None => format!("No conozco este programa y no tiene manual instalado. Prueba `{base} --help`."),
                },
            };
            // Tras sudo viene el programa de verdad.
            expect_program = matches!(base.as_str(), "sudo" | "doas");
            program = base;
            doc
        } else if program == "journalctl" && prev == "-b" && tok.starts_with('-') && tok[1..].chars().all(|c| c.is_ascii_digit()) {
            "Cuenta arranques hacia atrás: -1 es el arranque anterior al actual.".to_string()
        } else if tok.starts_with('-') || matches!(program.as_str(), "systemctl" | "ip" | "ps") && flag_doc(&program, &tok).is_some() {
            flag_doc(&program, &tok).unwrap_or_else(|| format!("Opción de {program}. Búscala en `man {program}`."))
        } else {
            match (program.as_str(), prev.as_str()) {
                ("pacman" | "yay" | "paru", _) => format!("El paquete sobre el que actúa: {tok}."),
                ("systemctl", _) => format!("El servicio sobre el que actúa: {tok}."),
                ("journalctl", "-n") => "Cuántas líneas mostrar.".to_string(),
                ("wpctl", _) if tok == "status" => "Muestra dispositivos, salidas y programas que están sonando.".to_string(),
                ("journalctl", "-p") => "La prioridad mínima: err muestra errores y cosas peores.".to_string(),
                ("journalctl", "-u") => format!("El servicio cuyos mensajes quieres ver: {tok}."),
                ("man" | "tldr" | "whatis", _) => format!("El comando del que quieres leer: {tok}."),
                ("cd" | "ls" | "mkdir" | "rm" | "cp" | "mv" | "cat" | "less" | "du" | "nano" | "vim", _) => format!("La ruta sobre la que actúa: {tok}."),
                _ => "Un argumento: el dato con el que trabaja el comando.".to_string(),
            }
        };
        prev = tok.clone();
        steps.push(Step { token: tok, doc });
    }
    steps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desarma_pacman() {
        let s = explain("sudo pacman -Syu");
        assert_eq!(s.len(), 3);
        assert_eq!(s[0].token, "sudo");
        assert!(s[1].doc.contains("gestor de paquetes"));
        assert!(s[2].doc.contains("actualiza todo"));
    }

    #[test]
    fn entiende_tuberias_y_paquetes() {
        let s = explain("pacman -Qq | grep -i fire");
        assert_eq!(s.iter().map(|x| x.token.as_str()).collect::<Vec<_>>(), ["pacman", "-Qq", "|", "grep", "-i", "fire"]);
        assert!(s[2].doc.starts_with("Tubería"));
        assert!(s[4].doc.contains("mayúsculas"));
        let s = explain("sudo pacman -S --needed firefox");
        assert!(s[4].doc.contains("paquete"));
    }

    #[test]
    fn junta_flags_cortos() {
        let s = explain("ls -la");
        assert!(s[1].doc.contains("-l:") && s[1].doc.contains("-a:"));
    }

    #[test]
    fn programa_principal() {
        assert_eq!(main_program("sudo pacman -Syu").as_deref(), Some("pacman"));
        assert_eq!(main_program("$ /usr/bin/free -h").as_deref(), Some("free"));
    }
}
