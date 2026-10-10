# Hook de Archito para bash. Añade a tu ~/.bashrc:  source /ruta/a/archito.bash
# Apunta cada comando y su código de salida en $XDG_RUNTIME_DIR/archito/recientes
# (carpeta en memoria, privada, que se borra al cerrar sesión).
__archito_prompt() {
  local st=$?
  [ -n "$XDG_RUNTIME_DIR" ] || return
  local entry cmd
  entry=$(HISTTIMEFORMAT='' history 1)
  # Enter sin escribir nada repite la misma entrada del historial: se ignora.
  [ "$entry" = "$__archito_last" ] && return
  __archito_last=$entry
  cmd=$(printf '%s' "$entry" | sed 's/^ *[0-9]* *//' | tr '\n' ' ')
  case $cmd in ''|archito*) return ;; esac
  local dir=$XDG_RUNTIME_DIR/archito
  [ -d "$dir" ] || mkdir -m 700 -p "$dir"
  printf '%s\t%s\t%s\n' "$(date +%s)" "$st" "$cmd" >>"$dir/recientes"
}
PROMPT_COMMAND="__archito_prompt${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
