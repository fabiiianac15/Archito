//! Normalización de texto y emparejamiento por palabras completas.

/// Minúsculas, sin acentos y sin signos. Conserva `-`, `/`, `.` y `_` dentro
/// de las palabras para no destrozar nombres de paquetes ni flags.
pub fn norm(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.trim().chars().flat_map(char::to_lowercase) {
        let c = match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            '¿' | '?' | '¡' | '!' | ',' | ';' | ':' | '"' | '\'' | '(' | ')' => ' ',
            other => other,
        };
        out.push(c);
    }
    out.split_whitespace()
        .map(|w| w.trim_end_matches('.'))
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Longitud de la coincidencia de `keyword` dentro de `input`, o `None`.
///
/// La frase clave tiene que aparecer como palabras completas y contiguas: así
/// "tema" no coincide con "sistema" ni "ram" con "programa". Una palabra clave
/// terminada en `*` acepta cualquier terminación ("instal*" → instalar,
/// instalo, instalando).
pub fn phrase_match(input: &[&str], keyword: &str) -> Option<usize> {
    let kw: Vec<&str> = keyword.split_whitespace().collect();
    if kw.is_empty() || kw.len() > input.len() {
        return None;
    }
    let word_ok = |have: &str, want: &str| match want.strip_suffix('*') {
        Some(prefix) => have.starts_with(prefix),
        None => have == want,
    };
    (0..=input.len() - kw.len())
        .any(|start| kw.iter().enumerate().all(|(i, w)| word_ok(input[start + i], w)))
        .then(|| keyword.len())
}

/// Si `input` empieza por alguno de los prefijos, devuelve el resto.
pub fn strip_any<'a>(input: &'a str, prefixes: &[&str]) -> Option<&'a str> {
    prefixes.iter().find_map(|p| {
        let rest = input.strip_prefix(p)?;
        (rest.is_empty() || rest.starts_with(' ')).then(|| rest.trim())
    })
}

/// Barra de texto para porcentajes, usada por la CLI.
pub fn bar(pct: f32, width: usize) -> String {
    let filled = ((pct.clamp(0.0, 100.0) / 100.0) * width as f32).round() as usize;
    format!("[{}{}]", "█".repeat(filled), "░".repeat(width - filled))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks(s: &str) -> Vec<&str> {
        s.split_whitespace().collect()
    }

    #[test]
    fn normaliza_acentos_y_signos() {
        assert_eq!(norm("¿Cuánta RAM tengo?"), "cuanta ram tengo");
        assert_eq!(norm("  explica   sudo pacman -Syu. "), "explica sudo pacman -syu");
    }

    #[test]
    fn las_subcadenas_no_cuentan() {
        assert!(phrase_match(&toks("mi sistema no arranca"), "tema").is_none());
        assert!(phrase_match(&toks("como instalo un programa"), "ram").is_none());
        assert!(phrase_match(&toks("ahora no tengo sonido"), "hora").is_none());
        assert!(phrase_match(&toks("cuanta ram tengo"), "ram").is_some());
    }

    #[test]
    fn el_asterisco_acepta_terminaciones() {
        assert!(phrase_match(&toks("como instalo un programa"), "como instal*").is_some());
        assert!(phrase_match(&toks("quiero instalar algo"), "instal*").is_some());
    }
}
