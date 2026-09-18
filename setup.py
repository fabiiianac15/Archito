from setuptools import setup, find_packages

setup(
    name="archito",
    version="0.5.0",
    description="Asistente local para Arch Linux",
    packages=find_packages(where="src"),
    package_dir={"": "src"},
    python_requires=">=3.10",
    install_requires=[
        "PyGObject>=3.42",
    ],
    entry_points={
        "console_scripts": [
            "archito=archito.app:main",
        ],
    },
)
