# Archito

Asistente local para Arch Linux con interfaz flotante, diagnóstico del sistema, acciones seguras y apoyo opcional de IA local.

> No reemplaza la terminal: ayuda a entenderla, a usarla con confianza y a tomar decisiones informadas.

Archito es un proyecto pensado para la comunidad de Arch y Linux avanzado: consulta el estado real del sistema, responde en lenguaje natural, ejecuta solo acciones autorizadas y explica cada operación con contexto.

## ¿Qué es Archito?

Archito combina varias ideas en una sola experiencia:

- asistente conversacional local
- panel visual sobre Wayland
- diagnóstico del sistema en lenguaje claro
- seguridad por diseño antes de ejecutar cambios
- educación técnica a través de explicaciones
- apoyo opcional a modelos locales con Ollama

La idea no es automatizar el sistema sin control del usuario, sino hacer que el usuario comprenda qué está pasando y pueda reaccionar con criterio.

## Filosofía del proyecto

1. Privacidad: todo se ejecuta localmente cuando es posible.
2. Transparencia: no se ocultan comandos ni decisiones.
3. Seguridad: las acciones destructivas o sensibles requieren confirmación.
4. Educación: cada respuesta intenta enseñar, no solo responder.
5. Libertad: el proyecto quiere ser abierto, adaptable y personalizable.

## Estado actual

Versión actual: 0.5.0

Archito se encuentra en una etapa activa de prototipo funcional con una interfaz muy pulida visualmente, pero todavía orientada a la investigación, la iteración y la adaptación a distintos entornos Arch/Wayland.

Características destacadas de esta etapa:

- panel flotante con estilo HUD glassmorphism
- dock lateral plegable/expandible
- interfaz conversacional con sugerencias, historial y mensajes animados
- detección y diagnóstico local del hardware y sistema
- seguridad y clasificación de acciones
- memoria local del usuario
- soporte opcional para IA local con Ollama
- temas visuales dinámicos, incluyendo variantes para Hyprland

## Capacidades principales

### Sistema y hardware

- información del sistema
- kernel, distro, hardware, CPU, RAM, disco, batería, GPU
- temperatura, uptime, boot time, sesiones activas
- diagnóstico de rendimiento y servicios fallidos
- revisión de salud general del equipo

### Arch y mantenimiento

- paquetes instalados, paquetes grandes, huérfanos
- actualizaciones pendientes
- archivos pacnew, vulnerabilidades, historial de instalación
- solución de problemas comunes de arranque y sistema
- explicación de comandos y mantenimiento de la máquina

### Escritorio y entorno

- capturas de pantalla
- bloqueo de pantalla
- brillo y volumen
- integración con Hyprland
- recarga de configuración del compositor

### Interacción y experiencia

- conversación natural
- sugerencias rápidas
- historial de la sesión
- memoria persistente del usuario
- temas visuales
- modo táctil para pantallas pequeñas
- experiencias easter eggs y personalidad del asistente

## Arquitectura del proyecto

La arquitectura está pensada como una separación clara entre interfaz, lógica de intención y acceso a recursos del sistema.

```text
Usuario
  │
  ▼
UI (GTK4 + layer-shell)
  │
  ▼
Brain / Intents (backend.py)
  │
  ├─> handlers.py: respuestas y decisiones por intención
  ├─> system.py: sondas de solo lectura del sistema
  ├─> actions.py: clasificación y ejecución segura
  ├─> memory.py: almacenamiento local de preferencias
  ├─> custom.py: comandos personalizados JSON
  ├─> ai.py: integración local con Ollama
  └─> explain.py: conocimiento para explicar comandos
  │
  ▼
Sistema operativo / Arch / Wayland / shell
```

### Capas principales

#### 1. Entrada e interfaz

- `src/archito/app.py`: aplicación GTK principal
- `src/archito/ui/window.py`: ventana de chat y panel principal
- `src/archito/ui/dock.py`: dock lateral plegable
- `src/archito/ui/style.py`: estilos visuales y temas

Estas partes controlan la presencia visual, la animación, el comportamiento de la ventana y la integración con layer-shell de Hyprland.

#### 2. Núcleo de decisión

- `src/archito/core/backend.py`: orquestación del procesamiento del texto
- `src/archito/core/handlers.py`: handlers por intención y tipo de petición
- `src/archito/core/textutils.py`: normalización y afinación de respuesta

Es el cerebro: toma el texto del usuario, identifica la intención, decide qué información consultar y prepara la respuesta.

#### 3. Sistema y seguridad

- `src/archito/core/system.py`: sondeo local del sistema, sin modificar nada
- `src/archito/core/actions.py`: clasificación de permisos, ejecución segura y control de terminal
- `src/archito/core/memory.py`: almacenamiento persistente en `~/.config/archito/`

Aquí se aplica la regla clave del proyecto: lectura segura por defecto, modificacion con confirmación y acciones peligrosas bloqueadas o explícitamente advertidas.

#### 4. Extensión y modelo local

- `src/archito/core/ai.py`: integración con Ollama local
- `src/archito/core/custom.py`: comandos definidos por el usuario en JSON

