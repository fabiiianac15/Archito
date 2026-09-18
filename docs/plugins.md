# Plugins de Archito — comandos personales

> Semilla del sistema de plugins (FASE 14 del roadmap).
> Sin APIs raras: un archivo JSON que ya casi todos saben escribir.

## Cómo funciona

Crea `~/.config/archito/commands.json` (puedes partir de
[`examples/commands.ejemplo.json`](../examples/commands.ejemplo.json)):

```json
{
  "backup fotos": {
    "command": "rsync -a ~/Fotos /run/media/backup/",
    "description": "Copia tus fotos al disco externo.",
    "dangerous": false
  }
}
```

Dile a Archito **«recarga los comandos»** y desde ese momento:

- «backup fotos» → tarjeta de confirmación con tu comando.
- «mis comandos» → lista de todo lo que definiste.

## Reglas de seguridad (no negociables)

| Regla | Motivo |
|---|---|
| Siempre terminal externa | Tu comando puede pedir contraseña o ser interactivo. |
| Siempre confirmación    | Archito nunca ejecuta nada sin tu visto bueno. |
| `dangerous: true`       | Tarjeta roja + advertencia extra antes de ejecutar. |
| Validación estricta     | JSON inválido se ignora; Archito no explota jamás por tu config. |

## Campos disponibles

| Campo | Obligatorio | Descripción |
|---|---|---|
| `command`      | ✅ | El comando a ejecutar. |
| `description`  | —  | Texto educativo en la tarjeta. |
| `dangerous`    | —  | `true` = confirmación roja. |
| `confirm_text` | —  | Pregunta personalizada de confirmación. |

## Por qué así

Este mecanismo es deliberadamente simple porque es la base del futuro
ecosistema: en Arch Mobile el mismo JSON funcionará igual, y más adelante
los plugins de la comunidad serán paquetes que *instalan* entradas aquí
(o su equivalente estructurado). Tus comandos son tuyos: viven en tu
carpeta local, sin nube, sincronizables con dotfiles si quieres.
