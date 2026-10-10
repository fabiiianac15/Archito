//! Traductor de errores: convierte un mensaje que asusta en qué pasó, por qué
//! y qué hacer. Es la parte de Archito pensada para el momento exacto en que
//! un principiante se rinde.

use crate::explain::tokenize;
use crate::probe;

pub struct ErrorHelp {
    pub title: &'static str,
    pub why: String,
    pub fix: Vec<String>,
    /// Comando que arregla o investiga el problema, si hay uno claro.
    pub command: Option<String>,
}

struct Pattern {
    /// Todas las cadenas tienen que aparecer (en minúsculas).
    needles: &'static [&'static str],
    title: &'static str,
    why: &'static str,
    fix: &'static [&'static str],
    command: Option<&'static str>,
}

const PATTERNS: &[Pattern] = &[
    Pattern {
        needles: &["unable to lock database"],
        title: "Pacman encontró su base de datos bloqueada",
        why: "Pacman deja un archivo de bloqueo mientras trabaja para que no corran dos instalaciones a la vez. Si una se cerró a la fuerza, el bloqueo se queda puesto.",
        fix: &[
            "Comprueba que no haya otra instalación o actualización abierta (otra terminal, una tienda de software).",
            "Si no hay ninguna, borra el bloqueo con el comando de abajo y repite lo que estabas haciendo.",
        ],
        command: Some("sudo rm /var/lib/pacman/db.lck"),
    },
    Pattern {
        needles: &["target not found"],
        title: "Pacman no encontró ese paquete",
        why: "El nombre no existe en los repositorios oficiales. Suele ser un nombre mal escrito, un paquete que se llama distinto en Arch o uno que solo está en el AUR.",
        fix: &[
            "Búscalo por una palabra: dime \"busca <palabra>\" o usa `pacman -Ss <palabra>`.",
            "Si no aparece, puede estar en el AUR: se instala con un ayudante como yay o paru.",
        ],
        command: None,
    },
    Pattern {
        needles: &["conflicting files"],
        title: "Dos paquetes quieren poner el mismo archivo",
        why: "Pacman se niega a sobrescribir un archivo que ya existe y que no pertenece al paquete que instalas. Nada cambió en tu sistema: la operación se canceló entera.",
        fix: &[
            "Mira a quién pertenece el archivo con `pacman -Qo /ruta/del/archivo`.",
            "Si no pertenece a nadie (lo creaste tú o quedó de una instalación manual), muévelo a otro sitio y repite.",
            "No uses --overwrite sin entender de dónde salió el archivo.",
        ],
        command: None,
    },
    Pattern {
        needles: &["signature", "trust"],
        title: "Pacman no pudo verificar la firma de un paquete",
        why: "Cada paquete viene firmado. Tu llavero de claves está desactualizado y no reconoce a quien firmó este.",
        fix: &["Actualiza primero el llavero con el comando de abajo.", "Después repite la actualización completa con `sudo pacman -Syu`."],
        command: Some("sudo pacman -Sy archlinux-keyring"),
    },
    Pattern {
        needles: &["invalid or corrupted package"],
        title: "Un paquete descargado llegó dañado o sin firma válida",
        why: "La descarga se cortó o el llavero de claves está viejo. Pacman lo detectó antes de instalar nada.",
        fix: &["Actualiza el llavero con el comando de abajo.", "Si sigue fallando, borra el archivo dañado de /var/cache/pacman/pkg/ y vuelve a intentar."],
        command: Some("sudo pacman -Sy archlinux-keyring"),
    },
    Pattern {
        needles: &["could not satisfy dependencies"],
        title: "Falta algo que el paquete necesita",
        why: "El paquete depende de otro que no se puede instalar, casi siempre porque el sistema está a medio actualizar.",
        fix: &["Actualiza el sistema completo y vuelve a intentar: en Arch las actualizaciones parciales rompen dependencias."],
        command: Some("sudo pacman -Syu"),
    },
    Pattern {
        needles: &["failed retrieving file"],
        title: "Pacman no pudo descargar un archivo",
        why: "El servidor (mirror) del que descargas no respondió o tiene una copia vieja.",
        fix: &["Comprueba tu conexión: dime \"internet\".", "Refresca la lista de paquetes y repite con el comando de abajo."],
        command: Some("sudo pacman -Syyu"),
    },
    Pattern {
        needles: &["no space left on device"],
        title: "El disco se llenó",
        why: "No queda espacio en la partición donde se estaba escribiendo.",
        fix: &["Mira cuánto queda: dime \"disco\".", "Lo que más espacio suele recuperar es la caché de paquetes: dime \"limpia la caché\"."],
        command: Some("df -h"),
    },
    Pattern {
        needles: &["could not resolve host"],
        title: "No se pudo traducir el nombre del servidor",
        why: "Falla el DNS o no hay conexión a Internet.",
        fix: &["Dime \"no tengo internet\" y reviso la red paso a paso."],
        command: None,
    },
    Pattern {
        needles: &["temporary failure in name resolution"],
        title: "No se pudo traducir el nombre del servidor",
        why: "Falla el DNS o no hay conexión a Internet.",
        fix: &["Dime \"no tengo internet\" y reviso la red paso a paso."],
        command: None,
    },
    Pattern {
        needles: &["you cannot perform this operation unless you are root"],
        title: "Esa operación necesita permisos de administrador",
        why: "Instalar, quitar y actualizar paquetes cambia archivos del sistema, y eso solo lo puede hacer root.",
        fix: &["Repite el mismo comando con `sudo` delante. Te pedirá tu contraseña (no se ve mientras la escribes, es normal)."],
        command: None,
    },
    Pattern {
        needles: &["permission denied"],
        title: "No tienes permiso para eso",
        why: "El archivo o la acción pertenece a otro usuario, normalmente a root.",
        fix: &[
            "Si es un archivo del sistema (/etc, /usr, /var), repite con `sudo` delante.",
            "Si es un programa tuyo que no arranca, puede faltarle el permiso de ejecución: `chmod +x archivo`.",
            "Mira los permisos con `ls -l archivo` antes de cambiarlos.",
        ],
        command: None,
    },
    Pattern {
        needles: &["is not in the sudoers file"],
        title: "Tu usuario no puede usar sudo",
        why: "Sudo solo funciona para los usuarios del grupo wheel, y el tuyo no está.",
        fix: &["Entra como root con `su -` y añade tu usuario: `usermod -aG wheel tu_usuario`.", "Cierra la sesión y vuelve a entrar para que el cambio aplique."],
        command: None,
    },
    Pattern {
        needles: &["unit", "not found"],
        title: "Ese servicio no existe con ese nombre",
        why: "Systemd no tiene ninguna unidad llamada así. O el nombre es otro, o el paquete que la trae no está instalado.",
        fix: &["Busca el nombre real con `systemctl list-unit-files | grep <palabra>`."],
        command: None,
    },
    Pattern {
        needles: &["failed to start"],
        title: "Un servicio no pudo arrancar",
        why: "El servicio intentó iniciar y terminó con error. El motivo exacto queda en su registro.",
        fix: &["Mira qué servicios fallaron: dime \"servicios fallidos\".", "Lee el motivo con `journalctl -xeu <servicio>`."],
        command: Some("systemctl --failed"),
    },
    Pattern {
        needles: &["no such file or directory"],
        title: "Esa ruta no existe",
        why: "El archivo o la carpeta no está donde el comando la buscó. Linux distingue mayúsculas de minúsculas: Documentos y documentos son carpetas distintas.",
        fix: &["Comprueba dónde estás con `pwd` y qué hay ahí con `ls`.", "Usa Tab para completar rutas: evita los errores de escritura."],
        command: None,
    },
];

