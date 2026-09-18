# Archito — Roadmap Maestro del Proyecto

## 1. ¿Qué es Archito?

Archito es un asistente inteligente diseñado para sistemas Linux basados en Arch.

Su objetivo no es ser simplemente un chatbot que responde preguntas, ni un asistente de voz tradicional. Archito busca convertirse en una nueva forma de interactuar con el sistema operativo.

El usuario debería poder hablar o escribir de forma natural:

> "Archito, actualiza el sistema."

> "¿Por qué mi computadora está lenta?"

> "Instala Steam."

> "¿Qué está ocupando tanto espacio?"

> "Explícame qué hace este comando."

Archito interpreta la intención, consulta el sistema, analiza la información disponible y ayuda al usuario a realizar la acción.

Su filosofía principal es:

> **No reemplazar la terminal. Ayudar al usuario a entenderla.**

Archito debe ayudar a las personas a perderle el miedo a Arch Linux, aprender cómo funciona su sistema y recuperar el control sobre su propio dispositivo.

---

# 2. Visión del proyecto

Archito será el primer componente de un ecosistema más grande.

La visión a largo plazo sería:

```text
                 ┌─────────────────┐
                 │     ARCHITO     │
                 │ Asistente local │
                 └────────┬────────┘
                          │
          ┌───────────────┼───────────────┐
          │               │               │
          ▼               ▼               ▼
     Arch Linux        Arch Mobile       Plugins
       / PC            / Teléfono       Comunidad
```

La primera versión funcionará en PC.

Más adelante, Archito deberá poder adaptarse a un sistema móvil basado en Linux/Arch.

La idea es que el núcleo de Archito sea independiente de la interfaz.

Por ejemplo:

- En PC se activa con un atajo.
- En móvil se activa con un botón o gesto.
- Ambos utilizan el mismo núcleo.
- Ambos pueden acceder a herramientas del sistema.
- Ambos pueden funcionar localmente.

---

# 3. Principios fundamentales

Estas son las reglas que deberían acompañar el proyecto desde el primer día.

## Privacidad

- [ ] Archito debe funcionar localmente siempre que sea posible.
- [ ] El usuario debe saber cuándo se utiliza Internet.
- [ ] El usuario debe decidir qué información puede salir del dispositivo.
- [ ] No enviar información del sistema a servidores externos sin autorización.

## Transparencia

- [ ] Archito debe mostrar qué está haciendo.
- [ ] Si ejecuta un comando, el usuario debe poder verlo.
- [ ] Debe explicar los comandos cuando sea necesario.
- [ ] Nunca debe inventar información sobre el sistema.

## Libertad

- [ ] El código debe ser abierto.
- [ ] El usuario debe poder modificar Archito.
- [ ] Debe ser posible cambiar el modelo de IA.
- [ ] Debe ser posible crear plugins.
- [ ] El usuario debe mantener el control.

## Seguridad

- [ ] Archito no debe ejecutar comandos peligrosos automáticamente.
- [ ] Las acciones importantes requieren confirmación.
- [ ] Debe explicar las consecuencias de una acción.
- [ ] Debe respetar permisos y privilegios del sistema.
- [ ] `sudo` no debe utilizarse sin necesidad.

## Educación

- [ ] Archito debe enseñar Linux.
- [ ] Debe explicar lo que hace.
- [ ] Debe ayudar a entender errores.
- [ ] Debe ayudar a perder el miedo a la terminal.
- [ ] El usuario debe poder aprender mientras utiliza el asistente.

---

# 4. Organización del tablero

El tablero principal puede tener estas columnas:

## 💡 Ideas

Todo lo que se ocurra.

Aquí no importa si parece imposible.

Ejemplos:

- Integración con Hyprland.
- Plugins.
- Modo desarrollador.
- Control del teléfono.
- Automatización.
- Diagnóstico inteligente.
- Interfaz animada.
- Memoria local.

Nada se pierde.

---

