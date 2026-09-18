"""
Backend simple de Archito — Milestone 0.0.1

Por ahora solo devuelve respuestas prefijadas.
En fases posteriores aquí se conectará el modelo de IA.
"""


def get_response(user_input: str) -> str:
    """Procesa el texto del usuario y devuelve una respuesta simple."""
    text = user_input.lower().strip()

    if not text:
        return "No te escuché bien. ¿Puedes repetir?"

    if any(w in text for w in ["hola", "buenas", "hey", "saludos", "qué tal", "que tal"]):
        return "¡Hola! Soy Archito.\nTodavía estoy aprendiendo, pero ya puedo escucharte."

    if any(w in text for w in ["quién eres", "quien eres", "qué eres", "que eres", "tu nombre"]):
        return "Soy Archito, un asistente para Arch Linux.\nEstoy en mi primera fase de desarrollo."

    if any(w in text for w in ["ayuda", "help", "ayúdame", "no sé", "no se"]):
        return "Aún estoy en desarrollo.\nPronto podré ayudarte a administrar tu sistema Arch Linux."

    if any(w in text for w in ["gracias", "thanks"]):
        return "¡De nada! Estoy aquí para ayudarte."

    if any(w in text for w in ["adiós", "adios", "chao", "bye", "cerrar", "salir"]):
        return "¡Hasta pronto!\nPresiona Escape o SUPER+ESPACIO para cerrarme."

    if any(w in text for w in ["arch", "linux", "cachyos", "pacman"]):
        return "¡Sí! Estoy diseñado para sistemas Arch Linux como CachyOS.\nPronto podré ayudarte más con tu sistema."

    return "Todavía estoy aprendiendo, pero ya puedo escucharte.\n¡Pronto podré ayudarte más!"