/// Reconoce un error conocido dentro de un texto pegado por el usuario.
pub fn from_text(text: &str) -> Option<ErrorHelp> {
    let low = text.to_lowercase();
    if let Some(help) = command_not_found(&low) {
        return Some(help);
    }
    PATTERNS.iter().find(|p| p.needles.iter().all(|n| low.contains(n))).map(|p| ErrorHelp {
        title: p.title,
        why: p.why.to_string(),
        fix: p.fix.iter().map(|s| s.to_string()).collect(),
        command: p.command.map(str::to_string),
    })
}

fn command_not_found(low: &str) -> Option<ErrorHelp> {
    let name = if let Some((before, _)) = low.split_once(": command not found") {
        before.rsplit([' ', ':']).next()?.to_string()
    } else if let Some((_, after)) = low.split_once("unknown command:") {
        after.split_whitespace().next()?.to_string()
    } else if let Some((_, after)) = low.split_once("command not found:") {
        after.split_whitespace().next()?.to_string()
    } else {
        return None;
    };
    Some(missing_program(name.trim_matches(['\'', '"', '`'])))
}

fn missing_program(name: &str) -> ErrorHelp {
    let mut help = ErrorHelp {
        title: "Ese programa no está instalado (o está mal escrito)",
        why: format!("La shell buscó `{name}` en las carpetas de programas y no lo encontró."),
        fix: Vec::new(),
        command: None,
    };
    match probe::package_providing(name) {
        Some(pkg) => {
            help.fix.push(format!("Lo trae el paquete **{pkg}**. Instálalo con el comando de abajo."));
            help.command = Some(format!("sudo pacman -S --needed {pkg}"));
        }
        None => {
            help.fix.push("Revisa que esté bien escrito: usa Tab para autocompletar.".into());
            help.fix.push(format!("Búscalo en los repositorios: dime \"busca {name}\"."));
        }
    }
    help
}