## 📋 Backlog

Ideas que ya fueron aceptadas, pero todavía no tienen prioridad inmediata.

---

## 🎯 Próximamente

Solo las tareas que pertenecen a la siguiente fase.

Aquí debe haber pocas cosas.

---

## 🚧 En progreso

Lo que realmente estás haciendo ahora.

Regla:

> Intentar tener máximo 1–3 tareas aquí.

No empezar veinte cosas al mismo tiempo.

---

## 🧪 Pruebas

Funciones terminadas que todavía necesitan ser probadas.

---

## 🐛 Bugs

Errores encontrados.

Cada error debería tener:

- Qué pasó.
- Qué esperabas que pasara.
- Cómo reproducirlo.
- Información del sistema.
- Estado.

---

## ✅ Hecho

Todo lo terminado.

Nunca borrar estas tarjetas.

Dentro de unos años será interesante ver cómo empezó todo.

---

# FASE 0 — Fundamentos del proyecto

## Objetivo

Crear la identidad, filosofía y estructura antes de empezar a programar.

### Tarjetas

- [ ] Elegir oficialmente el nombre: Archito.
- [ ] Crear una descripción corta del proyecto.
- [ ] Escribir la visión completa.
- [ ] Definir la filosofía.
- [ ] Definir principios de privacidad.
- [ ] Definir principios de seguridad.
- [ ] Definir principios de software libre.
- [ ] Definir qué cosas Archito sí hará.
- [ ] Definir qué cosas Archito nunca hará.
- [ ] Elegir una licencia para el proyecto.
- [ ] Crear el repositorio principal.
- [ ] Crear documentación inicial.

### Documentos

- [ ] `README.md`
- [ ] `VISION.md`
- [ ] `PRINCIPLES.md`
- [ ] `ROADMAP.md`
- [ ] `ARCHITECTURE.md`
- [ ] `CONTRIBUTING.md`
- [ ] `LICENSE`

---

# FASE 1 — Identidad visual de Archito

## Objetivo

Decidir cómo se verá y cómo se sentirá Archito.

No se trata solamente de hacer algo bonito.

La interfaz debe transmitir:

- tecnología;
- cercanía;
- calma;
- modernidad;
- libertad;
- personalidad.

### Personaje

- [ ] Diseñar la apariencia oficial de Archito.
- [ ] Crear diferentes expresiones.
- [ ] Crear estado normal.
- [ ] Crear estado escuchando.
- [ ] Crear estado pensando.
- [ ] Crear estado analizando.
- [ ] Crear estado feliz.
- [ ] Crear estado de error.
- [ ] Crear estado inactivo.

### Animaciones

- [ ] Animación al aparecer.
- [ ] Animación al esconderse.
- [ ] Animación de respiración.
- [ ] Animación de parpadeo.
- [ ] Animación mientras escucha.
- [ ] Animación mientras piensa.
- [ ] Animación al completar una tarea.

### Interfaz

- [ ] Diseñar ventana flotante.
- [ ] Diseñar modo texto.
- [ ] Diseñar modo voz.
- [ ] Diseñar estado de procesamiento.
- [ ] Diseñar confirmación de comandos.
- [ ] Diseñar respuestas.
- [ ] Diseñar errores.
- [ ] Diseñar modo compacto.
- [ ] Diseñar configuración.

---

# FASE 2 — Primer prototipo visual

## Objetivo

Crear Archito sin IA.

Solo la experiencia visual.

El objetivo es poder presionar un atajo y sentir:

> "Aquí está Archito."

### Primera versión

- [ ] Elegir tecnología para la interfaz.
- [ ] Crear una ventana flotante.
- [ ] Posicionarla en una esquina.
- [ ] Configurar transparencia.
- [ ] Añadir bordes redondeados.
- [ ] Añadir animación de entrada.
- [ ] Añadir animación de salida.
- [ ] Añadir caja de texto.
- [ ] Permitir escribir.
- [ ] Permitir cerrar la ventana.
- [ ] Crear un atajo de teclado.

