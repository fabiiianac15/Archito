"""Bitácora local y legible de las acciones que el usuario decide tomar.

No guarda conversaciones ni telemetría. Solo decisiones de acción (propuesta,
cancelada, ejecutada o enviada a una terminal) para que Archito pueda responder
con evidencia a «¿qué hice?» sin sacar información del equipo.
"""

import json
import os
import threading
from datetime import datetime
from pathlib import Path


ACTIVITY_DIR = Path(os.environ.get("ARCHITO_CONFIG_DIR")
                    or Path.home() / ".config" / "archito")
ACTIVITY_FILE = ACTIVITY_DIR / "activity.jsonl"
MAX_ENTRIES = 400
_LOCK = threading.Lock()


class ActivityLog:
    """Registro append-only, privado y pequeño."""

    def record_action(self, command: str, label: str, outcome: str) -> None:
        event = {
            "at": datetime.now().astimezone().isoformat(timespec="seconds"),
            "kind": "action",
            "label": str(label)[:120],
            "command": str(command)[:500],
            "outcome": outcome,
        }
        try:
            with _LOCK:
                ACTIVITY_DIR.mkdir(parents=True, exist_ok=True)
                with ACTIVITY_FILE.open("a", encoding="utf-8") as f:
                    f.write(json.dumps(event, ensure_ascii=False) + "\n")
                self._trim()
        except OSError:
            # La bitácora nunca debe impedir una acción solicitada.
            pass

    def _trim(self) -> None:
        try:
            lines = ACTIVITY_FILE.read_text(encoding="utf-8").splitlines()
            if len(lines) > MAX_ENTRIES:
                ACTIVITY_FILE.write_text("\n".join(lines[-MAX_ENTRIES:]) + "\n",
                                         encoding="utf-8")
        except OSError:
            pass

    def recent(self, limit: int = 12) -> list[dict]:
        try:
            with _LOCK:
                lines = ACTIVITY_FILE.read_text(encoding="utf-8").splitlines()
        except OSError:
            return []
        entries = []
        for line in reversed(lines[-max(1, limit):]):
            try:
                event = json.loads(line)
            except json.JSONDecodeError:
                continue
            if event.get("kind") == "action":
                entries.append(event)
        return entries
