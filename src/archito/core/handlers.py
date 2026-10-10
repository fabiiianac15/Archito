"""
Manejadores de intenciones de Archito — FASE 5 y FASE 6 del roadmap.

Cada handler recibe (brain, n_normalizado, texto_original) y devuelve
una Response estructurada. Sin lógica de UI aquí: solo núcleo.
"""

import datetime
import random
import re
import subprocess
from pathlib import Path

from archito.core import system
from archito.core.actions import (
    Action,
    Response,
    CONFIRM,
    DANGEROUS,
    MODE_INLINE,
    MODE_SPAWN,
    MODE_TERMINAL,
    clean_env,
    has_tool,
)
from archito.core.mathcalc import calc
from archito.core.textutils import HELP_TEXT, bar, pick


def _exec_now(command: str) -> str | None:
    try:
        r = subprocess.run(["bash", "-c", command], capture_output=True,
                           text=True, timeout=6, env=clean_env())
        return r.stdout.strip() or r.stderr.strip() or None
    except Exception:
        return None


PKG_OK = re.compile(r"^[A-Za-z0-9][A-Za-z0-9.@_+-]*$")


# ══════════════════════════════════════════════
# CONVERSACIÓN
# ══════════════════════════════════════════════

def h_saludo(brain, n, raw):
    name = brain.memory.user_name
    hora = datetime.datetime.now().hour
    if hora < 12:
        saludo = "¡Buenos días"
    elif hora < 20:
        saludo = "¡Buenas tardes"
    else:
        saludo = "¡Buenas noches"
    if name:
        return Response(
            text=f"{saludo}, {name}! 🐧\n¿En qué te ayudo con tu sistema hoy?",
            chips=["info del sistema", "ayuda"],
        )
    return Response(
        text=(f"{saludo}! 🐧 Soy Archito.\n¿En qué te ayudo? "
              "Si me dices tu nombre, lo recuerdo."),
        chips=["ayuda"],
    )


def h_identidad(brain, n, raw):
    return Response(text=(
        "Soy **Archito**, tu asistente local para Arch Linux. 🐧\n\n"
        "No soy un chatbot en la nube: vivo en tu equipo,\n"
        "no uso Internet para pensar y no mando nada\n"
        "de tu máquina hacia fuera.\n\n"
        "> Mi filosofía: *no reemplazar la terminal,\n> sino ayudarte a entenderla.*"
    ), chips=["filosofía", "quién te creó", "versión"])


def h_comoestas(brain, n, raw):
    return Response(text=pick(
        "¡De maravilla! ⚡ Corriendo a velocidad kernel.\n¿Y tú? ¿Cómo va tu sistema?",
        "Como un buen Arch: estable, minimalista y sin bloat. 😎\n¿Qué necesitas?",
    ))


def h_gracias(brain, n, raw):
    name = brain.memory.user_name
    extra = f", {name}" if name else ""
    return Response(text=pick(
        f"¡De nada{extra}! 🎉 Aquí estaré cuando necesites algo.",
        f"Un placer{extra}. 🐧 Para eso estoy.",
        "De nada ✨ Y recuerda: cada comando que ves, apréndelo.",
    ))


def h_despedida(brain, n, raw):
    resp = Response(text=pick(
        "¡Hasta luego! 👋 Me guardo aquí por si me necesitas.",
        "Chao chao 🐧 Vuelve cuando quieras.",
        "Nos vemos ✨ Que tu uptime sea largo y tus logs cortos.",
    ))
    resp.status = "bye"
    return resp


def h_ayuda(brain, n, raw):
    return Response(text=HELP_TEXT)


def h_version(brain, n, raw):
    import sys
    import archito
    ver = getattr(archito, "__version__", "?")
    py = f"{sys.version_info.major}.{sys.version_info.minor}"
    return Response(text=(
        f"**Archito v{ver}** 🐧\n\n"
        f"$ Hecho a mano en Python {py}\n"
        "$ Sin IA en la nube · 100% local\n"
        "$ GTK4 + layer-shell sobre Wayland"
    ), chips=["filosofía", "ayuda"])


def h_filosofia(brain, n, raw):
    return Response(text=(
        "**No reemplazar la terminal. Ayudar a entenderla.**\n\n"
        "Por eso cada respuesta te muestra el comando real:\n"
        "para que poco a poco no me necesites.\n\n"
        "> El objetivo final es que pierdas el miedo\n"
        "> a Arch Linux y recuperes el control."
    ), chips=["consejo", "explica sudo pacman -Syu"])


def h_creador(brain, n, raw):
    return Response(text=(
        "Me construyó alguien como tú: 🛠️\n"
        "alguien que quería hablarle a su Linux\nsin miedo y sin depender de la nube.\n\n"
        "Todo lo que sé está en código abierto:\npuedes leerme, cambiarme y hacerme crecer."
    ), chips=["filosofía", "versión"])


def h_clima(brain, n, raw):
    return Response(text=(
        "El clima es lo único que NO puedo decirte. ☀️\n\n"
        "Funciono 100% local: no consulto servicios externos\n"
        "ni mando tu ubicación a ningún servidor.\n\n"
        "> Es una decisión de privacidad, no una limitación técnica 😉"
    ), chips=["temperatura cpu", "ayuda"])


# ══════════════════════════════════════════════
# UTILIDADES
# ══════════════════════════════════════════════

DIAS = ["lunes", "martes", "miércoles", "jueves", "viernes", "sábado", "domingo"]
MESES = ["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio",
         "agosto", "septiembre", "octubre", "noviembre", "diciembre"]


def h_hora(brain, n, raw):
    ahora = datetime.datetime.now()
    return Response(text=(
        f"Son las **{ahora.strftime('%H:%M')}** ⏰\n"
        "(Leída directamente del reloj de tu sistema.)"
    ))


def h_fecha(brain, n, raw):
    hoy = datetime.date.today()
    return Response(text=(
        f"Hoy es **{DIAS[hoy.weekday()]}**, {hoy.day} de "
        f"**{MESES[hoy.month - 1]}** de {hoy.year}. 📅"
    ))


def h_calculadora(brain, expr):
    try:
        result = calc(expr)
        return Response(text=f"**= {result}** 🧮",
                        chips=["cuánto es 2^10 * 3"])
    except ZeroDivisionError:
        return Response(text="Dividir entre cero rompe universos. 🕳️ Prueba otro número.",
                        status="warn")
    except Exception:
        return Response(text=(
            "Esa expresión no la entendí como cuenta. 🤔\n"
            "Entiendo + - * / ^ paréntesis sqrt(...) pi...\n"
            "Ejemplo: `cuánto es sqrt(144) + 3^3`"
        ), status="warn")


def h_consejo(brain, n, raw):
    from archito.core.textutils import TIPS
    return Response(text=pick(*TIPS), chips=["otro consejo"])