### Hito de la fase

```text
Presiono:

SUPER + ESPACIO

↓

Archito aparece

↓

La caja de texto recibe el foco

↓

Puedo escribir

↓

Archito se oculta
```

Todavía no responde nada inteligente.

Pero la experiencia visual ya existe.

---

# FASE 3 — Sistema de entrada

## Objetivo

Permitir dos formas de comunicarse con Archito:

```text
          Usuario
             │
     ┌───────┴────────┐
     │                │
     ▼                ▼
  Teclado         Micrófono
     │                │
     │                ▼
     │          Transcripción
     │                │
     └────────┬───────┘
              ▼
           Texto
```

## Entrada por teclado

- [ ] Capturar texto.
- [ ] Enviar mensaje al núcleo.
- [ ] Mostrar historial.
- [ ] Permitir cancelar una solicitud.
- [ ] Crear atajos rápidos.

## Entrada por voz

La idea inicial sería utilizar Handy como parte del flujo de captura/transcripción que quieres experimentar.

El objetivo es que esta parte sea intercambiable.

```text
Micrófono
    ↓
Captura de audio
    ↓
Motor de transcripción
    ↓
Texto
    ↓
Archito Core
```

### Tareas

- [ ] Detectar micrófono.
- [ ] Iniciar captura.
- [ ] Detener captura.
- [ ] Guardar audio temporalmente.
- [ ] Enviar audio al motor de transcripción.
- [ ] Recibir texto.
- [ ] Mostrar texto reconocido.
- [ ] Permitir corregir el texto.
- [ ] Eliminar audio temporal después del procesamiento.

### Push to Talk

- [ ] Definir combinación de teclas.
- [ ] Activar micrófono.
- [ ] Mostrar animación "Escuchando".
- [ ] Detectar finalización.
- [ ] Detener micrófono.
- [ ] Enviar audio a transcripción.
- [ ] Mostrar texto resultante.

---

# FASE 4 — Archito Core

## Objetivo

Crear el verdadero núcleo del proyecto.

La interfaz no debería saber cómo funciona todo.

El núcleo recibe una solicitud y decide qué hacer.

```text
Interfaz
    ↓
Archito Core
    ↓
¿Entender?
¿Consultar?
¿Ejecutar?
¿Preguntar?
    ↓
Respuesta
```

### Tareas

- [ ] Crear estructura principal del proyecto.
- [ ] Crear sistema de mensajes.
- [ ] Crear gestor de solicitudes.
- [ ] Crear sistema de contexto.
- [ ] Crear sistema de errores.
- [ ] Crear registro de actividades.
- [ ] Crear sistema de permisos.
- [ ] Separar interfaz y núcleo.

---

# FASE 5 — Primeras acciones sin IA

## Objetivo

Antes de añadir una IA, Archito debe ser capaz de hacer tareas básicas.

Esto permitirá construir y probar el sistema de acciones.

### Acciones iniciales

- [ ] Mostrar información del sistema.
- [ ] Mostrar versión del kernel.
- [ ] Mostrar distribución.
- [ ] Mostrar paquetes instalados.
- [ ] Buscar paquetes.
- [ ] Actualizar repositorios.
- [ ] Preparar actualización del sistema.
- [ ] Mostrar espacio disponible.
- [ ] Mostrar memoria RAM.
- [ ] Mostrar procesos.
- [ ] Mostrar servicios.
- [ ] Mostrar información de red.

### Ejemplo

Usuario:

> "Actualiza el sistema."

Archito identifica la intención:

```text
ACTUALIZAR_SISTEMA
```

Después genera una acción.

Antes de ejecutarla:

```text
Voy a actualizar el sistema.

Comando:

sudo pacman -Syu

Esto actualizará los paquetes instalados.

¿Deseas continuar?
```

