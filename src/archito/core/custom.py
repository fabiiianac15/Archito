"""
Comandos personales de Archito — semilla del sistema de plugins (FASE 14).

El usuario puede definir SUS propias órdenes en:

    ~/.config/archito/commands.json

Formato:

    {
      "_ayuda": "clave → frase que activa el comando",
      "backup fotos": {
        "command": "rsync -a ~/Fotos /run/media/backup/",
        "description": "Copia incremental de Fotos al disco externo.",
        "dangerous": false
      }
    }

Reglas de seguridad (FASE 11):
  · Todo comando personal se ejecuta SIEMPRE en terminal externa.
  · Nunca inline, aunque parezca de lectura: el usuario manda,
    pero Archito verifica y confirma.
  · `dangerous: true` marca la tarjeta en rojo con advertencia extra.
  · La ruta se puede redirigir con ARCHITO_CONFIG_DIR (tests/portabilidad).
"""

import json
import os
from pathlib import Path

CUSTOM_DIR = Path(os.environ.get("ARCHITO_CONFIG_DIR")
                  or Path.home() / ".config" / "archito")
CUSTOM_FILE = CUSTOM_DIR / "commands.json"


def load_commands() -> dict[str, dict]:
    """
    Carga y valida commands.json. Devuelve {frase_normalizada: spec}.
    Inválido o ausente → {} sin explotar.
    """
    if not CUSTOM_FILE.exists():
        return {}
    try:
        raw = json.loads(CUSTOM_FILE.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError):
        return {}

    valid: dict[str, dict] = {}
    if not isinstance(raw, dict):
        return {}
    for key, spec in raw.items():
        if key.startswith("_"):
            continue  # claves de documentación
        if not isinstance(spec, dict):
            continue
        command = spec.get("command")
        if not isinstance(command, str) or not command.strip():
            continue
        valid[key.strip().lower()] = {
            "command": command.strip(),
            "description": str(spec.get("description", "")).strip(),
            "dangerous": bool(spec.get("dangerous", False)),
            "confirm_text": str(spec.get("confirm_text", "")).strip(),
        }
    return valid


def template_json() -> str:
    """Plantilla comentada para que el usuario empiece en segundos."""
    return json.dumps({
        "_ayuda": (
            "Cada clave es la frase que le dirás a Archito. "
            "command es lo que se ejecutará (siempre con confirmación "
            "y siempre en una terminal visible). dangerous:true pinta "
            "la confirmación en rojo. Borra este _ayuda si quieres."
        ),
        "backup documentos": {
            "command": "rsync -avh --delete ~/Documentos/ /run/media/$USER/backup/Documentos/",
            "description": "Copia incremental de Documentos al disco externo montado.",
            "dangerous": False,
        },
        "limpiar journal": {
            "command": "sudo journalctl --vacuum-time=7d",
            "description": "Recorta los logs de systemd dejando solo la última semana.",
            "dangerous": True,
            "confirm_text": "Esto borra logs antiguos de forma permanente. ¿Seguro?",
        },
        "velocidad red": {
            "command": "speedtest-cli --simple",
            "description": "Mide tu velocidad de descarga/subida (usa Internet).",
            "dangerous": False,
        },
    }, indent=2, ensure_ascii=False)