def h_limpiar_chat(brain, n, raw):
    resp = Response(text="Chat limpio. Empezamos de cero. ✨")
    resp.clear_chat = True
    return resp


def h_preferencias_on(brain, n, raw):
    brain.memory.prefer_explanations = True
    return Response(text=(
        "Hecho ✅ Desde ahora te explico cada comando\nantes de ejecutarlo. Modo profesor activado. 🎓"
    ))


def h_preferencias_off(brain, n, raw):
    brain.memory.prefer_explanations = False
    return Response(text=(
        "Hecho ✅ Ya no añadiré explicaciones automáticas.\nPuedes pedirlas igual con el botón *Explicar*."
    ))

# ══════════════════════════════════════════════
# SISTEMA — información (FASE 6)
# ══════════════════════════════════════════════

def h_info_sistema(brain, n, raw):
    return Response(
        text="Aquí va un resumen de tu máquina: 📋",
        code=system.quick_summary(),
        chips=["temperatura cpu", "espacio en disco", "por qué va lento"],
    )


def h_salud(brain, n, raw):
    """Centro preventivo: una lectura breve antes de que algo se convierta en bug."""
    from archito.core.health import inspect
    report = inspect()
    icon = {"sereno": "✨", "atención": "⚠️", "prioridad": "🚨"}[report.state]
    title = f"Estado de salud: **{report.score}/100** {icon}"
    body = "\n".join(f"• {item}" for item in report.findings)
    return Response(
        text=(f"{title}\n\n{body}\n\n"
              "> Revisión local: solo leo datos del equipo; no cambio nada."),
        stats=report.stats,
        status="warn" if report.state != "sereno" else "ok",
        chips=report.chips,
    )


def h_actividad(brain, n, raw):
    """Muestra evidencia de acciones; no conserva ni expone el chat completo."""
    events = brain.activity.recent()
    if not events:
        return Response(text=(
            "Aún no hay acciones en la bitácora. 🧾\n\n"
            "Solo guardo localmente cuando confirmas, cancelas o envías una acción; "
            "las conversaciones no se registran."
        ), chips=["estado de salud", "ayuda"])
    labels = {
        "cancelled": "cancelado",
        "completed": "ejecutado",
        "failed": "falló",
        "terminal_opened": "enviado a terminal",
        "spawned": "lanzado",
        "terminal_failed": "no abrió terminal",
    }
    lines = []
    for event in events:
        when = str(event.get("at", ""))[11:16] or "--:--"
        outcome = labels.get(event.get("outcome"), event.get("outcome", "registrado"))
        lines.append(f"$ {when} · {outcome:18} {event.get('command', '')}")
    return Response(
        text=("Tu bitácora local de acciones: 🧾\n\n"
              "> No almacena el chat ni se sincroniza con nadie."),
        code="\n".join(lines), chips=["estado de salud", "ayuda"],
    )


def h_kernel(brain, n, raw):
    k = system.kernel_version()
    if not k:
        return Response(text="No pude leer la versión del kernel.", status="error")
    return Response(text=(
        f"Tu kernel es **{k}** 🐧\n\n"
        "El kernel es el corazón de Linux: habla directo\n"
        "con tu hardware (CPU, RAM, discos...)."
    ), chips=["distro", "cpu"])


def h_distro(brain, n, raw):
    d = system.distribution()
    if not d:
        return Response(text="No pude leer tu distribución.", status="error")
    pkgs = system.package_count()
    extra = f"\nCon **{pkgs}** paquetes instalados." if pkgs else ""
    return Response(text=f"Tu sistema es **{d}**. 💿{extra}",
                    chips=["cuántos paquetes", "kernel", "info del sistema"])


def h_hostname(brain, n, raw):
    h = system.hostname()
    return Response(text=(f"Tu equipo se llama **{h}**. 🏷️\n"
                          "Puedes cambiarlo con `hostnamectl set-hostname nuevo-nombre`.")
                    if h else Response(text="No pude leer el hostname.", status="error"))


def h_usuario(brain, n, raw):
    u = system.current_user()
    sh = system.shell_name()
    shell = f" y tu shell es `{sh}`" if sh else ""
    return Response(text=f"Eres **{u}** 👤{shell}.",
                    chips=[f"explica whoami", "usuarios conectados"])


def h_usuarios_conectados(brain, n, raw):
    r = _exec_now("who")
    if not r:
        return Response(text="No hay sesiones activas visibles (o `who` no está).")
    lines = [l for l in r.splitlines() if l.strip()][:6]
    users = sorted({l.split()[0] for l in lines})
    return Response(text=f"Hay {len(users)} usuario(s) con sesión activa: 👥",
                    code="\n".join(lines))


def h_shell(brain, n, raw):
    sh = system.shell_name()
    if not sh:
        return Response(text="No detecté tu shell.")
    return Response(text=(
        f"Tu shell es **{sh}** 🐚\n\n"
        "> La shell es el intérprete: traduce lo que escribes\n"
        "> a órdenes que el kernel entiende."
    ))


def h_escritorio(brain, n, raw):
    de = system.desktop_env()
    ws = system.hypr_workspace()
    mons = system.hypr_monitors()
    parts = []
    if de:
        parts.append(f"Entorno: **{de}**")
    if ws:
        parts.append(f"{ws}")
    if mons:
        parts.append(f"Pantalla: {mons}")
    if not parts:
        return Response(text="No detecté un entorno gráfico (¿estás en TTY?).")
    return Response(text="Tu escritorio ahora mismo: 🖥️",
                    code="\n".join(f"$ {p}" for p in parts))


def h_uptime(brain, n, raw):
    up = system.uptime_pretty()
    if not up:
        return Response(text="No pude leer el uptime.", status="error")
    return Response(text=(
        f"Llevas encendido **{up}** ⏱️\n\n"
        "Los servidores pueden años sin dormir; los PCs\n"
        "descansan cada noche. Reiniciar aplica actualizaciones."
    ), chips=["tiempo de arranque", "actualiza el sistema"])


def h_boot(brain, n, raw):
    bt = system.boot_time()
    if not bt:
        return Response(text="`systemd-analyze` no respondió.", status="warn")
    return Response(text=f"Tu arranque tardó: ⚡\n$ {bt}",
                    chips=["uptime"])


def h_errores_recientes(brain, n, raw):
    r = _exec_now("journalctl -p err -n 8 --no-pager -q")
    if not r or "No journal files" in r:
        return Response(text=(
            "Sin errores recientes legibles en tu journal. ✅\n"
            "(Si crees que falta permiso, añade tu usuario\n"
            "al grupo `systemd-journal`.)"
        ), chips=["servicios fallidos"])
    return Response(text="Últimos errores del sistema: 📜",
                    code=r, chips=["servicios fallidos", "por qué va lento"])


