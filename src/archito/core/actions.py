"""
Sistema de acciones y permisos — FASE 11 del roadmap.

Clasifica comandos en niveles de seguridad:

    Nivel 1 SAFE     → solo lectura. Se ejecutan en segundo plano
                       y el resultado se muestra dentro del chat.
    Nivel 2 CONFIRM  → modifican el sistema. Archito explica qué hará
                       y pide confirmación explícita.
    Nivel 3 BLOCKED  → destructivos. Nunca se ejecutan automáticamente;
                       Archito solo los explica.

Además, `run_inline()` ejecuta comandos seguros capturando su salida,
para que el usuario no tenga que abrir una terminal para cada consulta.
"""

import os
import re
import shlex
import shutil
import subprocess
from dataclasses import dataclass, field


def clean_env() -> dict:
    """
    Entorno limpio para procesos hijos.

    CRÍTICO: elimina LD_PRELOAD (y similares) del entorno heredado.
    Archito se lanza con LD_PRELOAD=libgtk4-layer-shell.so para poder
    usar layer-shell; si un emulador de terminal hereda esa variable,
    el preload rompe su cliente de Wayland y la terminal se cierra
    al instante.
    """
    env = dict(os.environ)
    for var in ("LD_PRELOAD", "GTK_MODULES", "GDK_BACKEND"):
        env.pop(var, None)
    return env

# ── Niveles de seguridad ──
SAFE = "safe"
CONFIRM = "confirm"
DANGEROUS = "danger"

# Modos de ejecución
MODE_INLINE = "inline"      # captura salida y la muestra en el chat
MODE_TERMINAL = "terminal"  # abre terminal externa (interactivo / sudo)
MODE_SPAWN = "spawn"        # lanza proceso gráfico sin capturar salida


@dataclass
class Action:
    """Acción que Archito propone al usuario."""
    label: str                       # Texto del botón principal
    command: str                     # Comando a ejecutar
    level: str = CONFIRM             # safe | confirm | danger
    mode: str = MODE_TERMINAL        # inline | terminal | spawn
    description: str = ""            # Qué hace, en cristiano
    danger_note: str = ""            # Advertencia extra (nivel peligroso)
    chips: list = field(default_factory=list)


# Comandos que Archito considera seguros para ejecución inline.
# Lista curada a mano: si no está aquí, NO se ejecuta inline.
SAFE_INLINE = [
    r"^uname\b", r"^cat\s+/etc/os-release\b", r"^free\b", r"^df\b",
    r"^ps\b", r"^uptime$", r"^pacman\s+-Q\b", r"^pacman\s+-Ss\b",
    r"^pacman\s+-S[iI]\b", r"^systemctl\s+--failed", r"^journalctl\b",
    r"^nmcli\b", r"^ip\b", r"^lsblk\b", r"^lspci\b", r"^sensors\b",
    r"^hyprctl\b", r"^whoami$", r"^hostname$", r"^date\b",
    r"^du\s+-sh\b", r"^checkupdates$", r"^pactl\s+get\b",
    r"^bluetoothctl\s+(show|devices)$", r"^wpctl\s+(get|status)\b",
    r"^brightnessctl\s+info$", r"^echo\b", r"^hostnamectl$",
]

# Patrones que JAMÁS se ejecutan automáticamente.
FORBIDDEN = [
    r"\brm\b", r"\bmkfs\b", r"\bdd\b\s+if=", r":(){", r"\bshutdown\b",
    r"\breboot\b", r"\bpoweroff\b", r"\binitialize\b", r">\s*/dev/",
    r"chmod\s+-R\s+777\s+/", r"\bpacman\s+-R", r"\bcurl\b.*\|\s*(ba)?sh",
    r"\bwget\b.*\|\s*(ba)?sh", r"\bmv\b\s+/\s", r"\bfork\s*bomb\b",
]


def classify(command: str) -> str:
    """Devuelve el nivel de seguridad de un comando."""
    for pattern in FORBIDDEN:
        if re.search(pattern, command):
            return DANGEROUS
    if "sudo" in command:
        return CONFIRM
    return CONFIRM


def can_inline(command: str) -> bool:
    """True solo si el comando está en la lista blanca de lectura."""
    command = command.strip()
    if any(re.search(p, command) for p in FORBIDDEN):
        return False
    if "sudo" in command:
        return False
    return any(re.search(p, command) for p in SAFE_INLINE)


@dataclass
class CommandResult:
    """Resultado de una ejecución inline."""
    ok: bool
    output: str = ""
    error: str = ""


@dataclass
class Response:
    """Respuesta estructurada que el núcleo devuelve a la interfaz."""
    text: str = ""                                   # Burbuja principal
    code: str = ""                                   # Bloque monoespaciado
    followup: str = ""                               # Segunda burbuja opcional
    actions: list = field(default_factory=list)      # lista[Action]
    chips: list = field(default_factory=list)        # Sugerencias clicables
    stats: list = field(default_factory=list)        # [(label, pct, valor, nivel)]
    status: str = "ok"                               # ok | warn | error | bye
    clear_chat: bool = False                         # Pedir limpieza del chat
    ui_command: str | None = None                    # Orden a la UI (theme:, sync-touch)


