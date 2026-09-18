"""
Módulo del sistema — FASE 6 del roadmap.

Todas las funciones aquí son de SOLO LECTURA (Nivel 1 — Seguro):
no modifican nada, no necesitan sudo, no usan Internet.

Cada sonda devuelve texto ya formateado para el chat de Archito,
o None si la información no está disponible en esta máquina.
"""

import os
import re
import socket
import subprocess
import time
from pathlib import Path

from archito.core.actions import run_inline


def _run(cmd: list[str], timeout: int = 8) -> str | None:
    """Ejecuta un comando de lectura y devuelve stdout o None."""
    try:
        r = subprocess.run(
            cmd, capture_output=True, text=True, timeout=timeout
        )
        if r.returncode == 0 and r.stdout.strip():
            return r.stdout.strip()
    except (FileNotFoundError, subprocess.TimeoutExpired, OSError):
        pass
    return None


def _read(path: str) -> str | None:
    try:
        with open(path, encoding="utf-8", errors="replace") as f:
            return f.read().strip()
    except OSError:
        return None


# ──────────────────────────────────────────────
# Identidad
# ──────────────────────────────────────────────

def kernel_version() -> str | None:
    return _run(["uname", "-r"])


def distribution() -> str | None:
    txt = _read("/etc/os-release")
    if txt:
        for line in txt.splitlines():
            if line.startswith("PRETTY_NAME="):
                return line.split("=", 1)[1].strip('"')
    return None


def hostname() -> str | None:
    try:
        return socket.gethostname() or None
    except OSError:
        return None


def current_user() -> str | None:
    return os.environ.get("USER") or os.environ.get("LOGNAME")


def shell_name() -> str | None:
    sh = os.environ.get("SHELL", "")
    return sh.split("/")[-1] or None


def desktop_env() -> str | None:
    return os.environ.get("XDG_CURRENT_DESKTOP") or (
        "Hyprland" if os.environ.get("HYPRLAND_INSTANCE_SIGNATURE") else None
    )


def is_hyprland() -> bool:
    return bool(os.environ.get("HYPRLAND_INSTANCE_SIGNATURE"))


def uptime_pretty() -> str | None:
    secs = _read("/proc/uptime")
    if not secs:
        return None
    total = int(float(secs.split()[0]))
    days, rem = divmod(total, 86400)
    hours, rem = divmod(rem, 3600)
    mins = rem // 60
    parts = []
    if days:
        parts.append(f"{days}d")
    if hours:
        parts.append(f"{hours}h")
    parts.append(f"{mins}m")
    return " ".join(parts)


def boot_time() -> str | None:
    out = _run(["systemd-analyze"])
    if out and "boot" in out.lower():
        return out.splitlines()[0].replace("Startup finished in ", "").strip()
    return None


# ──────────────────────────────────────────────
# Hardware
# ──────────────────────────────────────────────

def cpu_model() -> str | None:
    info = _read("/proc/cpuinfo")
    if info:
        for line in info.splitlines():
            if line.startswith("model name"):
                return line.split(":", 1)[1].strip()
    return None


def cpu_cores() -> int | None:
    n = os.cpu_count()
    return n


def cpu_usage() -> float | None:
    """Uso de CPU global (%) muestreando /proc/stat."""
    def sample():
        txt = _read("/proc/stat")
        if not txt:
            return None
        parts = txt.splitlines()[0].split()[1:]
        vals = [int(x) for x in parts]
        idle = vals[3] + (vals[4] if len(vals) > 4 else 0)
        total = sum(vals)
        return idle, total

    a = sample()
    if a is None:
        return None
    time.sleep(0.15)
    b = sample()
    if b is None:
        return None
    dt = b[1] - a[1]
    di = b[0] - a[0]
    if dt <= 0:
        return None
    return round((1 - di / dt) * 100, 1)