# ══════════════════════════════════════════════
# HARDWARE — CPU, RAM, disco, GPU, batería...
# ══════════════════════════════════════════════

def _temp_verdict(c: float) -> str:
    if c < 50:
        return "Fresquito como una mañana en la montaña. ✅"
    if c < 70:
        return "Temperatura normal de trabajo. ✅"
    if c < 85:
        return "Está calentito. Vigílala si sube más. ⚠️"
    return "¡Caliente! Revisa ventilación o procesos pesados. 🔥"


def h_cpu(brain, n, raw):
    model = system.cpu_model() or "CPU desconocida"
    cores = system.cpu_cores() or "?"
    usage = system.cpu_usage()
    temp = system.cpu_temp()
    lines = [f"$ {model}", f"$ Núcleos/hilos: {cores}"]
    if usage is not None:
        lines.append(f"$ Uso ahora: {usage}%  {bar(usage)}")
    resp_text = "Este es tu procesador: ⚙️"
    chips = ["procesos top", "temperatura cpu"]
    if usage is not None and usage > 80:
        resp_text += "\n¡Vaya, está trabajando duro!"
        chips.insert(0, "por qué va lento")
    if temp:
        lines.append(f"$ Temperatura: {temp[0]:.0f}°C ({temp[1]})")
    return Response(text=resp_text, code="\n".join(lines), chips=chips)


def h_temperatura(brain, n, raw):
    t = system.cpu_temp()
    if not t:
        return Response(text=(
            "No encontré sensores de temperatura accesibles. 🌡️\n"
            "Prueba instalando `lm_sensors` y ejecutando `sensors`."
        ), status="warn", chips=["busca lm_sensors"])
    c, label = t
    icon = "✅" if c < 70 else ("⚠️" if c < 85 else "🔥")
    return Response(text=(
        f"Sensor más caliente: **{c:.0f}°C** ({label}) {icon}\n\n"
        f"{_temp_verdict(c)}"
    ), chips=["procesos top", "por qué va lento"])


def h_procesos(brain, n, raw):
    by_mem = "memoria" in n or "ram" in n
    procs = system.top_processes(by="mem" if by_mem else "cpu", limit=6)
    if not procs:
        return Response(text="No pude listar los procesos.", status="error")
    title = "Los que más memoria usan: 🐏" if by_mem else "Los que más CPU consumen: ⚙️"
    lines = []
    for p in procs:
        val = p["mem"] if by_mem else p["cpu"]
        label = "RAM" if by_mem else "CPU"
        lines.append(f"$ {p['name']:<20} {label}: {val}%  (pid {p['pid']})")
    return Response(text=title, code="\n".join(lines),
                    chips=["por qué va lento", "temperatura cpu"])


def h_ram(brain, n, raw):
    r = system.ram_report()
    if not r:
        return Response(text="No pude leer la memoria RAM.", status="error")
    sev = "danger" if r["pct"] > 90 else ("warn" if r["pct"] > 80 else "good")
    stats = [("RAM", r["pct"], f"{r['used_gb']:.1f} / {r['total_gb']:.0f} GB", sev)]
    if r.get("swap_pct") is not None:
        stats.append(("Swap", r["swap_pct"], "en uso", None))
    chips = ["procesos por memoria", "temperatura cpu"]
    warn = ""
    if r["pct"] > 85:
        warn = "\n\n⚠️ Vas justo de memoria. Mira quién la consume:"
        chips.insert(0, "procesos por memoria")
    return Response(text=f"Memoria RAM: 🐏{warn}",
                    stats=stats, chips=chips)


def h_disco(brain, n, raw):
    d = system.disk_report("/")
    if not d:
        return Response(text="No pude leer el disco.", status="error")
    sev = "danger" if d["pct"] > 90 else ("warn" if d["pct"] > 75 else "good")
    stats = [("Disco raíz (/)", d["pct"],
              f"{d['free_gb']:.0f} GB libres", sev)]
    warn = ""
    chips = ["paquetes grandes", "limpia la caché"]
    if d["pct"] > 90:
        warn = ("\n\n🚨 ¡Queda poco espacio! Un disco lleno vuelve lento\n"
                "el sistema. Te ayudo a liberar:")
        chips.insert(0, "paquetes grandes")
    elif d["pct"] > 75:
        warn = "\n\n⚠️ Vas llenándote. Con >20% libre el sistema respira mejor."
    return Response(text=f"Almacenamiento: 💾{warn}",
                    stats=stats, chips=chips)


def h_gpu(brain, n, raw):
    g = system.gpu_info()
    if not g:
        return Response(text="No pude detectar la GPU (¿lspci instalado?).", status="warn")
    return Response(text=f"Tu(s) tarjeta(s) gráfica(s): 🎮\n\n$ {g.replace(chr(10), chr(10)+'$ ')}")


def h_bateria(brain, n, raw):
    b = system.battery()
    if not b:
        return Response(text=(
            "No encontré batería (¿es un PC de sobremesa?). 🔌"
        ))
    estado = b["status"]
    sev = "danger" if (b["percent"] <= 20 and estado == "Discharging") else \
          ("good" if b["percent"] >= 95 or "Charging" in estado else None)
    stats = [(f"Batería {b['icon']}", b["percent"], estado, sev)]
    extra = ""
    if b["percent"] <= 20 and estado == "Discharging":
        extra = "\n\n⚠️ Batería baja: enchúfala antes de que se apague."
    elif b["percent"] >= 95:
        extra = "\n\n✅ Más que lista para desconectar el cargador."
    return Response(text=f"Estado de la batería:{extra}",
                    stats=stats)

# ══════════════════════════════════════════════
# PAQUETES
# ══════════════════════════════════════════════

def h_paquetes(brain, n, raw):
    count = system.package_count()
    if not count:
        return Response(text="No pude consultar pacman (¿está instalado?).", status="error")
    recientes = system.recent_packages(3)
    extra = ""
    if recientes:
        names = ", ".join(f"{n_} ({v})" for n_, v in recientes)
        extra = f"\n\nÚltimas instalaciones:\n$ {names}"
    return Response(text=(
        f"Tienes **{count} paquetes** instalados. 📦{extra}\n\n"
        "> Cada uno es una pieza de tu sistema.\n"
        "> ¿Quieres ver cuáles ocupan más?"
    ), chips=["paquetes más grandes", "huérfanos", "busca un paquete"])


def h_paquetes_grandes(brain, n, raw):
    sizes = system.biggest_packages()
    if not sizes:
        return Response(text="No pude calcular los tamaños de los paquetes.",
                        status="warn")
    lines = []
    for name, mib in sizes:
        size = f"{mib/1024:.1f} GB" if mib >= 1024 else f"{mib:.0f} MiB"
        lines.append(f"$ {name:<24} {size}")
    return Response(text=(
        "Los paquetes que más espacio ocupan: 📦⚖️\n\n"
        "> Grande no significa innecesario: muchos son\n"
        "> librerías compartidas. No los borres sin saber."
    ), code="\n".join(lines),
        chips=["espacio en disco", "limpia la caché"])


