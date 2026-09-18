"""
Gestión de rutas de assets (imágenes) de Archito.

Busca las imágenes en varias ubicaciones para que funcione
tanto desde el directorio del proyecto como instalado.
"""

from pathlib import Path


def get_assets_dir() -> Path:
    """
    Retorna el directorio donde están las imágenes.

    Busca en este orden:
    1. assets/img/ (junto al código del proyecto)
    2. img/        (carpeta original con las imágenes)
    3. ~/.local/share/archito/img/ (instalación del usuario)
    """
    here = Path(__file__).resolve().parent  # src/archito/ui/
    project_root = here.parent.parent.parent  # raíz del proyecto

    candidates = [
        project_root / "assets" / "img",
        project_root / "img",
        Path.home() / ".local" / "share" / "archito" / "img",
    ]

    for c in candidates:
        if c.is_dir():
            return c

    # Fallback: el directorio assets/img aunque no exista todavía
    return project_root / "assets" / "img"


def get_asset_path(name: str) -> Path | None:
    """Retorna la ruta completa a un asset, o None si no existe."""
    p = get_assets_dir() / name
    return p if p.exists() else None
