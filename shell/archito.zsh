# Hook de Archito para zsh. Añade a tu ~/.zshrc:  source /ruta/a/archito.zsh
# Apunta cada comando y su código de salida en $XDG_RUNTIME_DIR/archito/recientes
# (carpeta en memoria, privada, que se borra al cerrar sesión).
autoload -Uz add-zsh-hook
__archito_preexec() { __archito_cmd=$1 }
__archito_precmd() {
  local st=$?
  local cmd=$__archito_cmd
  __archito_cmd=
  [[ -n $cmd && -n $XDG_RUNTIME_DIR && $cmd != archito* ]] || return
  local dir=$XDG_RUNTIME_DIR/archito
  [[ -d $dir ]] || mkdir -m 700 -p $dir
  printf '%s\t%s\t%s\n' "$(date +%s)" "$st" "${cmd//$'\n'/ }" >>$dir/recientes
}
add-zsh-hook preexec __archito_preexec
add-zsh-hook precmd __archito_precmd
