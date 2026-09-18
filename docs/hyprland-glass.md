# Archito Glass — integración con Hyprland

Archito no toma una captura del wallpaper ni lo redibuja. Sus ventanas son
*layer surfaces* transparentes y Hyprland compone detrás de ellas el contenido
real del escritorio (wallpaper, ventanas y barras) con blur. Por eso el efecto
siempre pertenece a tu rice y cambia al cambiar de wallpaper.

## 1. Carga las reglas de Archito

Incluye una sola vez esto en tu configuración de Hyprland:

```ini
source = ~/Escritorio/Archito/hyprland/archito.conf
```

Ese archivo no toca los valores globales de tu configuración; solo registra
blur para los namespaces `archito` y `archito-dock`.

## 2. Asegúrate de que tu rice tenga blur global

Normalmente tus dotfiles ya lo definen. Si no, añade o adapta este bloque a tu
configuración principal. Es una base suave: suficiente para leer texto, sin
convertir el panel en una lámina oscura.

```ini
decoration {
    blur {
        enabled = true
        size = 8
        passes = 2
        ignore_opacity = true
        noise = 0.01
        contrast = 0.95
        brightness = 1.0
        vibrancy = 0.12
    }
}
```

No copies el bloque si ya tienes uno: fusiona únicamente los valores que te
interesen. Subir `size` o `passes` intensifica el desenfoque, pero también usa
más GPU.

## 3. Colores de tus dotfiles

Di **«tema auto»** para que Archito lea la paleta de pywal o matugen. Así los
colores del texto, los acentos y el vidrio acompañan al wallpaper mientras
Hyprland se ocupa de desenfocar lo que hay detrás.

## Si no aparece el blur

1. Ejecuta `hyprctl reload` después de editar la configuración.
2. Comprueba los namespaces con `hyprctl layers`; deben aparecer `archito` y
   `archito-dock`.
3. Verifica que no exista otra regla posterior que desactive blur para esos
   namespaces.

Las versiones recientes de Hyprland documentan `blur` e `ignore_alpha` como
efectos específicos de las layer surfaces. La sintaxis heredada
`layerrule = blur, archito` se conserva aquí porque es compatible con los
dotfiles Hyprland habituales.
