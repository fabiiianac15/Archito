"""
Memoria local de Archito — FASE 12 del roadmap.

Guarda preferencias del usuario en ~/.config/archito/memory.json.
Nada sale del dispositivo: es memoria 100% local.
"""

import json
import os
from pathlib import Path

MEMORY_DIR = Path(os.environ.get("ARCHITO_CONFIG_DIR")
                  or Path.home() / ".config" / "archito")
MEMORY_FILE = MEMORY_DIR / "memory.json"

DEFAULTS = {
    "user_name": None,
    "prefer_explanations": True,
    "touch_mode": False,
    "theme": "archito-noche",
    "local_ai": True,
    "local_ai_model": "llama3.2:3b",
}


class Memory:
    """Preferencias persistidas del usuario."""

    def __init__(self):
        self._data = dict(DEFAULTS)
        self.load()

    def load(self):
        try:
            if MEMORY_FILE.exists():
                raw = json.loads(MEMORY_FILE.read_text(encoding="utf-8"))
                self._data.update({k: v for k, v in raw.items() if k in DEFAULTS})
        except (json.JSONDecodeError, OSError):
            self._data = dict(DEFAULTS)

    def save(self):
        try:
            MEMORY_DIR.mkdir(parents=True, exist_ok=True)
            MEMORY_FILE.write_text(
                json.dumps(self._data, indent=2, ensure_ascii=False),
                encoding="utf-8",
            )
        except OSError:
            pass

    # ── Nombre del usuario ──
    @property
    def user_name(self) -> str | None:
        return self._data.get("user_name")

    @user_name.setter
    def user_name(self, value: str | None):
        self._data["user_name"] = value
        self.save()

    @property
    def prefer_explanations(self) -> bool:
        return bool(self._data.get("prefer_explanations"))

    @prefer_explanations.setter
    def prefer_explanations(self, value: bool):
        self._data["prefer_explanations"] = bool(value)
        self.save()

    @property
    def touch_mode(self) -> bool:
        return bool(self._data.get("touch_mode"))

    @touch_mode.setter
    def touch_mode(self, value: bool):
        self._data["touch_mode"] = bool(value)
        self.save()

    @property
    def theme(self) -> str:
        return str(self._data.get("theme") or "archito-noche")

    @theme.setter
    def theme(self, value: str):
        self._data["theme"] = str(value)
        self.save()

    @property
    def local_ai(self) -> bool:
        return bool(self._data.get("local_ai"))

    @local_ai.setter
    def local_ai(self, value: bool):
        self._data["local_ai"] = bool(value)
        self.save()

    @property
    def local_ai_model(self) -> str:
        return str(self._data.get("local_ai_model") or "llama3.2:3b")
