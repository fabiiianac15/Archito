"""
Explicador de comandos — FASE 9 (Archito Profesor).

Descompone un comando en sus partes y explica qué hace cada una.
Funciona sin IA: usa una base de conocimiento local con los comandos
y flags más comunes de Arch Linux.
"""

import shlex

COMMANDS = {
    "sudo": "Ejecuta el comando siguiente con privilegios de administrador (root). Úsalo solo cuando sea necesario.",
    "pacman": "El gestor de paquetes de Arch Linux. Instala, actualiza y elimina software del sistema.",
    "systemctl": "Controla los servicios (demonios) de systemd: arrancarlos, pararlos, ver su estado...",
    "journalctl": "Lee los registros (logs) del sistema gestionados por systemd.",
    "nmcli": "Herramienta de línea de comandos para gestionar la red (NetworkManager).",
    "ip": "Muestra y configura interfaces, direcciones y rutas de red.",
    "lsblk": "Lista todos los discos y particiones en forma de árbol.",
    "df": "Muestra el espacio usado y libre de los sistemas de archivos montados.",
    "free": "Muestra el uso de memoria RAM y swap.",
    "ps": "Lista los procesos en ejecución.",
    "top": "Monitor interactivo de procesos en tiempo real.",
    "htop": "Monitor interactivo de procesos, versión mejorada de top.",
    "uname": "Imprime información del kernel y la arquitectura.",
    "uptime": "Tiempo que lleva el sistema encendido y carga promedio.",
    "whoami": "Muestra el nombre del usuario actual.",
    "who": "Lista los usuarios conectados al sistema.",
    "hostname": "Muestra el nombre del equipo en la red.",
    "ls": "Lista los archivos y carpetas del directorio actual.",
    "cd": "Cambia de directorio.",
    "pwd": "Muestra la ruta del directorio actual.",
    "mkdir": "Crea una carpeta nueva.",
    "rm": "ELIMINA archivos o carpetas. ¡No hay papelera! Lo borrado no se recupera fácilmente.",
    "cp": "Copia archivos o carpetas.",
    "mv": "Mueve o renombra archivos y carpetas.",
    "cat": "Muestra el contenido de un archivo en pantalla.",
    "less": "Ver un archivo página por página (salir con 'q').",
    "grep": "Busca texto dentro de archivos o salidas de otros comandos.",
    "echo": "Imprime texto en la terminal.",
    "man": "Muestra el manual oficial de cualquier comando. Ejemplo: man pacman",
    "chmod": "Cambia los permisos de un archivo o carpeta.",
    "chown": "Cambia el propietario de un archivo o carpeta.",
    "kill": "Termina un proceso por su número PID.",
    "pkill": "Termina procesos por nombre.",
    "hyprctl": "Herramienta para controlar Hyprland: ventanas, workspaces, monitores, animaciones...",
    "git": "Control de versiones: guarda el historial de cambios de tu código.",
    "curl": "Transfiere datos desde/hacia servidores. Usa Internet.",
    "wget": "Descarga archivos desde Internet.",
    "tar": "Empaqueta o desempaqueta archivos .tar.",
    "zip": "Comprime archivos en formato zip.",
    "du": "Calcula cuánto espacio ocupa una carpeta.",
    "date": "Muestra la fecha y hora actuales del sistema.",
    "lspci": "Lista el hardware conectado vía PCI (GPU, placas de red...).",
    "lsusb": "Lista los dispositivos USB conectados.",
    "bluetoothctl": "Gestiona Bluetooth desde la terminal.",
    "wpctl": "Controla el audio (PipeWire): volumen, dispositivos, silenciar.",
    "brightnessctl": "Controla el brillo de la pantalla.",
    "grim": "Captura de pantalla para Wayland.",
    "swaylock": "Bloquea la pantalla en Wayland.",
    "neofetch": "Muestra un resumen bonito del sistema con logo.",
    "checkupdates": "Lista los paquetes que tienen actualización disponible (paquete pacman-contrib).",
}