def h_cache(brain, n, raw):
    pc = system.pacman_cache_size()
    uc = system.user_cache_size()
    lines = []
    if pc:
        lines.append(f"$ Caché de pacman (/var/cache/pacman): {pc}")
    if uc:
        lines.append(f"$ Caché de usuario (~/.cache): {uc}")
    if not lines:
        return Response(text="No pude medir las cachés.", status="warn")
    resp_text = "Tus cachés ocupan lo siguiente: 🧹\n"
    actions = []
    if pc and has_tool("paccache"):
        actions.append(Action(
            label="Limpiar caché de pacman",
            command="sudo paccache -r",
            level=CONFIRM,
            mode=MODE_TERMINAL,
            description=("Borra versiones antiguas de paquetes descargados, "
                         "conservando las 3 últimas. Seguro y reversible."),
        ))
    return Response(text=resp_text, code="\n".join(lines),
                    actions=actions,
                    chips=["espacio en disco", "explica paccache"])


def h_actualizaciones(brain, n, raw):
    updates = system.available_updates()
    if updates is None:
        return Response(text=(
            "`checkupdates` no está instalado (viene en `pacman-contrib`)."
        ), status="warn", chips=["instala pacman-contrib"])
    if not updates:
        return Response(text=(
            "¡Todo actualizado! ✨✅\nTu sistema está al día."
        ), chips=["uptime"])
    shown = ", ".join(updates[:8])
    more = f"\n… y {len(updates) - 8} más" if len(updates) > 8 else ""
    return Response(text=(
        f"Hay **{len(updates)} paquetes** con actualización disponible. ⬆️\n\n"
        f"$ {shown}{more}"
    ), code="", chips=["actualiza el sistema"])


def h_huerfanos(brain, n, raw):
    orphans = system.orphan_packages()
    if orphans is None:
        orphans = []
    if not orphans:
        return Response(text=(
            "No tienes paquetes huérfanos. ✨\nTu sistema está limpio de dependencias sin uso."
        ))
    shown = "\n".join(f"$ {o}" for o in orphans[:10])
    more = f"\n… y {len(orphans) - 10} más" if len(orphans) > 10 else ""
    action = Action(
        label=f"Eliminar {len(orphans)} huérfano(s)",
        command="sudo pacman -Rns $(pacman -Qtdq)",
        level=DANGEROUS,
        mode=MODE_TERMINAL,
        description=("Elimina paquetes instalados como dependencia que ninguna "
                     "aplicación usa ya."),
        danger_note=("Revisa la lista antes de confirmar: si algo te interesa, "
                     "instálalo explícitamente para adoptarlo."),
    )
    return Response(text=(
        f"Tienes **{len(orphans)} paquete(s) huérfano(s)** 👻\n\n"
        "> Son dependencias que quedaron solas al desinstalar\n"
        "> otros programas. No hacen daño, pero ocupan espacio."
    ), code=shown + more, actions=[action])


def h_buscar_paquete(brain, termino):
    results = system.search_packages(termino.strip())
    if results is None:
        return Response(text="No pude buscar (¿falló pacman -Ss?).", status="error")
    if not results:
        return Response(text=(
            f"No encontré nada llamado «{termino}» en los repositorios. 🔍\n"
            "Prueba con otras palabras o revisa el nombre."
        ), chips=[f"busca {' '.join(termino.split()[:2])}"])
    lines = [f"$ {name}\n  {desc[:70]}" for name, desc in results]
    first_pkg = results[0][0].split("/")[-1]
    return Response(text=(
        f"Encontré esto para «{termino}»: 🔍\n"
    ), code="\n".join(lines), chips=[
        f"instala {first_pkg}",
        f"info del paquete {first_pkg}",
    ])


def h_info_paquete(brain, pkg):
    info = system.package_info(pkg)
    if not info:
        return Response(text=(f"No encuentro información de «{pkg}». 🤷"), status="warn")
    keys = ("Nombre", "Name", "Versión", "Version", "Descripción", "Description",
            "Tamaño de la instalación", "Tamaño instalado", "Installed Size",
            "Depende de", "Depends On")
    filtered = [l for l in info.splitlines() if l.startswith(keys)]
    return Response(text=f"Esto sé de **{pkg}**: 📦", code="\n".join(filtered[:8]),
                    chips=[f"instala {pkg}"])


# ══════════════════════════════════════════════
# RED
# ══════════════════════════════════════════════

def h_mi_ip(brain, n, raw):
    ip = system.local_ip()
    host = system.hostname() or "?"
    if not ip:
        return Response(text="No encontré una dirección IP activa. 📡", status="warn")
    return Response(text=(
        f"Tu IP local es **{ip}** 📡\n(Equipo: {host})\n\n"
        "> Esto se leyó de tu sistema: cero Internet usado. 🔒"
    ), chips=["redes wifi", "estado del internet"])


def h_interfaces_red(brain, n, raw):
    out = system.network_interfaces()
    if not out:
        return Response(text="No pude leer las interfaces de red.", status="error")
    lines = [l for l in out.splitlines() if l.strip()][:6]
    return Response(text="Tus interfaces de red: 🌐", code="\n".join(lines))


def h_wifi(brain, n, raw):
    nets = system.wifi_networks()
    if nets is None:
        return Response(text=(
            "Necesito NetworkManager (`nmcli`) para escanear redes."
        ), status="warn")
    if not nets:
        return Response(text="No encontré redes visibles ahora mismo. 📶")
    lines = []
    for ssid, signal in nets:
        try:
            pct = int(signal)
        except ValueError:
            pct = 0
        icon = "📶" if pct >= 60 else ("📶" if pct >= 30 else "🥱")
        lines.append(f"$ {icon} {ssid:<24} señal {pct}%")
    return Response(text=f"Redes WiFi a tu alcance: {len(nets)} encontradas.",
                    code="\n".join(lines))


def h_internet(brain, n, raw):
    ok = system.internet_connected()
    if ok:
        up = system.uptime_pretty()
        return Response(text=(
            "✅ Tienes conexión a Internet.\n"
            "(Hice un ping silencioso a 1.1.1.1 para comprobarlo.)"
        ), chips=["mi ip", "actualizaciones disponibles"])
    return Response(text=(
        "❌ Parece que NO hay conexión a Internet.\n\n"
        "> Revisa el cable/WiFi o prueba `nmcli device wifi connect <red>`"
    ), status="error", chips=["redes wifi", "interfaces de red"])


