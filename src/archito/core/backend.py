"""
Backend de Archito — el cerebro (FASE 4 + FASE 5 + FASE 7-lite).

Interpreta texto natural SIN IA:

    Usuario → Brain → intención estructurada → acción permitida
            → sistema (lectura inline / confirmación) → respuesta

Motor:
    1. Normalización (sin acentos, sin mayúsculas, sin puntuación).
    2. Patrones regulares para órdenes concretas
       (instala X, busca Y, explica Z, cuánto es...).
    3. Tabla de intenciones por palabras clave, priorizada por longitud.
    4. Contexto conversacional ("¿y la temperatura?", "otra vez").
    5. Memoria local (nombre, preferencias).
"""

import re

from archito.core import handlers as H
from archito.core import system
from archito.core.actions import Action, Response, CONFIRM, DANGEROUS, MODE_TERMINAL, classify
from archito.core.activity import ActivityLog
from archito.core.memory import Memory
from archito.core.textutils import DEFAULT_REPLIES, WELCOME_TEXT, norm, pick


# Palabras que empiezan una frase de continuación.
_CONTINUATION = re.compile(r"^(?:y|tambien|ahora|entonces|dame)\s+(.+)$")

_MATH_WORDS = ["sqrt", "sin", "cos", "tan", "log2", "log10", "log", "floor",
               "ceil", "abs", "round", "factorial", "pi"]


def _looks_math(n: str) -> bool:
    """True si `n` parece una expresión aritmética pura."""
    if not any(c.isdigit() for c in n):
        return False
    if not any(c in n for c in "+-*/^%("):
        return False
    cleaned = n
    for w in _MATH_WORDS:
        cleaned = re.sub(rf"\b{w}\b", "", cleaned)
    return bool(re.fullmatch(r"[\d\s+\-*/%^().,]+", cleaned.strip()))