PACMAN_FLAGS = {
    "-S": "Instala paquetes desde los repositorios.",
    "-Syu": "Sincroniza repositorios y actualiza TODO el sistema. La forma canónica de actualizar Arch.",
    "-Sy": "Solo sincroniza las bases de datos de repositorios (no actualiza).",
    "-Ss": "Busca en los repositorios remotos un paquete por nombre o descripción.",
    "-Si": "Información detallada de un paquete de los repositorios.",
    "-Qi": "Información de un paquete YA instalado localmente.",
    "-Q": "Lista los paquetes instalados localmente.",
    "-R": "Elimina un paquete.",
    "-Rns": "Elimina un paquete, sus dependencias huérfanas y sus archivos de configuración.",
    "-Qtdq": "Lista paquetes instalados como dependencia que ya nadie necesita (huérfanos).",
    "--needed": "No reinstala si el paquete ya está actualizado.",
    "-u": "(con -Sy) actualiza los paquetes que ya tienes instalados.",
    "-y": "(con -S) descarga la base de datos fresca de los repositorios.",
}

SYSTEMCTL_FLAGS = {
    "status": "Muestra el estado actual de un servicio.",
    "start": "Inicia un servicio.",
    "stop": "Detiene un servicio.",
    "restart": "Reinicia un servicio (stop + start).",
    "enable": "Hace que un servicio arranque automáticamente al iniciar el sistema.",
    "disable": "Quita el auto-arranque de un servicio.",
    "--failed": "Lista únicamente los servicios que fallaron.",
    "list-units": "Lista servicios y unidades activas.",
}

GENERIC_FLAGS = {
    "-h": "Muestra la ayuda del comando.",
    "--help": "Muestra la ayuda extendida del comando.",
    "-v": "Modo verboso: muestra más información de lo que hace.",
    "-a": "All / todo: incluye elementos ocultos.",
    "-r": "Recursivo: aplica a carpetas y todo su contenido.",
    "-q": "Quiet / silencioso: menos salida.",
}


def explain_command(cmdline: str) -> str:
    """
    Devuelve un desglose educativo del comando, estilo Archito.

    Ejemplo de salida:

        sudo
        ↓ Ejecuta con privilegios de administrador.
        pacman
        ↓ Gestor de paquetes de Arch Linux.
        -Syu
        ↓ Sincroniza repositorios y actualiza todo el sistema.
    """
    cmdline = cmdline.strip().lstrip("$").strip()
    if not cmdline:
        return "> Escribe un comando después de \"explica\".\n> Ejemplo: explica sudo pacman -Syu"

    try:
        tokens = shlex.split(cmdline)
    except ValueError:
        tokens = cmdline.split()

    if not tokens:
        return f"No reconocí el comando `{cmdline}`."

    lines = []
    base = tokens[0].split("/")[-1]

    doc = COMMANDS.get(base)
    lines.append((base, doc or "Programa externo. Prueba `man %s` para ver su manual." % base))

    i = 1
    while i < len(tokens):
        tok = tokens[i]
        if tok.startswith("-"):
            flag_doc = None
            if base == "pacman":
                # Flags compuestos de pacman: probar el token completo primero
                flag_doc = PACMAN_FLAGS.get(tok)
                if not flag_doc and len(tok) > 2 and tok.startswith("--"):
                    flag_doc = PACMAN_FLAGS.get(tok)
            if not flag_doc and base == "systemctl":
                flag_doc = SYSTEMCTL_FLAGS.get(tok)
            if not flag_doc:
                flag_doc = GENERIC_FLAGS.get(tok)
            if not flag_doc and tok.startswith("-") and not tok.startswith("--") and len(tok) > 2:
                # Desglosar flags cortos combinados (-la → l + a)
                parts = []
                for ch in tok[1:]:
                    single = "-" + ch
                    d = GENERIC_FLAGS.get(single) or (
                        PACMAN_FLAGS.get(single) if base == "pacman" else None)
                    parts.append(f"  · {single} → {d}" if d else f"  · {single}")
                if len(parts) > 1:
                    lines.append((tok, "Combina varias opciones:\n" + "\n".join(parts)))
                    i += 1
                    continue
            lines.append((tok, flag_doc or "Opción de este programa. Consulta `man %s`." % base))
        else:
            prev = tokens[i - 1] if i > 0 else ""
            if prev in ("-S", "--needed") or (base == "pacman" and any(
                    t.startswith("-S") for t in tokens[:i])):
                lines.append((tok, f"Nombre del paquete objetivo ({tok})."))
            elif prev == "status" and base == "systemctl":
                lines.append((tok, f"Nombre del servicio ({tok}.service)."))
            else:
                lines.append((tok, "Argumento del comando."))
        i += 1

    out = []
    for tok, doc in lines:
        first, *rest = doc.split("\n")
        out.append(f"$ {tok}\n↓ {first}")
        for extra in rest:
            out.append(f"  {extra}")
    return "\n".join(out)