Esto permite ampliar el comportamiento del asistente sin depender de servicios externos ni de una nube.

## Modelo de seguridad

Archito no ejecuta comandos arbitrariamente. La lógica de seguridad incluye:

- comandos de solo lectura: se ejecutan en línea y se muestran dentro del chat
- comandos que modifican el sistema: requieren confirmación explícita
- acciones peligrosas: están bloqueadas o marcadas con advertencias claras
- entorno limpio para subprocesos: se eliminan variables problemáticas del entorno
- no se usa Internet para la lógica principal del sistema

Esto convierte a Archito en una herramienta más segura y transparente que un asistente que ejecuta "sin explicar".

## Instalación

### Requisitos de sistema

En Arch Linux:

```bash
sudo pacman -S gtk4 python-gobject gtk4-layer-shell
```

Opcionalmente, según tus usos:

```bash
sudo pacman -S pacman-contrib grim hyprlock brightnessctl
```

### Instalación automática

```bash
./install.sh
```

Este script instala dependencias, prepara el lanzador y puede configurar Ollama para IA local opcional.

### Instalación manual

```bash
pip install --user .
```

O desde el repositorio:

```bash
PYTHONPATH=src python -m archito
```

## Uso rápido

Lanza el asistente:

```bash
archito
```

Ejemplos de consultas:

- `info del sistema`
- `temperatura`
- `ram`
- `actualiza el sistema`
- `explica sudo pacman -Syu`
- `tema catppuccin`
- `mi ip`
- `estado de salud`
- `por qué va lento`

## Integración con Hyprland

El proyecto está pensado para funcionar muy bien sobre Hyprland.

```ini
bind = SUPER, space, exec, archito

layerrule = blur, archito
layerrule = ignorealpha 0.01, archito
layerrule = blur, archito-dock
layerrule = ignorealpha 0.01, archito-dock
```

También se incluye un archivo de ejemplo en `hyprland/archito.conf` y una guía de estilo/glass en `docs/hyprland-glass.md`.

## Estructura del repositorio

```text
Archito/
├── README.md
├── LICENSE
├── setup.py
├── install.sh
├── uninstall.sh
├── run.sh
├── Archito — Roadmap Maestro del Proyecto.md
├── src/
│   └── archito/
│       ├── __init__.py
│       ├── __main__.py
│       ├── app.py
│       ├── core/
│       │   ├── actions.py
│       │   ├── activity.py
│       │   ├── ai.py
│       │   ├── backend.py
│       │   ├── custom.py
│       │   ├── explain.py
│       │   ├── handlers.py
│       │   ├── health.py
│       │   ├── mathcalc.py
│       │   ├── memory.py
│       │   ├── system.py
│       │   ├── textutils.py
│       │   └── themes.py
│       └── ui/
│           ├── assets.py
│           ├── dock.py
│           ├── style.py
│           └── window.py
├── docs/
│   ├── hyprland-glass.md
│   └── plugins.md
├── examples/
│   └── commands.ejemplo.json
├── hyprland/
│   └── archito.conf
├── assets/
│   └── img/
├── img/
├── legacy/
│   ├── archito_window.py
│   ├── backend.py
│   └── main.py
└── scripts/
    └── archito.sh
```

## IA local opcional

Archito puede funcionar sin IA, pero si tienes Ollama disponible, admite un modelo local para responder preguntas abiertas y ayudar a analizar el contexto del sistema sin exponerlo fuera del equipo.

La integración está pensada para ser segura:

- solo acepta el servidor local
- no ejecuta automáticamente acciones sin confirmación
- no reemplaza el flujo de permisos ni de seguridad del sistema

## Roadmap y dirección

El proyecto ya tiene una hoja de ruta propia en `Archito — Roadmap Maestro del Proyecto.md` con fases, principios y visión a largo plazo. La visión central es construir un asistente local y educativo para Linux, con una base técnica sólida y una experiencia visual útil.

### Fases visibles del proyecto

- prototipo visual de ventana
- análisis de sistema
- interpretación de intenciones
- acciones seguras con confirmación
- memoria local y personalización
- temporización visual, temas y refinamiento de UX
- soporte de plugins y extensiones
- IA local opcional
- visión hacia Arch Mobile / ecosistema más amplio

## Contribución

Las contribuciones son bienvenidas.

Si quieres colaborar:

1. haz un fork del proyecto
2. crea una rama para tu cambio
3. prueba tus cambios localmente
4. abre un pull request con descripción clara

Se agradece especialmente ayuda en:

- mejoras de UX
- integración con más herramientas del sistema
- diagnósticos más inteligentes
- documentación
- soporte para más escritorios y entornos
- prueba en distintos equipos Arch

## Licencia

Este proyecto está licenciado bajo la licencia MIT.

Consulta el archivo [LICENSE](LICENSE) para más detalles.

## Resumen corto

Archito es un asistente local para Arch Linux que ayuda a entender el sistema, responder consultas reales del equipo y ejecutar cambios con seguridad, claridad y educación.

Su objetivo principal no es reemplazar la terminal, sino hacer que aprender a usar Linux sea más accesible, seguro y transparente.