def cpu_temp() -> tuple[float, str] | None:
    """Temperatura más alta encontrada + nombre del sensor."""
    best = None
    base = Path("/sys/class/hwmon")
    try:
        for hwmon in base.iterdir():
            name = (_read(str(hwmon / "name")) or "").lower()
            for tfile in sorted(hwmon.glob("temp*_input")):
                val = _read(str(tfile))
                if not val:
                    continue
                celsius = int(val) / 1000.0
                label_file = tfile.with_name(tfile.name.replace("_input", "_label"))
                label = _read(str(label_file)) or name or "sensor"
                if best is None or celsius > best[0]:
                    best = (round(celsius, 1), label)
    except OSError:
        pass
    if best:
        return best
    # Fallback: thermal zones
    try:
        for zone in sorted(Path("/sys/class/thermal").glob("thermal_zone*")):
            ztype = (_read(str(zone / "type")) or "zona").lower()
            val = _read(str(zone / "temp"))
            if val:
                c = int(val) / 1000.0
                if best is None or c > best[0]:
                    best = (round(c, 1), ztype)
    except OSError:
        pass
    return best


def battery() -> dict | None:
    base = Path("/sys/class/power_supply")
    try:
        for bat in sorted(base.glob("BAT*")):
            cap = _read(str(bat / "capacity"))
            status = _read(str(bat / "status"))
            if cap:
                icons = {"Charging": "⚡", "Discharging": "🔋",
                         "Full": "✅", "Not charging": "🔌"}
                return {
                    "percent": int(cap),
                    "status": status or "?",
                    "icon": icons.get(status, "🔋"),
                }
    except OSError:
        pass
    return None


def gpu_info() -> str | None:
    out = _run(["lspci"])
    if not out:
        return None
    gpus = []
    for line in out.splitlines():
        if re.search(r"VGA compatible|3D controller|Display controller", line, re.I):
            desc = line.split(":", 2)[-1].strip()
            desc = re.sub(r"\(rev [0-9a-f]+\)", "", desc).strip()
            gpus.append(desc)
    return "\n".join(gpus[:2]) or None


def ram_report() -> dict | None:
    txt = _read("/proc/meminfo")
    if not txt:
        return None
    vals = {}
    for line in txt.splitlines():
        k, _, v = line.partition(":")
        vals[k.strip()] = int(re.sub(r"[^0-9]", "", v) or 0)
    total = vals.get("MemTotal", 0)
    avail = vals.get("MemAvailable", 0)
    if not total:
        return None
    used = total - avail
    pct = used / total * 100
    swap_total = vals.get("SwapTotal", 0)
    swap_free = vals.get("SwapFree", 0)
    return {
        "total_gb": total / 1048576,
        "used_gb": used / 1048576,
        "pct": round(pct, 1),
        "swap_pct": round((1 - swap_free / swap_total) * 100, 1) if swap_total else None,
    }


def disk_report(path: str = "/") -> dict | None:
    st = os.statvfs(path)
    total = st.f_blocks * st.f_frsize
    free = st.f_bavail * st.f_frsize
    used = total - free
    if not total:
        return None
    return {
        "total_gb": total / 1073741824,
        "used_gb": used / 1073741824,
        "free_gb": free / 1073741824,
        "pct": round(used / total * 100, 1),
    }


def top_processes(by: str = "cpu", limit: int = 5) -> list[dict]:
    sort_key = "-pcpu" if by == "cpu" else "-pmem"
    out = _run([
        "ps", "-eo", "pid,pcpu,pmem,comm", f"--sort={sort_key}", "--no-headers"
    ])
    procs = []
    if out:
        for line in out.splitlines()[:limit]:
            parts = line.split(None, 3)
            if len(parts) == 4:
                procs.append({
                    "pid": parts[0],
                    "cpu": float(parts[1]),
                    "mem": float(parts[2]),
                    "name": parts[3][:24],
                })
    return procs


# ──────────────────────────────────────────────
# Paquetes
# ──────────────────────────────────────────────

def package_count() -> int | None:
    out = run_inline("pacman -Q").output
    if out and out != "(sin salida)":
        lines = [l for l in out.splitlines() if l.strip()]
        if not lines[0].startswith(("…", "(")):
            return len(lines)
    return None


