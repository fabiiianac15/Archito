"""Puente opcional hacia un modelo local de Ollama.

El modelo nunca recibe acceso a la terminal: solo puede redactar una respuesta
para preguntas que el enrutador determinista de Archito no reconoce. Por
defecto se habla únicamente con ``localhost``; no hay nube ni telemetría.
"""

import json
import os
import shutil
import subprocess
from dataclasses import dataclass
from urllib.error import URLError, HTTPError
from urllib.parse import urlparse
from urllib.request import Request, urlopen


DEFAULT_URL = "http://127.0.0.1:11434"
DEFAULT_MODEL = "llama3.2:3b"

SYSTEM_PROMPT = """Eres Archito, un asistente local, amable y preciso para Arch Linux.
Responde en español claro y conciso. No inventes resultados del sistema del usuario.
No tienes acceso directo a archivos, red, terminal ni privilegios. Puedes elegir
una herramienta de lectura conocida o proponer UN comando para que el usuario lo
revise y confirme. Jamás afirmes que ya ejecutaste algo. Enseña sin sermonear."""

TOOLS = (
    "system_summary", "health", "performance", "updates", "recent_errors",
    "failed_services", "network", "packages",
)

PLAN_SCHEMA = {
    "type": "object",
    "properties": {
        "kind": {"type": "string", "enum": ["answer", "tool", "command"]},
        "answer": {"type": "string"},
        "tool": {"type": "string", "enum": list(TOOLS) + [""]},
        "command": {"type": "string"},
        "action_label": {"type": "string"},
        "safety_note": {"type": "string"},
    },
    "required": ["kind", "answer", "tool", "command", "action_label", "safety_note"],
    "additionalProperties": False,
}


@dataclass(frozen=True)
class Plan:
    """Plan validado, no una orden ejecutable por el modelo."""
    kind: str
    answer: str
    tool: str = ""
    command: str = ""
    action_label: str = ""
    safety_note: str = ""


def _local_url(url: str) -> str | None:
    """Acepta solo el servidor local para conservar la promesa de privacidad."""
    parsed = urlparse(url)
    if parsed.scheme not in ("http", "https"):
        return None
    if parsed.hostname not in ("localhost", "127.0.0.1", "::1"):
        return None
    return url.rstrip("/")


def ensure_server() -> str:
    """Hace que la IA local esté disponible sin pedir ``ollama serve``.

    El instalador configura el servicio de systemd para el caso normal. Esto
    cubre también instalaciones portables: si Ollama existe pero su servicio no
    está activo, Archito inicia un servidor hijo local y aislado. Nunca descarga
    un modelo aquí: una descarga grande debe ser una decisión explícita durante
    la instalación.
    """
    base = _local_url(os.environ.get("ARCHITO_OLLAMA_URL", DEFAULT_URL))
    if not base:
        return "invalid-url"
    try:
        with urlopen(f"{base}/api/version", timeout=0.35):
            return "ready"
    except (URLError, HTTPError, OSError, ValueError):
        pass

    executable = shutil.which("ollama")
    if not executable:
        return "missing"
    env = dict(os.environ)
    # El preload de layer-shell no debe contaminar el cliente de Ollama.
    env.pop("LD_PRELOAD", None)
    try:
        subprocess.Popen(
            [executable, "serve"], stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL, stdin=subprocess.DEVNULL,
            start_new_session=True, env=env,
        )
        return "starting"
    except OSError:
        return "failed"


def ask(question: str, model: str | None = None) -> str | None:
    """Devuelve una respuesta de Ollama o ``None`` si no está disponible."""
    base = _local_url(os.environ.get("ARCHITO_OLLAMA_URL", DEFAULT_URL))
    if not base:
        return None
    payload = {
        "model": model or os.environ.get("ARCHITO_OLLAMA_MODEL", DEFAULT_MODEL),
        "stream": False,
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": question},
        ],
        "options": {"temperature": 0.45, "num_predict": 420},
    }
    request = Request(
        f"{base}/api/chat", data=json.dumps(payload).encode("utf-8"),
        headers={"Content-Type": "application/json"}, method="POST")
    try:
        with urlopen(request, timeout=45) as response:
            data = json.loads(response.read().decode("utf-8"))
        text = str(data.get("message", {}).get("content", "")).strip()
        return text or None
    except (URLError, HTTPError, OSError, ValueError, json.JSONDecodeError):
        return None


def plan(question: str, context: str, model: str | None = None) -> Plan | None:
    """Pide un plan estructurado al modelo local, sin darle una shell.

    ``format`` obliga un JSON con tres únicas salidas posibles. La validación de
    abajo vuelve a comprobarlo: la respuesta del modelo es siempre *datos*,
    nunca código que se ejecute directamente.
    """
    base = _local_url(os.environ.get("ARCHITO_OLLAMA_URL", DEFAULT_URL))
    if not base:
        return None
    prompt = (
        "Contexto real y limitado del sistema (puede estar incompleto):\n"
        f"{context}\n\n"
        "Herramientas permitidas: " + ", ".join(TOOLS) + ".\n"
        "Elige kind='tool' si una herramienta basta. Elige kind='command' solo "
        "si el usuario pide una acción que requiere comando; propone un único "
        "comando visible, sin pipes remotos ni redirecciones destructivas. "
        "Elige kind='answer' para explicar o conversar.\n\n"
        f"Petición del usuario: {question}"
    )
    payload = {
        "model": model or os.environ.get("ARCHITO_OLLAMA_MODEL", DEFAULT_MODEL),
        "stream": False,
        "format": PLAN_SCHEMA,
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": prompt},
        ],
        "options": {"temperature": 0.15, "num_predict": 360},
    }
    request = Request(
        f"{base}/api/chat", data=json.dumps(payload).encode("utf-8"),
        headers={"Content-Type": "application/json"}, method="POST")
    try:
        with urlopen(request, timeout=50) as response:
            data = json.loads(response.read().decode("utf-8"))
        raw = json.loads(data.get("message", {}).get("content", "{}"))
        kind = str(raw.get("kind", ""))
        answer = str(raw.get("answer", "")).strip()[:2000]
        tool = str(raw.get("tool", "")).strip()
        command = str(raw.get("command", "")).strip()
        label = str(raw.get("action_label", "")).strip()[:100]
        note = str(raw.get("safety_note", "")).strip()[:500]
        if kind not in ("answer", "tool", "command") or not answer:
            return None
        if kind == "tool" and tool not in TOOLS:
            return None
        # No aceptamos un script multilínea ni NUL. Aun un comando válido pasa
        # luego por la tarjeta de confirmación y la clasificación de riesgo.
        if kind == "command" and (not command or len(command) > 700 or
                                  "\x00" in command or "\n" in command):
            return None
        return Plan(kind, answer, tool, command, label, note)
    except (URLError, HTTPError, OSError, ValueError, TypeError, json.JSONDecodeError):
        return None


def status(model: str | None = None) -> str | None:
    """Nombre del modelo local en servicio, o ``None`` si Ollama no responde."""
    base = _local_url(os.environ.get("ARCHITO_OLLAMA_URL", DEFAULT_URL))
    if not base:
        return None
    try:
        with urlopen(f"{base}/api/tags", timeout=1.5) as response:
            data = json.loads(response.read().decode("utf-8"))
        names = {item.get("name") for item in data.get("models", [])}
        wanted = model or os.environ.get("ARCHITO_OLLAMA_MODEL", DEFAULT_MODEL)
        return wanted if wanted in names else None
    except (URLError, HTTPError, OSError, ValueError, json.JSONDecodeError):
        return None