- [ ] Confirmar.
- [ ] Ejecutar.
- [ ] Mostrar progreso.
- [ ] Mostrar resultado.

---

# FASE 6 — Conexión con el sistema

## Objetivo

Archito empieza a conocer el dispositivo.

No solo responder preguntas generales.

Debe poder observar su propio sistema.

## Información básica

- [ ] CPU.
- [ ] RAM.
- [ ] Disco.
- [ ] Kernel.
- [ ] Temperaturas.
- [ ] Batería.
- [ ] GPU.
- [ ] Procesos.

## Sistema

- [ ] Paquetes.
- [ ] Servicios.
- [ ] Logs.
- [ ] Usuarios.
- [ ] Discos.
- [ ] Montajes.
- [ ] Red.

## Herramientas iniciales

Archito puede trabajar con herramientas como:

- [ ] `pacman`
- [ ] `systemctl`
- [ ] `journalctl`
- [ ] `nmcli`
- [ ] `ip`
- [ ] `lsblk`
- [ ] `df`
- [ ] `free`
- [ ] `ps`
- [ ] `uname`

Cada integración debe convertirse en un módulo independiente.

---

# FASE 7 — El intérprete inteligente

## Objetivo

Ahora sí entra la IA.

Pero la IA no debe tener acceso libre al sistema.

La arquitectura debería ser:

```text
Usuario
   ↓
Modelo de IA
   ↓
Intención estructurada
   ↓
Validador
   ↓
Herramientas permitidas
   ↓
Sistema
   ↓
Resultado
   ↓
Modelo / Archito
   ↓
Respuesta
```

La IA debería entender:

> "Mi computador está lento."

Y convertirlo en algo parecido a:

```text
INTENCIÓN:
DIAGNOSTICAR_RENDIMIENTO

NECESITA:
- CPU
- RAM
- Procesos
- Temperatura
- Disco
```

Después Archito obtiene la información real.

Y solo entonces genera una respuesta.

### Tareas

- [ ] Elegir primer modelo local.
- [ ] Crear interfaz para modelos intercambiables.
- [ ] Definir formato de acciones.
- [ ] Limitar herramientas disponibles.
- [ ] Validar respuestas.
- [ ] Evitar ejecución directa de comandos generados por IA.
- [ ] Crear sistema de confirmación.

---

# FASE 8 — Archito Offline

## Objetivo

Cumplir una de las ideas más importantes del proyecto:

> Archito debe poder funcionar sin Internet.

El usuario debería poder preguntar:

- "¿Cuánto espacio tengo?"
- "¿Qué proceso usa más RAM?"
- "¿Qué paquetes tengo instalados?"
- "¿Por qué falló este servicio?"
- "¿Qué hace este comando?"

Sin depender de una empresa externa.

### Tareas

- [ ] Ejecutar IA local.
- [ ] Crear base de conocimiento local.
- [ ] Integrar páginas `man`.
- [ ] Integrar documentación instalada.
- [ ] Crear sistema de búsqueda local.
- [ ] Crear recuperación de contexto.
- [ ] Optimizar uso de RAM.
- [ ] Optimizar tiempo de respuesta.

---

# FASE 9 — Archito Profesor

## Objetivo

Esta sería una de las características más importantes.

Archito no debe esconder Linux.

Debe enseñarlo.

### Ejemplo

Usuario:

> "Instala Firefox."

Archito:

```text
Voy a utilizar:

sudo pacman -S firefox
```

Botones:

- Ejecutar.
- Cancelar.
- Explicar comando.

Si el usuario pulsa explicar:

```text
sudo
↓
Ejecuta el comando con privilegios administrativos.

pacman
↓
Gestor de paquetes de Arch Linux.

-S
↓
Instala un paquete.

firefox
↓
Nombre del paquete.
```

### Tareas

- [ ] Crear botón "Explicar".
- [ ] Explicar comandos.
- [ ] Explicar errores.
- [ ] Explicar servicios.
- [ ] Explicar procesos.
- [ ] Crear modo principiante.
- [ ] Crear modo avanzado.