class Brain:
    """El núcleo intérprete de Archito."""

    def __init__(self):
        self.memory = Memory()
        self.activity = ActivityLog()
        self.ai_server_state = "disabled"
        if self.memory.local_ai:
            from archito.core.ai import ensure_server
            self.ai_server_state = ensure_server()
        self._last_raw: str | None = None
        from archito.core.custom import load_commands
        self.custom_commands: dict[str, dict] = load_commands()

        # Tabla de intenciones: (id, palabras clave, handler).
        # El ORDEN importa cuando hay empate de longitud de keyword.
        self.intents = [
            # ── Conversación ──
            ("ayuda", ["ayuda", "help", "que puedes hacer", "que sabes hacer",
                        "comandos disponibles", "funciones", "menu"], H.h_ayuda),
            ("version", ["version de archito", "tu version", "archito version",
                          "que version eres"], H.h_version),
            ("filosofia", ["filosofia", "tu mision", "por que existes",
                            "tu objetivo", "tu lema"], H.h_filosofia),
            ("creador", ["quien te creo", "quien te hizo", "tu autor",
                          "quien programa"], H.h_creador),
            ("clima", ["clima", "pronostico", "tiempo va a hacer",
                        "va a llover"], H.h_clima),
            ("identidad", ["quien eres", "que eres", "tu nombre",
                            "como te llamas", "que haces"], H.h_identidad),
            ("comoestas", ["como estas", "que tal estas", "como te sientes",
                            "todo bien"], H.h_comoestas),
            ("gracias", ["gracias", "te lo agradezco", "thanks", "genial archito",
                          "buen trabajo", "bien hecho"], H.h_gracias),
            ("despedida", ["adios", "chao", "hasta luego", "me voy", "bye",
                            "nos vemos", "descansa"], H.h_despedida),
            ("saludo", ["hola", "buenas", "hey", "saludos", "que tal",
                         "buenos dias", "buenas tardes", "buenas noches",
                         "ey archito"], H.h_saludo),

            # ── Utilidades ──
            ("limpiachar", ["limpia el chat", "limpiar chat", "limpia la pantalla",
                             "borra el chat", "limpia la conversacion",
                             "nueva conversacion"], H.h_limpiar_chat),
            ("prefon", ["activa las explicaciones", "activa explicaciones",
                         "explicame todo", "modo principiante",
                         "quiero explicaciones"], H.h_preferencias_on),
            ("preffoff", ["desactiva las explicaciones", "desactiva explicaciones",
                           "modo experto", "sin explicaciones",
                           "no me expliques"], H.h_preferencias_off),
            ("hora", ["que hora es", "hora actual", "dime la hora", "la hora",
                       "que horas son", "hora"], H.h_hora),
            ("fecha", ["que dia es hoy", "fecha de hoy", "en que fecha",
                        "dia de la semana", "fecha"], H.h_fecha),
            ("moneda", ["lanza una moneda", "tira una moneda", "cara o cruz",
                         "lanzamiento de moneda"], H.h_moneda),
            ("dado", ["tira un dado", "lanza un dado", "tira un d20",
                       "dado", "un dado de"], H.h_dado),
            ("chiste", ["chiste", "broma", "hazme reir", "cuentame algo gracioso",
                         "otro chiste"], H.h_chiste),
            ("consejo", ["consejo", "tip", "truco", "dato curioso",
                          "otro consejo", "ensename algo"], H.h_consejo),

            # ── Sistema ──
            ("infosistema", ["info del sistema", "informacion del sistema",
                              "informacion de mi sistema", "datos del sistema",
                              "resumen del sistema", "info sistema",
                              "como esta mi sistema"], H.h_info_sistema),
            ("kernel", ["kernel", "version del kernel", "nucleo de linux"], H.h_kernel),
            ("distro", ["distro", "distribucion", "que linux tengo",
                         "version del sistema operativo", "que distro"], H.h_distro),
            ("hostname", ["hostname", "nombre del equipo", "nombre de la maquina",
                           "como se llama mi pc", "nombre de mi computadora"], H.h_hostname),
            ("usuariosconectados", ["usuarios conectados", "quien esta conectado",
                                     "sesiones activas", "quien mas usa"], H.h_usuarios_conectados),
            ("usuario", ["quien soy", "mi usuario", "usuario actual",
                          "con quien estoy", "mi nombre de usuario"], H.h_usuario),
            ("shell", ["shell", "que terminal uso", "bash o zsh", "mi consola"], H.h_shell),
            ("escritorio", ["escritorio", "entorno grafico", "window manager",
                             "workspace actual", "espacio de trabajo",
                             "mis monitores", "resolucion de pantalla"], H.h_escritorio),
            ("arranque", ["tiempo de arranque", "cuanto tarda en arrancar",
                           "arranque del sistema", "systemd analyze", "boot time"], H.h_boot),
            ("uptime", ["uptime", "tiempo encendido", "cuanto lleva encendida",
                         "encendido desde", "lleva encendido"], H.h_uptime),

            # ── Hardware ──
            ("temperatura", ["temperatura", "caliente", "se calienta",
                              "calentandose", "calor"], H.h_temperatura),
            ("procesosmemoria", ["procesos por memoria", "procesos que usan memoria",
                                  "memoria por proceso", "quien usa la ram",
                                  "quien consume memoria"], H.h_procesos),
            ("procesos", ["procesos", "top procesos", "que consume",
                           "uso de cpu", "consumo de cpu", "quien consume",
                           "proceso pesado"], H.h_procesos),
            ("cpu", ["cpu", "procesador", "microprocesador"], H.h_cpu),
            ("ram", ["ram", "memoria ram", "cuanta memoria", "memoria disponible",
                      "swap"], H.h_ram),
            ("disco", ["disco", "espacio en disco", "espacio libre", "almacenamiento",
                        "cuanto espacio", "particiones"], H.h_disco),
            ("gpu", ["gpu", "grafica", "tarjeta grafica", "tarjeta de video",
                      "aceleradora", "vga"], H.h_gpu),
            ("bateria", ["bateria", "autonomia", "energia restante",
                          "cuanta bateria"], H.h_bateria),

            # ── Paquetes ──
            ("paquetesgrandes", ["paquetes grandes", "paquetes mas grandes",
                                  "lo que mas ocupa", "paquetes que ocupan",
                                  "mas espacio"], H.h_paquetes_grandes),
            ("cache", ["cache", "caches", "limpia la cache", "limpiar cache",
                        "paccache"], H.h_cache),
            ("huerfanos", ["huerfano", "huerfanos", "dependencias sin uso",
                            "paquetes sin uso"], H.h_huerfanos),
            ("actualizaciones", ["actualizaciones", "actualizacion pendiente",
                                  "hay actualizaciones", "que hay por actualizar",
                                  "updates pendientes", "paquetes desactualizados"], H.h_actualizaciones),
            ("paquetes", ["cuantos paquetes", "paquetes instalados",
                           "numero de paquetes", "lista de paquetes",
                           "total de paquetes"], H.h_paquetes),

            # ── Red ──
            ("wifi", ["redes wifi", "redes disponibles", "redes cercanas",
                       "escanea redes", "listar redes wifi", "wifi disponibles"], H.h_wifi),
            ("internet", ["internet", "hay conexion", "conexion a internet",
                           "estoy conectado", "estoy en linea", "tengo red"], H.h_internet),
            ("miip", ["mi ip", "direccion ip", "cual es mi ip", "ip local",
                       "ip privada", "mi direccion ip"], H.h_mi_ip),
            ("interfacesred", ["interfaces de red", "estado de la red",
                                "adaptadores de red", "dispositivos de red"], H.h_interfaces_red),
            ("bluetooth", ["bluetooth", "bt estado"], H.h_bluetooth),
            ("audioestado", ["volumen actual", "en que volumen", "estado del audio",
                              "audio actual", "cuanto volumen"], H.h_audio_estado),

            # ── Diagnóstico ──
            ("diagnostico", ["por que va lento", "esta lento", "va lenta",
                              "por que esta lento", "rendimiento", "diagnostico",
                              "revisa mi sistema", "algo raro", "va lento"], H.h_diagnostico),
            ("errores", ["errores recientes", "errores del sistema",
                          "ultimos errores", "journal", "logs", "registro de errores",
                          "por que fallo"], H.h_errores_recientes),
            ("serviciosfallidos", ["servicios fallidos", "servicio caido",
                                    "servicios caidos", "failed services",
                                    "servicios con errores", "algo fallando",
             "servicios rotos", "systemd failed"], H.h_servicios_fallidos),
            ("salud", ["estado de salud", "salud del sistema", "estado del sistema",
                        "sistema necesita atencion", "revision del sistema",
                        "chequeo del sistema", "como esta mi pc"], H.h_salud),
            ("actividad", ["mi actividad", "que hice", "que hizo archito",
                            "historial de acciones", "ultimas acciones",
                            "bitacora"], H.h_actividad),

            # ── Acciones ──
            ("recargarhypr", ["recarga hyprland", "recargar hyprland",
                               "hyprctl reload", "reload hyprland",
                               "aplica la configuracion", "recarga la config"], H.a_recargar_hyprland),
            ("captura", ["captura de pantalla", "captura", "screenshot",
                          "pantallazo", "foto de pantalla", "toma una captura"], H.a_captura),
            ("bloquear", ["bloquea la pantalla", "bloquear pantalla",
                           "lock screen", "cierra mi sesion", "bloquea sesion"], H.a_bloquear),
            ("volumen", ["volumen", "silencia", "mutea", "mute", "sin sonido",
                          "desilencia"], H.h_volumen),
            ("brillo", ["brillo", "luminosidad"], H.h_brillo),
            ("actualizar", ["actualiza el sistema", "actualizar el sistema",
                             "actualiza todo", "haz un update", "update",
                             "upgrade", "pacman -syu", "actualizate",
                             "pon al dia el sistema"], H.a_actualizar),

            # ── Ampliaciones v0.4 — lo especial ──
            ("pacnew", ["pacnew", "pacsave", "archivos sin fusionar",
                         "configs sin fusionar", "configuraciones pendientes"],
             H.h_pacnew),
            ("seguridad", ["vulnerabilidades", "cve", "arch-audit",
                            "paquetes vulnerables", "auditoria de seguridad",
                            "seguridad de mis paquetes", "fallos de seguridad"],
             H.h_seguridad),
            ("arranquelento", ["que hace lento el arranque", "blame",
                                "que tarda en arrancar", "arranque lento",
                                "demora el arranque"], H.h_boot_blame),
            ("historialpkg", ["que instale hoy", "que instale ayer",
                               "instalado hoy", "instale ayer",
                               "historial de paquetes", "instalaciones recientes",
                               "instalaciones de esta semana",
                               "ultimos instalados", "que he instalado"],
             H.h_historial),
            ("miscomandos", ["mis comandos", "comandos personalizados",
                              "comandos propios", "mis plugins",
                              "mis macros"], H.h_mis_comandos),
            ("recargacomandos", ["recarga los comandos", "recargar comandos",
                                  "reload custom", "recarga plugins"], H.h_recarga_comandos),
            ("modomovil", ["modo movil", "modo tactil", "modo telefono",
                            "activa el modo movil", "desactiva el modo movil",
                            "pantalla pequena"], H.h_modo_movil),
            ("tema", ["tema", "temas disponibles", "lista de temas",
                       "catppuccin", "tokyo night", "tokio", "nord",
                       "rose pine", "rosé pine", "gruvbox", "dracula",
                       "everforest", "pywal", "matugen",
                       "colores de archito"], H.h_tema),

            # ── Easter eggs ──
            ("cafe", ["cafe", "coffee", "un cafecito"], H.h_cafe),
            ("gentoo", ["gentoo"], H.h_gentoo),
            ("matrix", ["matrix", "pastilla roja", "red pill"], H.h_matrix),
            ("hacker", ["hackea", "hazme hacker", "modo hacker"], H.h_hacker),
        ]

    # ──────────────────────────────────────────────
    # API pública
    # ──────────────────────────────────────────────

    def welcome(self) -> Response:
        return Response(text=WELCOME_TEXT,
                        chips=["ayuda", "info del sistema", "chiste"])

    def explain(self, command: str) -> Response:
        from archito.core.explain import explain_command
        return Response(text=explain_command(command), chips=["ayuda"])

    def process(self, raw: str) -> Response:
        n = norm(raw)

        if not n:
            return Response(text="No me llegó nada. ¿Me lo repites? 👂")

        resp = self._route(n, raw)
        if resp is None:
            resp = self._fallback(n, raw)

        # FASE 12 + 9: si el usuario quiere explicaciones, se las añadimos
        if resp.actions and self.memory.prefer_explanations:
            for action in resp.actions:
                if action.level != "safe":
                    from archito.core.explain import explain_command
                    extra = explain_command(action.command)
                    resp.followup = (
                        "> Modo profesor 🎓 — así funciona el comando:\n\n" + extra
                    )
                    break

        if not resp.clear_chat and resp.status != "bye":
            self._last_raw = raw
        return resp

    # ──────────────────────────────────────────────
    # Enrutado
    # ──────────────────────────────────────────────

    def _route(self, n, raw):
        # 1. Repetir la última consulta
        if re.fullmatch(r"(otra vez|repitel[oa]|repite|de nuevo|repetir)", n):
            if self._last_raw:
                last = self._last_raw
                self._last_raw = None
                return self.process(last)
            return Response(text="Todavía no me has preguntado nada. 😅")

        # 2. Gestión del nombre (FASE 12)
        m = re.match(r"^(?:me llamo|mi nombre es)\s+(\w{1,24})$", n) \
            or re.match(r"^soy\s+(\w{1,24})$", n)
        if m and m.group(1) not in ("archito", "linux"):
            name = m.group(1).capitalize()
            self.memory.user_name = name
            return Response(text=(
                f"¡Encantado, **{name}**! 🤝\nLo apunto en mi memoria local "
                "(~/.config/archito/).\nPuedes borrarlo cuando quieras con \"olvida mi nombre\"."
            ), chips=["info del sistema", "ayuda"])

        if re.match(r"^(?:olvida|borra)\s+(?:mi nombre|quien soy)", n):
            old = self.memory.user_name or "..."
            self.memory.user_name = None
            return Response(text=f"Hecho: olvidé tu nombre ({old}). 🗑️")

        # IA local opcional. No forma parte de la ruta de acciones ni puede
        # ejecutar comandos: solo enriquece conversaciones no reconocidas.
        if re.search(r"\b(?:desactiva|apaga)\s+(?:la\s+)?ia(?:\s+local)?\b", n):
            self.memory.local_ai = False
            return Response(text="IA local desactivada. Vuelvo a mi modo determinista. 🐧")
        if re.search(r"\b(?:activa|enciende)\s+(?:la\s+)?ia(?:\s+local)?\b", n):
            self.memory.local_ai = True
            from archito.core.ai import ensure_server
            self.ai_server_state = ensure_server()
            return Response(text=("IA local activada. Si Ollama está encendido, usaré "
                                  f"**{self.memory.local_ai_model}** para preguntas abiertas."),
                            chips=["estado de la ia"])
        if re.search(r"\b(?:estado|modelo)\s+(?:de\s+)?(?:la\s+)?ia\b", n):
            from archito.core.ai import status
            model = status(self.memory.local_ai_model)
            if model:
                return Response(text=f"IA local lista: **{model}** en este equipo. ✨")
            return Response(text=("No veo un modelo local activo. Puedes instalar Ollama y ejecutar:\n"
                                  "`./install.sh`\n\n"
                                  "El instalador descargará el modelo y dejará Ollama "
                                  "iniciándose automáticamente.\n\n"
                                  "Archito seguirá funcionando sin él."), status="warn")

        # 3. Continuación contextual: "¿y la temperatura?"
        cont = _CONTINUATION.match(n)
        if cont:
            rest = cont.group(1).strip()
            hit = self._match_intent(rest)
            if hit:
                return hit(self, rest, raw)

        # 4. Matemáticas
        mp = re.match(r"^(?:cuanto es |calcula )?(?:el\s*)?"
                      r"(\d+(?:[.,]\d+)?)\s*%\s*de\s+(.+)$", n)
        if mp:
            try:
                from archito.core.mathcalc import calc, fmt_num
                pct_v = float(mp.group(1).replace(",", "."))
                base = calc(mp.group(2))
                base_n = float(str(base).replace(".", "").replace(",", ".")) \
                    if isinstance(base, str) else float(base)
                return Response(
                    text=(f"El **{fmt_num(pct_v)}%** de **{fmt_num(base_n)}** "
                          f"es **{fmt_num(base_n * pct_v / 100)}** 🧮"),
                    chips=["cuánto es 2^10"],
                )
            except Exception:
                pass

        mm = re.match(r"^(?:cuanto (?:es|son|vale)|calcula(?:r)?|resultado de)\s+(.+)$", n)
        explicit = bool(mm)
        expr = mm.group(1).strip() if mm else (_looks_math(n) and n or None)
        if expr:
            resp = H.h_calculadora(self, expr)
            if resp.status == "ok" or explicit:
                return resp

        # 5. Comandos personales del usuario (plugins FASE 14)
        custom = getattr(self, "custom_commands", {})
        if custom:
            if n in custom:
                return H.a_custom(self, n)
            m = re.match(r"^(?:ejecuta|run|lanza)\s+(?:mi\s+comando\s+)?(.+)$", n)
            if m and m.group(1).strip() in custom:
                return H.a_custom(self, m.group(1).strip())

        # 6. Acciones con argumento (regex primero: más precisas)
        m = re.match(r"^(?:instala[r]?|agrega|anade|ponme|pon|meteme?)\s+"
                     r"(?:el\s+|la\s+)?(?:paquete\s+(?:de\s+)?)?(.+)$", n)
        if m:
            return H.a_instalar(self, m.group(1))

        m = re.match(r"^(?:elimina|desinstala|borra)\s+"
                     r"(?:el\s+|la\s+)?(?:paquete\s+(?:de\s+)?)?(.+)$", n)
        if m:
            target = m.group(1).strip()
            if target in ("la cache", "cache", "el cache", "las caches",
                          "el chat", "la conversacion"):
                return H.h_cache(self, n, raw)
            return H.a_eliminar(self, target)

        m = re.match(r"^(?:busca[rme]*|encuentra)\s+"
                     r"(?:el\s+|la\s+)?(?:paquete[s]?\s+(?:de\s+)?)?"
                     r"(?:llamad[oa]s?\s+)?(.+)$", n)
        if m and m.group(1).strip() not in ("algo", "un paquete"):
            return H.h_buscar_paquete(self, m.group(1))

        m = re.match(r"^(?:informacion|info|detalles)\s+(?:del?\s+)?"
                     r"(?:paquete\s+)?(.+)$", n)
        if m:
            pkg = m.group(1).strip()
            if PKG_SAFE.fullmatch(pkg):
                return H.h_info_paquete(self, pkg)

        m = re.match(r"^(?:explica[me]*|que hace|para que sirve|como funciona)\s+"
                     r"(?:el\s+comando\s+|este\s+comando\s+|el comando de\s+)?(.+)$", n)
        if m:
            objetivo = m.group(1).strip().rstrip("?").strip()
            if len(objetivo.split()) == 1 and objetivo in ("tu", "archito", "esto"):
                pass  # caerá en identidad u otra intención
            else:
                # Solo tratamos como comando si parece comando:
                # binario conocido, flag, ruta o cadena tipo shell.
                from archito.core.explain import COMMANDS
                tokens = objetivo.split()
                looks_like_cmd = (
                    any(t in COMMANDS for t in tokens)
                    or any(t.startswith("-") for t in tokens)
                    or ("/" in objetivo)
                    or ("=" in objetivo)
                    or objetivo.startswith("sudo")
                )
                if looks_like_cmd:
                    return H.h_explicar(self, objetivo)

        # 6. Elige entre A o B / A, B o C
        m = re.match(r"^(?:elige|escoge|decide)\s+(?:entre\s+)?(.+)$", n)
        if m:
            raw_opts = m.group(1)
            opts = re.split(r"\s+o\s+|\s*,\s*|\s+vs\.?\s+", raw_opts)
            return H.h_elige(self, [o.strip() for o in opts if o.strip()])

        # 7. Tabla de intenciones por palabra clave
        hit = self._match_intent(n)
        if hit:
            return hit(self, n, raw)

        # 8. Nada encontrado
        return None

    def _match_intent(self, n):
        """Devuelve el handler de la mejor intención para `n`."""
        best = None
        best_len = 0
        for intent_id, keywords, handler in self.intents:
            for kw in keywords:
                if kw in n and len(kw) > best_len:
                    best = handler
                    best_len = len(kw)
        return best

    def _fallback(self, n, raw: str) -> Response:
        # IA local primero: entiende variaciones libres y devuelve un plan
        # estructurado. El modelo no ve una shell ni puede ejecutar nada.
        if self.memory.local_ai:
            response = self._ai_plan(raw)
            if response is not None:
                return response

        # Sugerencia "¿quisiste decir...?" con similitud difusa
        corpus = []
        for intent_id, keywords, _ in self.intents:
            corpus.extend(keywords[:3])
        words = n.split()
        if len(words) <= 6:
            close = difflib_close(n, corpus)
            if close:
                return Response(
                    text=f"No estoy seguro de haber entendido. 🤔\n¿Quisiste decir «{close}»?",
                    chips=[close],
                )
        if self.memory.local_ai:
            from archito.core.ai import ask
            answer = ask(raw, self.memory.local_ai_model)
            if answer:
                return Response(text=answer, chips=["ayuda", "estado de la ia"])
        return Response(text=pick(*DEFAULT_REPLIES), chips=["ayuda"])

    def _ai_plan(self, raw: str) -> Response | None:
        """Convierte el plan de la IA en respuestas y capacidades verificadas."""
        from archito.core.ai import plan
        ai_plan = plan(raw, system.quick_summary(), self.memory.local_ai_model)
        if ai_plan is None:
            return None

        tool_handlers = {
            "system_summary": H.h_info_sistema,
            "health": H.h_salud,
            "performance": H.h_diagnostico,
            "updates": H.h_actualizaciones,
            "recent_errors": H.h_errores_recientes,
            "failed_services": H.h_servicios_fallidos,
            "network": H.h_internet,
            "packages": H.h_paquetes,
        }
        if ai_plan.kind == "tool":
            handler = tool_handlers.get(ai_plan.tool)
            if handler:
                response = handler(self, norm(ai_plan.tool), raw)
                response.text = f"{ai_plan.answer}\n\n{response.text}"
                return response
            return Response(text=ai_plan.answer, chips=["ayuda"])

        if ai_plan.kind == "command":
            level = classify(ai_plan.command)
            danger = level == DANGEROUS
            action = Action(
                label=ai_plan.action_label or "Revisar y ejecutar acción",
                command=ai_plan.command,
                level=level if danger else CONFIRM,
                mode=MODE_TERMINAL,
                description=(ai_plan.safety_note or
                             "La IA preparó esta acción. Revísala antes de continuar."),
                danger_note=("Este comando puede cambiar o borrar datos. Lee cada parte "
                             "antes de abrir la terminal.") if danger else "",
            )
            return Response(
                text=(f"{ai_plan.answer}\n\n"
                      "> Preparé una acción, pero no ejecuté nada. Revísala y confirma "
                      "solo si estás de acuerdo."),
                actions=[action], chips=["explica " + ai_plan.command, "ayuda"],
                status="warn" if danger else "ok",
            )
        return Response(text=ai_plan.answer, chips=["ayuda", "estado de la ia"])


PKG_SAFE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9.@_+-]{0,60}$")


def difflib_close(query: str, corpus: list[str], threshold: float = 0.62):
    import difflib
    match = difflib.get_close_matches(query, corpus, n=1, cutoff=threshold)
    return match[0] if match else None