def h_bluetooth(brain, n, raw):
    st = system.bluetooth_state()
    if not st:
        return Response(text="Bluetooth no disponible (¿`bluetoothctl` está instalado?).",
                        status="warn")
    return Response(text=f"Estado de Bluetooth: 📶\n\n$ {st}")


def h_audio_estado(brain, n, raw):
    st = system.audio_state()
    if not st:
        return Response(text=(
            "No pude leer el volumen (¿`wpctl` de PipeWire está instalado?)."
        ), status="warn", chips=["sube el volumen"])
    return Response(text=f"Estado del audio: 🔊\n\n$ Volumen → {st}",
                    chips=["sube el volumen", "silencia"])


# ══════════════════════════════════════════════
# DIAGNÓSTICO — FASE 10
# ══════════════════════════════════════════════

def _cpu_throttling() -> str | None:
    """Compara frecuencia actual vs máxima del primer núcleo (sysfs)."""
    base = Path("/sys/devices/system/cpu/cpu0/cpufreq")
    try:
        cur = int((base / "scaling_cur_freq").read_text().strip())
        mx = int((base / "cpuinfo_max_freq").read_text().strip())
    except (OSError, ValueError):
        return None
    if mx <= 0:
        return None
    pct = cur / mx * 100
    if pct < 65:
        return f"📉 CPU al **{pct:.0f}%** de su frecuencia máxima — posible throttling térmico o ahorro de energía."
    return None


def h_diagnostico(brain, n, raw):
    hallazgos = []
    usage = system.cpu_usage()
    if usage is not None and usage >= 70:
        top = system.top_processes("cpu", 2)
        who = ", ".join(p["name"] for p in top) if top else "varios procesos"
        hallazgos.append(f"⚙️ CPU al **{usage:.0f}%** — sospechosos: {who}")
    throttle = _cpu_throttling()
    if throttle and usage is not None and usage >= 50:
        hallazgos.append(throttle)
    ram = system.ram_report()
    if ram and ram["pct"] >= 80:
        hallazgos.append(f"🐏 RAM al **{ram['pct']:.0f}%** — vas justo de memoria.")
    disk = system.disk_report("/")
    if disk and disk["pct"] >= 85:
        hallazgos.append(f"💾 Disco al **{disk['pct']:.0f}%** — lleno, va a ralentizar todo.")
    temp = system.cpu_temp()
    if temp and temp[0] >= 80:
        hallazgos.append(f"🌡️ Temperatura **{temp[0]:.0f}°C** — posible throttling.")
    failed = system.failed_services()
    if failed:
        hallazgos.append(f"⚠️ {len(failed)} servicio(s) caído(s): {', '.join(failed[:3])}")
    load_txt = ""
    loadavg = open("/proc/loadavg").read().split()[0] if Path("/proc/loadavg").exists() else None

    if not hallazgos:
        return Response(text=(
            "Buenas noticias: **no encuentro nada raro**. ✨\n\n"
            "$ CPU, RAM, disco y temperatura en niveles sanos\n"
            "$ Sin servicios caídos\n\n"
            "> Si aun así lo notas lento, dime qué aplicación\n"
            "> te va mal y lo investigamos juntos."
        ), chips=["procesos top", "temperatura cpu"])

    text_lines = ["Hice una revisión completa y encontré esto: 🔍\n"]
    for h_ in hallazgos:
        text_lines.append(h_)
    text_lines.append("\n> Diagnóstico hecho leyendo tu sistema en vivo,\n"
                      "> sin IA ni servicios externos.")
    return Response(
        text="\n".join(text_lines),
        status="warn",
        chips=["procesos top", "paquetes grandes", "errores recientes"],
    )


def h_servicios_fallidos(brain, n, raw):
    failed = system.failed_services()
    if failed is None:
        failed = []
    if not failed:
        return Response(text=(
            "Todos los servicios están en pie. ✅\nNada fallido en systemd."
        ), chips=["errores recientes"])
    lines = "\n".join(f"$ {u}" for u in failed[:8])
    return Response(text=(
        f"Hay **{len(failed)} servicio(s) caído(s)**: ⚠️\n\n"
        "> Para investigar uno: `systemctl status <servicio>`\n"
        "> Yo puedo explicarte el comando si quieres."
    ), code=lines, chips=["errores recientes", "explica systemctl"])

# ══════════════════════════════════════════════
# ACCIONES — con confirmación (FASE 11)
# ══════════════════════════════════════════════

# Paquetes críticos: si se actualizan, conviene reiniciar o al menos
# reiniciar servicios. Inspirado en la clasificación de riesgo que
# hacen las herramientas tipo archward.
RISK_HIGH = (
    "glibc", "systemd", "linux", "mesa", "openssl", "openssh",
    "bash", "sudo", "dbus", "gnutls", "util-linux", "coreutils",
    "shadow", "pacman", "systemd-libs", "zstd", "xz", "e2fsprogs",
)
RISK_MED_PREFIX = (
    "docker", "nginx", "postgresql", "mariadb", "mysql", "php",
    "python", "pipewire", "xorg", "wayland", "hyprland", "sddm",
    "greetd", "libreoffice", "chromium", "firefox", "brave",
)


def classify_update_risk(names: list[str]) -> dict:
    """Clasifica paquetes por riesgo de la actualización."""
    high, med, low = [], [], []
    for name in names:
        base = name.split("-")[0]
        if name.startswith(RISK_HIGH) or base in RISK_HIGH or \
                name.startswith(("linux", "nvidia")):
            high.append(name)
        elif any(name.startswith(p) for p in RISK_MED_PREFIX):
            med.append(name)
        else:
            low.append(name)
    return {"high": high, "med": med, "low": low}


def a_actualizar(brain, n, raw):
    updates = system.available_updates()
    n_updates = len(updates) if updates else None

    risk_lines = []
    reboot_hint = ""
    if updates is not None and n_updates:
        risks = classify_update_risk(updates)

        def fmt(bucket):
            shown = ", ".join(bucket[:4])
            extra = f" (+{len(bucket)-4})" if len(bucket) > 4 else ""
            return f"{shown}{extra}" if bucket else "—"

        if risks["high"]:
            risk_lines.append(f"$ 🔴 Riesgo alto ({len(risks['high'])}): {fmt(risks['high'])}")
            reboot_hint = ("\n\n> Toca el kernel/librerías base:\n"
                           "> **reinicia cuando termines** para usar el nuevo kernel.")
        if risks["med"]:
            risk_lines.append(f"$ 🟡 Riesgo medio ({len(risks['med'])}): {fmt(risks['med'])}")
        if risks["low"]:
            risk_lines.append(f"$ 🟢 Riesgo bajo: {len(risks['low'])}")

    hint = f"\n\nAhora mismo hay **{n_updates}** paquetes esperando. ⬆️" if n_updates else ""
    risk_text = ("\n\n**Radar de riesgo:**\n" + "\n".join(risk_lines)) if risk_lines else ""

    return Response(text=(
        "Voy a actualizar el sistema completo. 🔄\n\n"
        "$ sudo pacman -Syu\n"
        "Esto sincroniza los repositorios y actualiza todos\n"
        "los paquetes instalados." + hint + risk_text + reboot_hint
    ), actions=[Action(
        label="Actualizar sistema",
        command="sudo pacman -Syu",
        level=CONFIRM,
        mode=MODE_TERMINAL,
        description="Actualización completa de Arch. Es la operación de mantenimiento más importante.",
    )], chips=["explica sudo pacman -Syu", "archivos pacnew", "actualizaciones disponibles"])


