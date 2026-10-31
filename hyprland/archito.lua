-- ──────────────────────────────────────────────
-- Archito para Hyprland con configuración en Lua (hyprland.lua).
-- Si tu Hyprland usa hyprland.conf, mira archito.conf en esta carpeta.
--
-- Cárgalo desde tu hyprland.lua con:
--   dofile(os.getenv("HOME") .. "/ruta/a/Archito/hyprland/archito.lua")
-- o copia estas líneas en un archivo que ya cargues (custom/keybinds.lua).
-- ──────────────────────────────────────────────

-- SUPER + Espacio abre o recoge la isla. Si usas la AppImage, pon su ruta.
hl.bind("SUPER + space", hl.dsp.exec_cmd("archito-panel"), { description = "Archito: abrir o recoger la isla" })

-- La isla es una superficie layer-shell con el namespace "archito".
-- Blur real de Hyprland detrás de sus partes translúcidas; usa tu
-- configuración de decoration.blur, no la cambia.
hl.layer_rule({ match = { namespace = "^archito$" }, blur = true, ignore_alpha = 0.01 })