def recent_packages(limit: int = 5) -> list[tuple[str, str]]:
    out = _run(["bash", "-c", "grep -i 'installed' /var/log/pacman.log | tail -5"])
    pkgs = []
    if out:
        for line in out.splitlines():
            m = re.search(r"installed (\S+) \((\S+)\)", line)
            if m:
                pkgs.append((m.group(1), m.group(2)))
    if not pkgs:
        out2 = _run(["bash", "-c", "ls -t /var/lib/pacman/local/*.desc 2>/dev/null | head -5"])
        if out2:
            for p in out2.splitlines():
                name = Path(p).parent.name
                ver = name.rsplit("-", 2)[-2] if "-" in name else ""
                pkgs.append((name.rsplit("-", 2)[0], ver))
    return pkgs[:limit]


def search_packages(term: str, limit: int = 6) -> list[tuple[str, str]] | None:
    out = _run(["pacman", "-Ss", term])
    if out is None:
        return None
    results = []
    block_name, block_desc = None, ""
    for line in out.splitlines():
        if line.startswith(("core/", "extra/", "multilib/")) or "/" in line.split(" ")[0]:
            repo_name = line.split(" ")[0]
            results.append((repo_name, block_desc))
            block_desc = ""
            if "/" in repo_name:
                block_name = repo_name
        elif line.startswith("    ") and block_name:
            block_desc = line.strip()
    # Reconstruir correctamente: pacman -Ss alterna línea repo/name y descripción
    results = []
    lines = out.splitlines()
    for i in range(0, len(lines) - 1):
        l1, l2 = lines[i], lines[i + 1]
        if "/" in l1.split(" ")[0] and l2.startswith("    "):
            results.append((l1.split(" ")[0], l2.strip()))
        if len(results) >= limit:
            break
    return results


def package_info(pkg: str) -> str | None:
    installed = _run(["pacman", "-Qi", pkg])
    if installed:
        return installed
    return _run(["pacman", "-Si", pkg])


def orphan_packages() -> list[str]:
    out = _run(["pacman", "-Qtdq"])
    return out.splitlines() if out else []


def biggest_packages(limit: int = 6) -> list[tuple[str, float]] | None:
    out = _run(["bash", "-c", "pacman -Qi $(pacman -Qq 2>/dev/null) 2>/dev/null"],
               timeout=25)
    if not out:
        return None
    sizes = []
    current = None
    size_keys = ("Tamaño de la instalación", "Tamaño instalado", "Installed Size")
    for line in out.splitlines():
        if line.startswith(("Nombre", "Name")):
            current = line.split(":", 1)[1].strip()
        elif line.startswith(size_keys):
            raw = line.split(":", 1)[1].strip()
            m = re.match(r"([\d.,]+)\s*(MiB|GiB|KiB)", raw)
            if m and current:
                num = float(m.group(1).replace(",", "."))
                mult = {"KiB": 1 / 1024, "MiB": 1, "GiB": 1024}[m.group(2)]
                sizes.append((current, num * mult))
                current = None
    sizes.sort(key=lambda x: -x[1])
    return sizes[:limit]


def pacman_cache_size() -> str | None:
    out = _run(["du", "-sh", "/var/cache/pacman/pkg"], timeout=10)
    return out.split("\t")[0] if out else None


def user_cache_size() -> str | None:
    out = _run(["du", "-sh", os.path.expanduser("~/.cache")], timeout=15)
    return out.split("\t")[0] if out else None


def available_updates() -> list[str]:
    out = _run(["checkupdates"], timeout=25)
    if not out:
        out = _run(["pacman", "-Qu"], timeout=25)
    names = []
    if out:
        for line in out.splitlines():
            name = line.split(" ")[0]
            if name and "->" not in name:
                names.append(name)
    return names


# ──────────────────────────────────────────────
# Servicios y logs
# ──────────────────────────────────────────────