def a_instalar(brain, pkg):
    pkg = pkg.strip()
    if not PKG_OK.fullmatch(pkg):
        return Response(text=(
            f"«{pkg}» no parece un nombre de paquete válido. 🤔\n"
            "¿Lo buscamos primero?"
        ), chips=[f"busca {pkg.split()[0] if pkg.split() else 'algo'}"])
    return Response(text=(
        f"Voy a instalar **{pkg}**. 📥\n\n"
        f"$ sudo pacman -S --needed {pkg}\n"
        "`--needed` evita reinstalar si ya lo tienes al día."
    ), actions=[Action(
        label=f"Instalar {pkg}",
        command=f"sudo pacman -S --needed {pkg}",
        level=CONFIRM,
        mode=MODE_TERMINAL,
        description=f"Instala {pkg} desde los repositorios oficiales.",
    )], chips=[f"busca {pkg}", "explica pacman"])


def a_eliminar(brain, pkg):
    pkg = pkg.strip()
    if not PKG_OK.fullmatch(pkg):
        return Response(text="Ese nombre no parece un paquete válido. 🤔")
    return Response(text=(
        f"Voy a eliminar **{pkg}** del sistema. 🗑️\n\n"
        f"$ sudo pacman -Rns {pkg}\n"
        "-R elimina · -n borra configuraciones · -s quita dependencias huérfanas."
    ), actions=[Action(
        label=f"Eliminar {pkg}",
        command=f"sudo pacman -Rns {pkg}",
        level=DANGEROUS,
        mode=MODE_TERMINAL,
        description=(f"Desinstala {pkg} junto a su configuración y las "
                     "dependencias que queden sin uso."),
        danger_note=("Otros programas podrían necesitar esas dependencias. "
                     "Pacman abortará si algo se rompe, pero revisa la lista."),
    )], chips=[f"info del paquete {pkg}"])


def a_recargar_hyprland(brain, n, raw):
    if not system.is_hyprland():
        return Response(text="No estoy corriendo dentro de Hyprland. 🪟")
    return Response(text=(
        "Voy a recargar la configuración de Hyprland. ♻️\n\n$ hyprctl reload\n"
        "Aplica cambios sin cerrar tus ventanas."
    ), actions=[Action(
        label="Recargar Hyprland",
        command="hyprctl reload",
        level=CONFIRM,
        mode=MODE_INLINE,
        description="Recarga ~/.config/hypr/hyprland.conf en caliente.",
    )])


def a_captura(brain, n, raw):
    if not has_tool("grim"):
        return Response(text=(
            "Para capturas en Wayland uso `grim` y no está instalado. 📸\n\n"
            "$ sudo pacman -S grim slurp\n"
            "(grim captura, slurp te deja elegir región)"
        ), chips=["instala grim"])
    home = Path.home()
    base = home / "Imágenes"
    if not base.exists():
        base = home / "Pictures"
    shots = base / "Screenshots"
    try:
        shots.mkdir(parents=True, exist_ok=True)
    except OSError:
        shots = home
    fname = datetime.datetime.now().strftime("archito_%Y%m%d_%H%M%S.png")
    path = shots / fname
    return Response(text=(
        "Captura de pantalla lista para disparar. 📸\n\n"
        f"$ grim \"{path}\""
    ), actions=[Action(
        label="Capturar ahora",
        command=f'grim "{path}"',
        level=CONFIRM,
        mode=MODE_SPAWN,
        description=f"Guarda una imagen de toda la pantalla en {path}.",
    )], chips=["instala slurp"])


def a_bloquear(brain, n, raw):
    for locker in ("hyprlock", "swaylock", "gtklock", "waylock"):
        if has_tool(locker):
            return Response(text=(
                f"Voy a bloquear la sesión con **{locker}**. 🔒"
            ), actions=[Action(
                label="Bloquear pantalla",
                command=locker,
                level=CONFIRM,
                mode=MODE_SPAWN,
                description="Bloquea la pantalla hasta que introduzcas tu contraseña.",
            )])
    return Response(text=(
        "No encontré un bloqueador (hyprlock/swaylock...). 🔒\n"
        "Instala uno: `sudo pacman -S hyprlock`"
    ), chips=["instala hyprlock"])


def _volume_action(direction: str) -> Response:
    if not has_tool("wpctl"):
        return Response(text=(
            "Necesito `wpctl` (PipeWire) para controlar el volumen."
        ), status="warn")
    cmd = {
        "up": "wpctl set-volume -l 1.5 @DEFAULT_AUDIO_SINK@ 10%+",
        "down": "wpctl set-volume @DEFAULT_AUDIO_SINK@ 10%-",
        "mute": "wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle",
    }[direction]
    verb = {"up": "Subiendo", "down": "Bajando", "mute": "Cambiando silencio de"}[direction]
    _exec_now(cmd)
    st = system.audio_state() or "?"
    icon = "🔊" if not st.startswith("🔇") and direction != "mute" else ""
    return Response(text=f"{verb} volumen... hecho. {st or ''}".strip(),
                    chips=["volumen actual", "baja el volumen"] if direction == "up"
                    else ["volumen actual", "sube el volumen"])


def h_volumen(brain, n, raw):
    if re.search(r"(sube|subir|mas|aumenta)", n):
        return _volume_action("up")
    if re.search(r"(baja|bajar|menos|reduce|disminuye)", n):
        return _volume_action("down")
    if re.search(r"(silencia|muto?|desilencia|quita el sonido|sin sonido)", n):
        return _volume_action("mute")
    return h_audio_estado(brain, n, raw)


def _brightness_action(direction: str):
    if not has_tool("brightnessctl"):
        return Response(text=(
            "Necesito `brightnessctl` para controlar el brillo.\n"
            "$ sudo pacman -S brightnessctl"
        ), status="warn", chips=["instala brightnessctl"])
    sign = "+10%" if direction == "up" else "-10%"
    out = _exec_now(f"brightnessctl set {sign}")
    info = _exec_now("brightnessctl info") or ""
    m = re.search(r"\((\d+)%\)", info)
    cur = f"**{m.group(1)}%**" if m else "ajustado"
    emoji = "☀️" if direction == "up" else "🌙"
    return Response(text=f"Brillo ajustado {emoji} — ahora: {cur}",
                    chips=["sube el brillo"] if direction == "down"
                    else ["baja el brillo"])