MAX_LINES = 30


def _trim(text: str) -> str:
    lines = text.rstrip().splitlines()
    if len(lines) > MAX_LINES:
        head = lines[: MAX_LINES - 1]
        hidden = len(lines) - len(head)
        head.append(f"… (+{hidden} líneas más — usa el botón de terminal para ver todo)")
        return "\n".join(head)
    return text


def run_inline(command: str, timeout: int = 12) -> CommandResult:
    """
    Ejecuta un comando SOLO si está en la lista blanca de lectura.
    Devuelve CommandResult con la salida recortada.
    """
    if not can_inline(command):
        return CommandResult(ok=False, error="Comando fuera de la lista segura.")

    try:
        proc = subprocess.run(
            ["bash", "-c", command],
            capture_output=True,
            text=True,
            timeout=timeout,
            env=clean_env(),
        )
        out = _trim(proc.stdout.strip())
        err = proc.stderr.strip()
        if proc.returncode == 0 and out:
            return CommandResult(ok=True, output=out)
        if proc.returncode == 0:
            return CommandResult(ok=True, output="(sin salida)")
        return CommandResult(
            ok=False,
            output=out,
            error=err or f"código de salida {proc.returncode}",
        )
    except FileNotFoundError:
        return CommandResult(ok=False, error=f"No existe el programa: {shlex.split(command)[0]}")
    except subprocess.TimeoutExpired:
        return CommandResult(ok=False, error="El comando tardó demasiado.")
    except Exception as exc:
        return CommandResult(ok=False, error=str(exc))


def run_detached(command: list[str]) -> bool:
    """Lanza un proceso gráfico (grim, hyprlock...) sin esperar su salida."""
    try:
        subprocess.Popen(
            command,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            start_new_session=True,
            env=clean_env(),
        )
        return True
    except Exception:
        return False


def has_tool(name: str) -> bool:
    """True si existe un binario en PATH."""
    return shutil.which(name) is not None


TERMINALS = [
    "kitty", "alacritty", "foot", "wezterm", "ghostty",
    "gnome-terminal", "konsole", "xfce4-terminal", "tilix", "xterm",
]


def find_terminal() -> str | None:
    """Detecta el emulador: primero $TERMINAL/$TERM_PROGRAM, luego lista."""
    for var in ("TERMINAL", "TERM_PROGRAM"):
        val = os.environ.get(var, "")
        if val and shutil.which(val.split()[0]):
            return val
    for term in TERMINALS:
        if shutil.which(term):
            return term
    return None


# Prefijos de cada emulador antes del comando final.
# Formato: lista fija de argumentos; {title} se sustituye por el título.
TERMINAL_ARGS = {
    "kitty":          ["--title", "{title}"],
    "alacritty":      ["--title", "{title}", "-e"],
    "foot":           ["--title", "{title}"],
    "wezterm":        ["start", "--title", "{title}"],
    "ghostty":        ["--title", "{title}", "-e"],
    "gnome-terminal": ["--title", "{title}", "--"],
    "konsole":        ["--title", "{title}", "-e"],
    "xfce4-terminal": ["--title", "{title}", "-x"],
    "tilix":          ["--title", "{title}", "-x"],
    "xterm":          ["-title", "{title}", "-e"],
}


def build_terminal_cmd(term: str, script: str, title: str = "Archito") -> list[str]:
    """Construye el argv correcto para cada emulador conocido."""
    prefix = TERMINAL_ARGS.get(term)
    if prefix is None:
        # Genérico: la mayoría acepta -e comando
        return [term, "-e", "bash", "-c", script]
    argv = [t.format(title=title) for t in prefix]
    return [term] + argv + ["bash", "-c", script]


def open_in_terminal(command: str, title: str = "Archito") -> bool:
    """
    Abre el comando en un emulador de terminal externo.

    El script muestra el comando, lo ejecuta y SIEMPRE queda abierta:
    tras `read`, si algo raro pasa, cae a un shell interactivo.
    Se limpia LD_PRELOAD del entorno hijo (ver clean_env).
    """
    term = find_terminal()
    if term is None:
        return False

    script = (
        f'printf "\\033[1;36m$ {command}\\033[0m\\n\\n"; '
        f'{command}; '
        f'status=$?; '
        f'echo; '
        f'if [ $status -ne 0 ]; then '
        f"printf '\\033[1;31m── Terminó con error (%s) ──\\033[0m\\n' \"$status\"; "
        f'else '
        f"printf '\\033[1;32m── Listo ──\\033[0m\\n'; "
        f'fi; '
        f'echo "── Presiona Enter para volver ──"; '
        f'read _ || true; '
        f'exec bash --norc -i'
    )

    cmd = build_terminal_cmd(term, script, title)

    try:
        subprocess.Popen(cmd, start_new_session=True, env=clean_env())
        return True
    except Exception:
        return False