---

# FASE 10 — Diagnóstico inteligente

## Objetivo

Que Archito no responda con información genérica.

Debe investigar.

Usuario:

> "¿Por qué está lento mi PC?"

Archito:

```text
Analizando CPU...
Analizando RAM...
Analizando procesos...
Analizando temperatura...
Analizando disco...
```

Después:

```text
Encontré tres posibles causas:

1. Firefox está utilizando...
2. La memoria RAM está...
3. La temperatura del procesador está...
```

### Diagnósticos

- [ ] Rendimiento.
- [ ] RAM.
- [ ] CPU.
- [ ] Disco.
- [ ] Temperatura.
- [ ] Wi-Fi.
- [ ] Servicios fallando.
- [ ] Problemas de paquetes.
- [ ] Errores de actualización.

---

# FASE 11 — Seguridad y permisos

## Objetivo

Evitar que Archito se convierta en una IA con acceso ilimitado a la terminal.

### Clasificación de acciones

## Nivel 1 — Seguro

Ejemplos:

```text
ls
df
free
uname
```

Puede ejecutarse directamente.

- [ ] Crear lista de comandos seguros.

## Nivel 2 — Requiere confirmación

Ejemplos:

```text
pacman -S paquete
systemctl restart servicio
```

Archito explica y pregunta.

- [ ] Crear sistema de confirmación.

## Nivel 3 — Peligroso

Ejemplos:

```text
rm
mkfs
dd
pacman -R
```

Archito debe explicar claramente las consecuencias.

- [ ] Crear sistema de advertencias.
- [ ] Detectar comandos destructivos.
- [ ] Mostrar impacto.
- [ ] Requerir confirmación explícita.

---

# FASE 12 — Memoria local

## Objetivo

Archito puede recordar preferencias del usuario.

No debería almacenar cosas innecesarias.

### Ejemplos

- [ ] Escritorio utilizado.
- [ ] Terminal preferida.
- [ ] Gestor de paquetes auxiliar.
- [ ] Herramientas favoritas.
- [ ] Nivel de experiencia.
- [ ] Preferencias de explicación.

Ejemplo:

> "Prefiero que me expliques los comandos."

Archito recuerda esa preferencia.

Todo debería permanecer local.

---

# FASE 13 — Personalidad

## Objetivo

Archito no debe ser frío.

Pero tampoco debe convertirse en un personaje molesto.

Características:

- [ ] Paciente.
- [ ] Curioso.
- [ ] Técnico cuando sea necesario.
- [ ] Fácil de entender.
- [ ] Nunca arrogante.
- [ ] Nunca oculta información.
- [ ] Reconoce cuando no sabe algo.

### Estados

- [ ] Normal.
- [ ] Feliz.
- [ ] Curioso.
- [ ] Pensando.
- [ ] Analizando.
- [ ] Advertencia.
- [ ] Error.
- [ ] Celebración.

---

# FASE 14 — Sistema de plugins

## Objetivo

Permitir que la comunidad amplíe Archito.

```text
Archito
   │
   ├── Core
   │
   ├── Pacman Plugin
   │
   ├── Docker Plugin
   │
   ├── Hyprland Plugin
   │
   ├── Git Plugin
   │
   ├── Python Plugin
   │
   └── Plugins de la comunidad
```

### Primera etapa

- [ ] Diseñar arquitectura de plugins.
- [ ] Definir permisos.
- [ ] Crear API.
- [ ] Crear documentación.
- [ ] Crear plugin de ejemplo.

---

# FASE 15 — Integración con Hyprland

## Objetivo

Convertir Archito en una experiencia natural dentro del escritorio.

### Funciones

- [ ] Atajo global.
- [ ] Overlay.
- [ ] Blur.
- [ ] Animaciones.
- [ ] Integración visual.
- [ ] Detección del entorno.
- [ ] Acciones relacionadas con ventanas.
- [ ] Acciones relacionadas con espacios de trabajo.