def h_brillo(brain, n, raw):
    if re.search(r"(sube|mas|aumenta|maxim)", n):
        return _brightness_action("up")
    if re.search(r"(baja|menos|reduce|minim|oscurece)", n):
        return _brightness_action("down")
    return _brightness_action("up")


# ══════════════════════════════════════════════
# PROFESOR — explicar comandos (FASE 9)
# ══════════════════════════════════════════════

def h_explicar(brain, objetivo):
    from archito.core.explain import explain_command
    text = explain_command(objetivo)
    return Response(text=text, chips=["consejo", "ayuda"])


# ══════════════════════════════════════════════
# AMPLIACIONES v0.4 — lo especial, no lo genérico
# ══════════════════════════════════════════════

def h_pacnew(brain, n, raw):
    """
    Detector de archivos .pacnew — la tarea de mantenimiento más
    olvidada de Arch (y la que rompe sistemas silenciosamente).
    """
    out = _exec_now("find /etc -name '*.pacnew' 2>/dev/null")
    files = [l for l in out.splitlines() if l.strip()] if out else []

    if not files:
        return Response(text=(
            "No hay archivos **.pacnew** pendientes. ✨\n\n"
            "> Cuando pacman actualiza un paquete cuya configuración\n"
            "> editaste, guarda la nueva versión como .pacnew.\n"
            "> Si se acumulan sin revisar, pierdes ajustes nuevos."
        ), chips=["actualiza el sistema"])

    shown = "\n".join(f"$ {f}" for f in files[:8])
    more = f"\n… y {len(files) - 8} más" if len(files) > 8 else ""
    return Response(text=(
        f"Tienes **{len(files)} archivo(s) .pacnew** sin fusionar. 📂⚠️\n\n"
        "Son versiones nuevas de configuraciones que TÚ editaste.\n"
        "Revisarlas evita sorpresas tras reiniciar."
    ), code=shown + more,
        actions=[Action(
            label="Revisar con pacdiff",
            command="sudo pacdiff",
            level=CONFIRM,
            mode=MODE_TERMINAL,
            description=("pacdiff te muestra las diferencias una a una: "
                         "puedes ver (v), fusionar, reemplazar o saltar."),
        )],
        chips=["explica pacdiff", "errores recientes"])


def h_seguridad(brain, n, raw):
    """Auditoría CVE local con arch-audit (si está instalado)."""
    if not has_tool("arch-audit"):
        return Response(text=(
            "Para auditar vulnerabilidades uso `arch-audit`\n"
            "(consulta el security tracker de Arch) y no está instalado. 🛡️\n\n"
            "$ sudo pacman -S arch-audit"
        ), chips=["instala arch-audit", "actualiza el sistema"])

    out = _exec_now("arch-audit")
    if not out:
        return Response(text=(
            "`arch-audit` no reportó paquetes vulnerables conocidos. ✅🛡️\n\n"
            "> Igual conviene actualizar: los CVEs nuevos salen cada semana."
        ), chips=["actualiza el sistema", "actualizaciones disponibles"])

    lines = [l for l in out.splitlines() if l.strip()]
    high = sum(1 for l in lines if "critical" in l.lower() or "high" in l.lower())
    return Response(text=(
        f"**{len(lines)} paquete(s)** con vulnerabilidades conocidas. 🛡️⚠️"
        + (f"\nDe ellos, **{high}** son de severidad alta o crítica." if high else "")
    ), code="\n".join(l[:90] for l in lines[:8]) + ("\n…" if len(lines) > 8 else ""),
        chips=["actualiza el sistema", "explica arch-audit"])


def h_boot_blame(brain, n, raw):
    blame = _exec_now("systemd-analyze blame | head -7")
    critical = _exec_now("systemd-analyze critical-chain | head -4")
    if not blame:
        return Response(text="`systemd-analyze` no respondió.", status="warn")
    text = "Los servicios que más tardan en arrancar: 🐢⏱️\n\n"
    code = "$ " + "\n$ ".join(blame.splitlines())
    if critical:
        code += "\n\n" + "\n".join(critical.splitlines()[:3])
    return Response(text=text, code=code,
                    chips=["tiempo de arranque", "servicios fallidos"],
                    status="ok")


def h_historial(brain, n, raw):
    """Historiador de pacman.log: qué instalaste hoy/ayer/esta semana."""
    import datetime as _dt
    low = n
    days = 0
    if "ayer" in low:
        days = 1
    elif "semana" in low or "dias" in low or "días" in low:
        days = 7
    else:
        days = 0  # hoy

    today = _dt.date.today()
    target = today - _dt.timedelta(days=days)
    prefix = target.isoformat()

    out = _exec_now("grep 'installed' /var/log/pacman.log")
    if not out:
        return Response(text="No pude leer /var/log/pacman.log.", status="warn")

    installs = []
    for line in out.splitlines():
        m = re.match(r"\[(\d{4}-\d{2}-\d{2})T[^\]]+\] \[ALPM\] installed (\S+) \((\S+)\)", line)
        if m and m.group(1) == prefix:
            installs.append((m.group(2), m.group(3)))

    when = {0: "hoy", 1: "ayer"}.get(days, f"el {_dt.timedelta(days=days) and prefix}")
    if not installs:
        return Response(text=f"No instalaste nada {when}. 📦✨",
                        chips=["cuántos paquetes", f"instalaciones de esta semana"]
                        if days == 0 else ["cuántos paquetes"])

    shown = "\n".join(f"$ {p_} ({v})" for p_, v in installs[:10])
    more = f"\n… y {len(installs) - 10} más" if len(installs) > 10 else ""
    return Response(text=f"Instalaste **{len(installs)} paquete(s)** {when}: 📜",
                    code=shown + more)


# ── Comandos personales (plugins FASE 14) ──

def h_mis_comandos(brain, n, raw):
    from archito.core.custom import load_commands
    cmds = brain.custom_commands if hasattr(brain, "custom_commands") else {}
    if not cmds:
        return Response(text=(
            "Aún no tienes comandos personales. 🧩\n\n"
            "Créelos en `~/.config/archito/commands.json`:\n\n"
            "$ {\n"
            '  "backup fotos": {\n'
            '    "command": "rsync -a ~/Fotos /media/backup/",\n'
            '    "description": "Copia tus fotos."\n'
            "  }\n"
            "}\n\n"
            "Luego dime «recarga los comandos»."
        ), chips=["recarga los comandos", "ayuda"])
    lines = []
    for phrase, spec in list(cmds.items())[:10]:
        flag = " 🔴" if spec.get("dangerous") else ""
        desc = spec.get("description", "")
        lines.append(f"$ {phrase}{flag}" + (f"\n  {desc[:60]}" if desc else ""))
    return Response(text=(
        f"Tienes **{len(cmds)} comando(s) personal(es)**: 🧩\n\n"
        "> Se ejecutan siempre en terminal visible,\n"
        "> siempre pidiendo confirmación. Tus reglas, tu control."
    ), code="\n".join(lines))


