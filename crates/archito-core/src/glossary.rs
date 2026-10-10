//! Glosario: las palabras de Arch que nadie explica, en cristiano.

pub struct Term {
    pub names: &'static [&'static str],
    pub title: &'static str,
    pub text: &'static str,
    /// Comando para verlo en el sistema propio, si aplica.
    pub try_it: Option<&'static str>,
}

pub const TERMS: &[Term] = &[
    Term {
        names: &["kernel", "nucleo"],
        title: "Kernel",
        text: "Es el programa central de Linux: reparte la CPU y la memoria entre los demás programas y habla con el hardware. Arch te deja elegir cuál usar (linux, linux-lts, linux-zen).",
        try_it: Some("uname -r"),
    },
    Term {
        names: &["paquete", "paquetes"],
        title: "Paquete",
        text: "Un programa (o una librería) empaquetado con la lista de lo que necesita para funcionar. Pacman los descarga, los instala y recuerda qué archivos puso cada uno, por eso puede quitarlos limpio.",
        try_it: Some("pacman -Qi bash"),
    },
    Term {
        names: &["repositorio", "repositorios", "repo", "repos"],
        title: "Repositorio",
        text: "Un almacén de paquetes mantenido por Arch. Los principales son core (la base del sistema) y extra (casi todo lo demás). Instalar desde ahí es más seguro que bajar instaladores de páginas web.",
        try_it: Some("pacman -Sl core"),
    },
    Term {
        names: &["aur", "arch user repository"],
        title: "AUR",
        text: "El repositorio de la comunidad. No guarda programas listos sino recetas (PKGBUILD) para construirlos en tu equipo. Tiene casi todo lo que falta en los oficiales, pero cualquiera puede subir una receta: conviene leerla antes de instalar. Se usa con ayudantes como yay o paru.",
        try_it: None,
    },
    Term {
        names: &["pacman"],
        title: "Pacman",
        text: "El gestor de paquetes de Arch. Tres letras lo resumen: -S trae cosas de los repositorios, -Q consulta lo que tienes instalado y -R quita.",
        try_it: Some("pacman -Qe"),
    },
    Term {
        names: &["dependencia", "dependencias"],
        title: "Dependencia",
        text: "Un paquete que otro necesita para funcionar. Pacman las instala solo. Cuando quitas el programa que las pidió, se quedan: a esas se les llama huérfanas.",
        try_it: Some("pacman -Qtdq"),
    },
    Term {
        names: &["huerfano", "huerfanos", "huerfana", "huerfanas"],
        title: "Paquete huérfano",
        text: "Una dependencia que se instaló para otro programa que ya no está. No hace daño, solo ocupa espacio.",
        try_it: Some("pacman -Qtdq"),
    },
    Term {
        names: &["systemd"],
        title: "systemd",
        text: "El primer programa que arranca el kernel y el que enciende todo lo demás: red, sonido, inicio de sesión. Cada cosa que maneja se llama unidad, y se controla con systemctl.",
        try_it: Some("systemctl list-units --type=service --state=running"),
    },
    Term {
        names: &["servicio", "servicios", "demonio", "daemon", "unidad"],
        title: "Servicio",
        text: "Un programa que corre de fondo sin ventana, como la red o el Bluetooth. Con systemctl lo arrancas (start), lo paras (stop) o lo dejas activado para cada inicio (enable).",
        try_it: Some("systemctl --failed"),
    },
    Term {
        names: &["journal", "logs", "log", "registro"],
        title: "Journal (el registro)",
        text: "El diario del sistema: cada servicio escribe ahí lo que hace y lo que le falla. Se lee con journalctl.",
        try_it: Some("journalctl -p err -b -n 15"),
    },
    Term {
        names: &["sudo", "root", "administrador", "superusuario"],
        title: "root y sudo",
        text: "root es el usuario administrador: puede cambiar o borrar cualquier archivo. sudo ejecuta un solo comando como root y te pide tu contraseña. La regla sana: sin sudo por defecto, con sudo solo cuando el sistema lo pida.",
        try_it: Some("id"),
    },
    Term {
        names: &["terminal", "consola", "emulador de terminal"],
        title: "Terminal",
        text: "La ventana donde escribes comandos. La terminal solo muestra texto; quien entiende lo que escribes es la shell que corre dentro.",
        try_it: None,
    },
    Term {
        names: &["shell", "bash", "zsh", "fish"],
        title: "Shell",
        text: "El intérprete que lee lo que escribes y lanza los programas. bash es la clásica, zsh y fish añaden autocompletado y sugerencias. Puedes cambiarla cuando quieras.",
        try_it: Some("whoami"),
    },
    Term {
        names: &["wayland", "xorg", "x11"],
        title: "Wayland y Xorg",
        text: "Son las dos formas que tiene Linux de dibujar ventanas. Xorg es la antigua; Wayland es la actual, más segura y fluida. Hyprland, Sway y las versiones nuevas de GNOME y KDE usan Wayland.",
        try_it: None,
    },
    Term {
        names: &["hyprland", "compositor", "gestor de ventanas", "window manager"],
        title: "Compositor",
        text: "El programa que coloca y dibuja las ventanas. Hyprland es un compositor de Wayland de tipo mosaico: acomoda las ventanas solo y se maneja casi todo con el teclado. Su configuración está en ~/.config/hypr/.",
        try_it: Some("hyprctl version"),
    },
    Term {
        names: &["pacnew", "pacsave"],
        title: "Archivos .pacnew",
        text: "Si editaste una configuración y una actualización trae una versión nueva, pacman no pisa la tuya: guarda la nueva al lado con la extensión .pacnew. Conviene revisarlas de vez en cuando con pacdiff.",
        try_it: None,
    },
    Term {
        names: &["mirror", "mirrors", "espejo", "espejos"],
        title: "Mirror",
        text: "Un servidor que tiene una copia de los repositorios. Tu equipo descarga del primero de la lista en /etc/pacman.d/mirrorlist. Si las descargas van lentas, suele ser por el mirror.",
        try_it: None,
    },
    Term {
        names: &["swap", "intercambio"],
        title: "Swap",
        text: "Espacio en disco que el sistema usa como memoria de reserva cuando la RAM se llena. Evita que los programas se cierren de golpe, a cambio de ir más lento.",
        try_it: Some("free -h"),
    },
    Term {
        names: &["rolling release", "rolling"],
        title: "Rolling release",
        text: "Arch no tiene versiones grandes cada seis meses: se actualiza de forma continua, un poco cada día. Por eso se recomienda actualizar seguido y siempre completo.",
        try_it: None,
    },
    Term {
        names: &["pkgbuild", "makepkg"],
        title: "PKGBUILD",
        text: "La receta de un paquete: de dónde se descarga el código, cómo se compila y dónde se instala. Es un archivo de texto corto que puedes leer. makepkg es quien la ejecuta.",
        try_it: None,
    },
    Term {
        names: &["grub", "bootloader", "gestor de arranque", "systemd-boot", "limine"],
        title: "Gestor de arranque",
        text: "El primer programa que corre al encender: carga el kernel. Los más comunes en Arch son GRUB, systemd-boot y Limine. Si el equipo no arranca tras una actualización, el problema suele estar entre él y el kernel.",
        try_it: None,
    },
    Term {
        names: &["dotfiles", "dotfile", "rice", "ricing"],
        title: "Dotfiles",
        text: "Los archivos de configuración de tu usuario. Se llaman así porque empiezan por punto y por eso están ocultos. Viven en tu carpeta personal y en ~/.config/. Personalizar el escritorio con ellos se conoce como ricing.",
        try_it: Some("ls -a"),
    },
    Term {
        names: &["permisos", "permiso", "chmod"],
        title: "Permisos",
        text: "Cada archivo dice quién puede leerlo (r), escribirlo (w) y ejecutarlo (x), para tres grupos: el dueño, su grupo y el resto. Se ven con ls -l y se cambian con chmod.",
        try_it: Some("ls -l"),
    },
    Term {
        names: &["pipewire", "wireplumber", "pulseaudio"],
        title: "PipeWire",
        text: "El servidor de sonido actual de Linux: reparte el audio entre programas y dispositivos. WirePlumber es su ayudante, el que decide por qué altavoz sale cada cosa.",
        try_it: Some("wpctl status"),
    },
];

pub fn lookup(query: &str) -> Option<&'static Term> {
    TERMS.iter().find(|t| t.names.contains(&query))
}
