# Hook de Archito para fish.
# Apunta cada comando y su código de salida en $XDG_RUNTIME_DIR/archito/recientes
# (carpeta en memoria, privada, que se borra al cerrar sesión). Con eso
# `archito porque` explica el último error y el camino comprueba tus misiones.
function __archito_postexec --on-event fish_postexec
    set -l st $status
    set -q XDG_RUNTIME_DIR; or return
    string match -q 'archito*' -- $argv[1]; and return
    set -l dir $XDG_RUNTIME_DIR/archito
    test -d $dir; or mkdir -m 700 -p $dir
    printf '%s\t%s\t%s\n' (date +%s) $st (string join ' ' -- (string split \n -- $argv[1])) >>$dir/recientes
end