def failed_services() -> list[str]:
    out = _run(["systemctl", "--failed", "--no-legend", "--plain"])
    units = []
    if out:
        for line in out.splitlines():
            unit = line.split(".")[0] if "." in line else line.split()[0]
            full = line.split()[0]
            units.append(full)
    return units


def recent_errors(limit: int = 5) -> str | None:
    return _run(["journalctl", "-p", "err", "-n", str(limit), "--no-pager", "-q"],
                timeout=10)


# ──────────────────────────────────────────────
# Red
# ──────────────────────────────────────────────

def local_ip() -> str | None:
    out = _run(["hostname", "-I"])
    if out:
        return out.split()[0]
    return None


def network_interfaces() -> str | None:
    return _run(["ip", "-brief", "address"])


def wifi_networks(limit: int = 6) -> list[tuple[str, str]] | None:
    out = _run(["nmcli", "-t", "-f", "SSID,SIGNAL,SECURITY", "device", "wifi",
                "list"], timeout=12)
    if out is None:
        return None
    nets = []
    for line in out.splitlines():
        parts = line.split(":")
        if len(parts) >= 3 and parts[0]:
            nets.append((parts[0], parts[1], parts[2] if len(parts) > 2 else ""))
    nets.sort(key=lambda n: -int(n[1] or 0))
    return [(n[0], n[1]) for n in nets[:limit]]


def internet_connected() -> bool | None:
    out = _run(["ping", "-c", "1", "-W", "2", "1.1.1.1"], timeout=5)
    if out is None:
        return False
    return True


def bluetooth_state() -> str | None:
    out = _run(["bluetoothctl", "show"])
    if not out:
        return None
    powered = "on" if "Powered: yes" in out else "off"
    devs = _run(["bluetoothctl", "devices"]) or ""
    count = len([l for l in devs.splitlines() if l.strip()])
    return f"Bluetooth {powered} · {count} dispositivo(s) emparejados"


def audio_state() -> str | None:
    out = _run(["wpctl", "get-volume", "@DEFAULT_AUDIO_SINK@"])
    if not out:
        return None
    muted = "MUTED" in out.upper()
    vol = re.search(r"([\d.]+)", out)
    pct = round(float(vol.group(1)) * 100) if vol else 0
    return f"{'🔇 Silenciado' if muted else f'🔊 {pct}%'}"


# ──────────────────────────────────────────────
# Hyprland
# ──────────────────────────────────────────────

def hypr_workspace() -> str | None:
    if not is_hyprland():
        return None
    out = _run(["hyprctl", "activeworkspace"])
    if out:
        m = re.search(r"workspace ID (\d+)", out)
        if m:
            return f"Workspace {m.group(1)}"
    return None


def hypr_monitors() -> str | None:
    if not is_hyprland():
        return None
    out = _run(["hyprctl", "monitors"])
    if out:
        res = re.findall(r"(\d+x[\d.]+@\d+\.\d+)", out)
        return ", ".join(res) if res else None
    return None


# ──────────────────────────────────────────────
# Resumen completo (para el diagnóstico — FASE 10)
# ──────────────────────────────────────────────

def quick_summary() -> str:
    distro = distribution() or "¿?"
    kernel = kernel_version() or "¿?"
    up = uptime_pretty() or "¿?"
    ram = ram_report()
    disk = disk_report()
    bat = battery()
    lines = [
        f"$ Sistema: {distro} · kernel {kernel}",
        f"$ Encendido desde hace {up}",
    ]
    if ram:
        lines.append(f"$ RAM: {ram['pct']}% usado ({ram['used_gb']:.1f}/{ram['total_gb']:.0f} GB)")
    if disk:
        lines.append(f"$ Disco: {disk['pct']}% usado ({disk['free_gb']:.0f} GB libres)")
    if bat:
        lines.append(f"$ Batería: {bat['percent']}% ({bat['status']})")
    temp = cpu_temp()
    if temp:
        lines.append(f"$ Temperatura CPU: {temp[0]:.0f}°C ({temp[1]})")
    return "\n".join(lines)
