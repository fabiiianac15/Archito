"""
Utilidades de texto y contenido de personalidad de Archito.
"""

import random
import re
import unicodedata


def norm(s: str) -> str:
    """Minúsculas, sin acentos, sin signos de puntuación.
    Preserva números decimales (85.5 → 85.5, no «85 5»)."""
    s = s.lower().strip()
    s = unicodedata.normalize("NFD", s)
    s = "".join(c for c in s if not unicodedata.combining(c))
    s = re.sub(r"(?<=\d)[.,](?=\d)", "\x00", s)
    s = re.sub(r"[¿?¡!.,;:\"']+", " ", s)
    s = s.replace("\x00", ".")
    return re.sub(r"\s+", " ", s).strip()


def pick(*options):
    return random.choice(options)


def bar(pct: float, width: int = 14) -> str:
    """Barra visual tipo [█████░░░░] para porcentajes."""
    pct = max(0.0, min(100.0, pct))
    filled = round(pct / 100 * width)
    return "[" + "█" * filled + "░" * (width - filled) + "]"


TIPS = [
    "**man comando** es tu mejor amigo: el manual vive dentro de tu sistema, sin Internet.",
    "Con **Tab** completas nombres de archivos y comandos. Con **↑** recuperas el anterior.",
    "**Ctrl+R** busca en tu historial de la terminal mientras escribes. Pruébalo.",
    "En Arch conviene actualizar seguido: **sudo pacman -Syu** evita conflictos grandes.",
    "Los paquetes huérfanos se listan con **pacman -Qtdq**. Yo puedo hacerlo por ti.",
    "**df -h** muestra el espacio de tus discos de forma legible.",
    "**journalctl -p err** muestra solo errores del sistema. Muy útil tras un fallo.",
    "Si algo se cuelga en terminal: **Ctrl+C** lo interrumpe.",
    "Los permisos son simples: leer (r=4), escribir (w=2), ejecutar (x=1).",
    "Antes de copiar un comando de Internet, entiéndelo. Dime \"explica <comando>\" y lo desarmo.",
]

DEFAULT_REPLIES = [
    "Hmm, esa no me la sé todavía. 🤔\nEscribe **ayuda** para ver todo lo que puedo hacer.",
    "No entendí eso del todo. 🤔\nPrueba con \"cuánta ram tengo\", \"busca firefox\"\no escribe **ayuda**.",
    "Todavía no llego a eso. 🐣\nEstoy hecho a mano, sin IA externa: cada cosa que sé\nme la programaron con cariño. Escribe **ayuda**.",
]

WELCOME_TEXT = (
    "¡Hola! Soy **Archito** 🐧\n"
    "Tu copiloto local de Arch Linux.\n\n"
    "Pregúntame por tu sistema, pídeme acciones\n"
    "o escribe **ayuda** para ver todo lo que sé hacer."
)

HELP_TEXT = (
    "**Esto sé hacerlo hoy:** 🛠️\n\n"
    "$ Sistema — info del sistema · kernel · distro · cpu · temperatura · ram · disco · gpu · batería · uptime\n"
    "$ Red — mi ip · redes wifi · internet · bluetooth · volumen actual\n"
    "$ Paquetes — cuántos paquetes · busca <algo> · huérfanos · paquetes grandes · caché\n"
    "$ Acciones — actualiza el sistema · instala <paquete> · recarga hyprland · captura de pantalla\n"
    "$ Hardware — sube/baja el volumen · brillo +10% · bloquea pantalla\n"
    "$ Diagnóstico — estado de salud · por qué va lento · servicios fallidos · errores recientes\n"
    "$ Transparencia — mi actividad (bitácora local de acciones)\n"
    "$ IA local — preguntas libres, acciones preparadas con confirmación obligatoria\n"
    "$ Profesor — explica <comando> · consejo · modo principiante\n\n"
    "> Escríbelo con tus palabras: entiendo variantes,\n"
    "> con o sin acentos. Y nada sale de tu equipo. 🔒"
)