/// Explica por qué pudo fallar el último comando de la terminal, a partir del
/// comando y su código de salida (lo que guarda el hook de la shell).
pub fn from_failure(command: &str, status: i32) -> ErrorHelp {
    let tokens = tokenize(command);
    let has_sudo = tokens.first().is_some_and(|t| t == "sudo" || t == "doas");
    let program = tokens.iter().find(|t| *t != "sudo" && *t != "doas").cloned().unwrap_or_default();
    let args: Vec<&str> = tokens.iter().skip_while(|t| **t != program).skip(1).map(String::as_str).collect();
    match status {
        127 => return missing_program(&program),
        126 => {
            return ErrorHelp {
                title: "El archivo existe pero no se puede ejecutar",
                why: "Le falta el permiso de ejecución o no es un programa.".into(),
                fix: vec![format!("Dale permiso con `chmod +x {program}` si confías en él.")],
                command: None,
            }
        }
        130 => {
            return ErrorHelp {
                title: "Lo interrumpiste con Ctrl+C",
                why: "El código 130 significa que el programa recibió la señal de interrupción. No es un fallo.".into(),
                fix: vec!["Si no fue a propósito, vuelve a ejecutarlo.".into()],
                command: None,
            }
        }
        137 => {
            return ErrorHelp {
                title: "El sistema mató el proceso",
                why: "El código 137 suele indicar que se acabó la memoria y el kernel eligió ese proceso para liberar.".into(),
                fix: vec!["Mira el uso de memoria: dime \"ram\".".into(), "Busca el aviso en el registro del kernel.".into()],
                command: Some("journalctl -k -b -n 20".into()),
            }
        }
        _ => {}
    }
    let needs_root = match program.as_str() {
        "pacman" => args.first().is_some_and(|f| f.starts_with("-S") && !matches!(*f, "-Ss" | "-Si") || f.starts_with("-R") || f.starts_with("-U")),
        "systemctl" => !args.contains(&"--user") && args.iter().any(|a| matches!(*a, "start" | "stop" | "restart" | "enable" | "disable")),
        _ => false,
    };
    if needs_root && !has_sudo {
        return ErrorHelp {
            title: "Faltó sudo",
            why: format!("`{program}` iba a cambiar archivos del sistema y lo ejecutaste como usuario normal."),
            fix: vec!["Repite el comando con sudo delante.".into()],
            command: Some(format!("sudo {command}")),
        };
    }
    match program.as_str() {
        "pacman" | "yay" | "paru" => ErrorHelp {
            title: "Pacman terminó con error",
            why: "El motivo exacto está en las líneas que empiezan por \"error:\" en la terminal.".into(),
            fix: vec![
                "Copia esa línea y pégamela: la mayoría las reconozco.".into(),
                "Las causas más comunes son un nombre de paquete que no existe, el sistema a medio actualizar y la base de datos bloqueada.".into(),
            ],
            command: None,
        },
        "systemctl" => {
            let unit = args.iter().rev().find(|a| !a.starts_with('-') && !matches!(**a, "start" | "stop" | "restart" | "enable" | "disable" | "status")).copied();
            ErrorHelp {
                title: "Systemd no pudo completar la orden",
                why: "El servicio falló o el nombre no existe. En `status`, un código 3 solo significa que el servicio está parado.".into(),
                fix: vec!["El registro del servicio dice por qué.".into()],
                command: unit.map(|u| format!("journalctl -b -n 30 -u {u}")),
            }
        }
        "cd" | "ls" | "cat" | "cp" | "mv" => ErrorHelp {
            title: "Problema con una ruta",
            why: "Lo normal es que la ruta no exista, esté mal escrita o no tengas permiso para leerla.".into(),
            fix: vec!["Comprueba dónde estás con `pwd` y qué hay con `ls`.".into(), "Recuerda que las mayúsculas cuentan.".into()],
            command: None,
        },
        "grep" if status == 1 => ErrorHelp {
            title: "grep no encontró coincidencias",
            why: "En grep el código 1 significa \"no hay resultados\". No es un fallo.".into(),
            fix: vec!["Prueba con -i para ignorar mayúsculas.".into()],
            command: None,
        },
        _ => ErrorHelp {
            title: "El comando terminó con error",
            why: format!("`{program}` devolvió el código {status}. Cada programa decide qué significan sus códigos; cualquier valor distinto de 0 es un fallo."),
            fix: vec![
                "Pégame el mensaje de error que salió en la terminal y lo traduzco.".into(),
                format!("El manual suele listar los códigos: `man {program}`."),
            ],
            command: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconoce_errores_de_pacman() {
        assert!(from_text("error: failed to init transaction (unable to lock database)").unwrap().title.contains("bloqueada"));
        assert!(from_text("error: target not found: fierfox").unwrap().title.contains("no encontró"));
        assert!(from_text("error: you cannot perform this operation unless you are root.").unwrap().title.contains("administrador"));
        assert!(from_text("hola, cómo estás").is_none());
    }

    #[test]
    fn un_programa_inexistente_no_inventa_paquete() {
        let h = from_failure("zzz-no-existe-zzz", 127);
        assert!(h.command.is_none(), "{:?}", h.command);
    }

    #[test]
    fn falta_sudo() {
        let h = from_failure("pacman -S firefox", 1);
        assert_eq!(h.title, "Faltó sudo");
        assert_eq!(h.command.as_deref(), Some("sudo pacman -S firefox"));
        assert_ne!(from_failure("pacman -Ss firefox", 1).title, "Faltó sudo");
    }

    #[test]
    fn codigos_especiales() {
        assert!(from_failure("sleep 100", 130).title.contains("Ctrl+C"));
        assert!(from_failure("grep hola archivo", 1).title.contains("coincidencias"));
    }
}