### Hito

```text
SUPER + ESPACIO
        ↓
Archito aparece
        ↓
Usuario habla o escribe
        ↓
Archito responde
        ↓
SUPER + ESPACIO
        ↓
Archito desaparece
```

---

# FASE 16 — Beta privada

## Objetivo

Probar Archito con pocas personas.

No lanzar públicamente todavía.

### Tareas

- [ ] Crear instalador.
- [ ] Crear guía.
- [ ] Probar en diferentes configuraciones.
- [ ] Recibir errores.
- [ ] Medir consumo de RAM.
- [ ] Medir velocidad.
- [ ] Corregir problemas.
- [ ] Mejorar interfaz.

---

# FASE 17 — Primera versión pública

## Archito v0.1

La primera versión NO necesita hacer todo.

Objetivo mínimo:

- [ ] Interfaz.
- [ ] Texto.
- [ ] Voz.
- [ ] Transcripción.
- [ ] Modelo local.
- [ ] Información básica del sistema.
- [ ] Ejecución controlada.
- [ ] Explicación de comandos.
- [ ] Confirmación de acciones.

---

# FASE 18 — Camino hacia Arch Mobile

Solo después de que Archito tenga una base sólida.

La arquitectura debería permitir:

```text
              Archito Core
                    │
        ┌───────────┴───────────┐
        │                       │
        ▼                       ▼
     Arch PC                 Arch Mobile
        │                       │
   Hyprland UI             Mobile UI
        │                       │
   Teclado / Voz          Táctil / Voz
```

### Futuras funciones móviles

- [ ] Activación por botón.
- [ ] Activación por gesto.
- [ ] Modo voz.
- [ ] Control del sistema.
- [ ] Estado de batería.
- [ ] Temperatura.
- [ ] Wi-Fi.
- [ ] Bluetooth.
- [ ] Aplicaciones.
- [ ] Procesos.
- [ ] Diagnóstico del dispositivo.

---

# 5. Primera misión real

Antes de pensar en IA, modelos, plugins o Arch Mobile:

## ARCHITO MILESTONE 0.0.1

El objetivo es únicamente conseguir esto:

- [ ] Presionar un atajo.
- [ ] Archito aparece desde una esquina.
- [ ] Ejecutar una animación.
- [ ] Mostrar una caja de texto.
- [ ] Escribir una pregunta.
- [ ] Presionar Enter.
- [ ] El backend recibe el texto.
- [ ] El backend devuelve una respuesta simple.
- [ ] La interfaz muestra esa respuesta.
- [ ] Archito puede desaparecer.

Ejemplo:

```text
SUPER + ESPACIO

↓

👋 Hola, soy Archito.

¿En qué puedo ayudarte?

> Hola

↓

Hola. Todavía estoy aprendiendo,
pero ya puedo escucharte.
```

Si eso funciona, Archito oficialmente habrá nacido.

---

# 6. Regla principal del desarrollo

Nunca intentar construir todo al mismo tiempo.

La secuencia será:

```text
1. Que aparezca
        ↓
2. Que escuche
        ↓
3. Que lea
        ↓
4. Que entienda
        ↓
5. Que consulte
        ↓
6. Que explique
        ↓
7. Que ejecute
        ↓
8. Que aprenda preferencias
        ↓
9. Que pueda ampliarse
        ↓
10. Que llegue a Arch Mobile
```

---

# 7. Definición final del proyecto

Archito no será simplemente:

> "Una IA que ejecuta comandos."

Será:

> **Un asistente local, abierto y transparente que ayuda al usuario a conversar con su sistema Linux, entenderlo, administrarlo y mantener el control sobre su propio dispositivo.**

Y, a largo plazo:

> **El núcleo inteligente de un ecosistema Linux que pueda vivir tanto en computadoras como en dispositivos móviles.**