def h_recarga_comandos(brain, n, raw):
    from archito.core.custom import load_commands
    cmds = load_commands()
    brain.custom_commands = cmds
    if not cmds:
        return Response(text=(
            "No encontré comandos en ~/.config/archito/commands.json.\n"
            "¿Quieres que te muestre cómo crearlos?"
        ), chips=["mis comandos"])
    return Response(text=f"Recargados **{len(cmds)} comando(s) personal(es)**. ✅🧩",
                    chips=["mis comandos"])


def a_custom(brain, phrase: str):
    spec = brain.custom_commands[phrase]
    danger = spec.get("dangerous", False)
    confirm_text = spec.get("confirm_text", "")
    return Response(text=(
        f"Ejecutaré tu comando personal **«{phrase}»**.\n\n"
        f"$ {spec['command']}"
    ), actions=[Action(
        label=f"Ejecutar «{phrase}»",
        command=spec["command"],
        level=DANGEROUS if danger else CONFIRM,
        mode=MODE_TERMINAL,
        description=spec.get("description")
        or "Comando personalizado definido por ti en commands.json.",
        danger_note=confirm_text or (
            "Comando personal: revísalo bien antes de confirmar." if danger else ""),
    )])


def h_modo_movil(brain, n, raw):
    current = brain.memory.touch_mode
    new = not current
    brain.memory.touch_mode = new
    resp = Response(
        text=("Modo táctil **activado** 📱\nBotones y texto más grandes:\n"
              "perfecto para pantallas pequeñas o dedos torpes.")
        if new else
        ("Modo táctil **desactivado** 🖥️\nVolvemos al diseño compacto de escritorio."),
        ui_command="sync-touch",
    )
    resp.chips = ["modo móvil"] if not new else ["ayuda"]
    return resp


# ══════════════════════════════════════════════
# TEMAS — la estética de la comunidad de ricing
# ══════════════════════════════════════════════

THEME_CHIP_ORDER = [
    ("tema catppuccin", "Catppuccin Mocha"),
    ("tema tokyo night", "Tokyo Night"),
    ("tema nord", "Nord"),
    ("tema rose pine", "Rosé Pine"),
    ("tema gruvbox", "Gruvbox"),
    ("tema dracula", "Dracula"),
    ("tema everforest", "Everforest"),
    ("tema noche", "Archito Noche ★"),
    ("tema auto", "Auto (pywal)"),
]


def h_temas(brain, n, raw):
    current = brain.memory.theme
    lines = []
    for phrase, label in THEME_CHIP_ORDER:
        key = {"tema catppuccin": "catppuccin", "tema tokyo night": "tokyo-night",
               "tema nord": "nord", "tema rose pine": "rose-pine",
               "tema gruvbox": "gruvbox", "tema dracula": "dracula",
               "tema everforest": "everforest", "tema noche": "archito-noche",
               "tema auto": "auto"}[phrase]
        star = "  ← actual" if key == current else ""
        lines.append(f"$ {label}{star}")
    return Response(text=(
        "Estos son mis temas — dilo en voz alta y me pongo\n"
        "los colores al instante (sin reiniciar): 🎨"
    ), code="\n".join(lines),
        chips=[p for p, _ in THEME_CHIP_ORDER[:5]])


def h_tema(brain, n, raw):
    from archito.core.themes import normalize_theme_name, resolve_auto

    # ¿Pide la lista?
    if any(w in n for w in ("lista de temas", "que temas", "temas disponibles",
                             "cambia el tema", "cambiar el tema")):
        return h_temas(brain, n, raw)

    target = normalize_theme_name(n)
    if target is None:
        return h_temas(brain, n, raw)

    if target == "auto":
        pal, source = resolve_auto()
        if pal is None:
            return Response(text=(
                "El modo **auto** necesita colores generados del wallpaper. 🖼️\n\n"
                "$ sudo pacman -S pywal\n"
                "$ wal -i tu-wallpaper.jpg\n\n"
                "> Después de eso, «tema auto» y combino contigo."
            ), chips=["temas disponibles"])
        text = (f"Modo **auto** activado con colores de **{source}**. 🖼️✨\n"
                "Me tiño con tu wallpaper.")
    else:
        from archito.core.themes import PALETTES
        label = PALETTES[target].label
        text = f"Tema **{label}** aplicado. 🎨✨"

    brain.memory.theme = target
    return Response(text=text,
                    ui_command=f"theme:{target}",
                    chips=["temas disponibles", "ayuda"])


# ══════════════════════════════════════════════
# EASTER EGGS — los huevos que hacen sonreír
# ══════════════════════════════════════════════

def h_cafe(brain, n, raw):
    return Response(text=(
        "      ( (\n"
        "       ) )\n"
        "    ........\n"
        "    |      |]\n"
        "    \\      /\n"
        "     `----'\n\n"
        "**sudo brew coffee --extra-hot** ☕\n\n"
        "> [sudo] password for archito:\n"
        "> Café servido. Productividad +100%.\n"
        "> Advertencia: la cafeína no se desinstala con pacman."
    ), chips=["consejo"])


def h_gentoo(brain, n, raw):
    return Response(text=pick(
        "¿Arch o Gentoo? 🍿 Yo soy de Arch, pero respeto a quien\n"
        "compila su propio mundo desde cero. Somos primos raros.",
        "Gentoo compila TODO... hasta la paciencia. 😏\n"
        "Aquí vivimos al borde: bleeding edge y sin miedo.",
        "Tranquilo, no se lo digas a Gentoo, pero\nentre nosotros... Arch es más rápido de instalar. 🐧",
    ))


def h_matrix(brain, n, raw):
    return Response(text=(
        "Despierta, Neo... 🐇\n\n"
        "$ whoami\narchito\n$ sudo pacman -S red-pill\n"
        "resolviendo dependencias...\n"
        ":: instalando libertad (1.0-1) [local]\n\n"
        "> Bienvenido al mundo real. Aquí todo es local.\n> Nada sale de tu máquina. Nunca."
    ), chips=["filosofía"])


def h_hacker(brain, n, raw):
    return Response(text=(
        "*tipea intensamente* ⌨️\n\n"
        "$ acceso concedido...\n"
        "Broma. Soy tan hacker que uso `less` para leer\n"
        "y `man` para enternderme. El verdadero poder\n"
        "es entender lo que ejecutas. 😉"
    ), chips=["consejo"])
    return resp


