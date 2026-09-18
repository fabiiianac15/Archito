"""Lectura preventiva del estado del sistema, sin ejecutar cambios."""

from dataclasses import dataclass, field

from archito.core import system


@dataclass
class HealthReport:
    score: int
    state: str
    stats: list[tuple] = field(default_factory=list)
    findings: list[str] = field(default_factory=list)
    chips: list[str] = field(default_factory=list)


def inspect() -> HealthReport:
    """Compone una revisión breve con señales reales y recomendaciones concretas."""
    score = 100
    findings: list[str] = []
    stats: list[tuple] = []
    chips: list[str] = ["por qué va lento", "servicios fallidos"]

    ram = system.ram_report()
    if ram:
        severity = "danger" if ram["pct"] >= 90 else ("warn" if ram["pct"] >= 78 else "good")
        stats.append(("RAM", ram["pct"],
                      f"{ram['used_gb']:.1f}/{ram['total_gb']:.0f} GB", severity))
        if ram["pct"] >= 90:
            score -= 28
            findings.append("🐏 RAM muy alta — revisa los procesos que más memoria usan.")
        elif ram["pct"] >= 78:
            score -= 12
            findings.append("🐏 RAM bastante ocupada — todavía funciona, pero conviene vigilarla.")

    disk = system.disk_report("/")
    if disk:
        severity = "danger" if disk["pct"] >= 92 else ("warn" if disk["pct"] >= 78 else "good")
        stats.append(("Disco", disk["pct"], f"{disk['free_gb']:.0f} GB libres", severity))
        if disk["pct"] >= 92:
            score -= 30
            findings.append("💾 Disco casi lleno — libera espacio antes de que afecte al sistema.")
        elif disk["pct"] >= 78:
            score -= 10
            findings.append("💾 El disco se está llenando — puedes revisar paquetes grandes o caché.")

    temp = system.cpu_temp()
    if temp:
        celsius = temp[0]
        severity = "danger" if celsius >= 85 else ("warn" if celsius >= 75 else "good")
        stats.append(("CPU", min(100, celsius), f"{celsius:.0f} °C", severity))
        if celsius >= 85:
            score -= 24
            findings.append("🌡️ CPU muy caliente — revisa ventilación y procesos intensivos.")
        elif celsius >= 75:
            score -= 9
            findings.append("🌡️ CPU caliente — observa si la temperatura sigue subiendo.")

    battery = system.battery()
    if battery and battery["status"] == "Discharging":
        percent = battery["percent"]
        severity = "danger" if percent <= 15 else ("warn" if percent <= 30 else "good")
        stats.append(("Batería", percent, f"{percent}%", severity))
        if percent <= 15:
            score -= 8
            findings.append("🔋 Batería baja — conecta el cargador cuando puedas.")

    failed = system.failed_services() or []
    if failed:
        score -= min(24, 8 * len(failed))
        shown = ", ".join(failed[:3])
        findings.append(f"⚠️ {len(failed)} servicio(s) fallido(s): {shown}.")

    updates = system.available_updates()
    if updates:
        findings.append(f"📦 Hay {len(updates)} actualización(es) pendiente(s); Arch recomienda mantenerlas al día.")
        chips.insert(0, "actualizaciones")

    score = max(0, score)
    if score >= 90:
        state = "sereno"
    elif score >= 65:
        state = "atención"
    else:
        state = "prioridad"
    if not findings:
        findings.append("✨ No detecté señales preocupantes en esta revisión.")
    return HealthReport(score=score, state=state, stats=stats[:4],
                        findings=findings, chips=chips)
