//! Traducciones con Fluent. Todos los idiomas van embebidos en el binario.
//! Si a un idioma le falta una clave, se usa la española.

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};

/// Códigos de idioma disponibles; el primero es el base.
pub const LANGUAGES: [&str; 4] = ["es", "en", "pt", "ca"];

fn source(lang: &str) -> &'static str {
    match lang {
        "en" => include_str!("../../assets/i18n/en/main.ftl"),
        "pt" => include_str!("../../assets/i18n/pt/main.ftl"),
        "ca" => include_str!("../../assets/i18n/ca/main.ftl"),
        _ => include_str!("../../assets/i18n/es/main.ftl"),
    }
}

fn bundle(lang: &str) -> FluentBundle<FluentResource> {
    let res = FluentResource::try_new(source(lang).to_owned())
        .unwrap_or_else(|_| panic!("{lang}/main.ftl tiene errores de sintaxis"));
    let id = lang
        .parse::<unic_langid::LanguageIdentifier>()
        .expect("código de idioma válido");
    let mut bundle = FluentBundle::new(vec![id]);
    // Sin marcas de aislamiento Unicode: egui las mostraría como caracteres raros.
    bundle.set_use_isolating(false);
    bundle
        .add_resource(res)
        .unwrap_or_else(|_| panic!("claves repetidas en {lang}/main.ftl"));
    bundle
}

/// Idioma del sistema si está entre los disponibles; si no, español.
pub fn system_language() -> &'static str {
    let code = sys_locale::get_locale().unwrap_or_default().to_lowercase();
    LANGUAGES
        .into_iter()
        .find(|l| code.starts_with(l))
        .unwrap_or(LANGUAGES[0])
}

pub struct I18n {
    lang: &'static str,
    bundle: FluentBundle<FluentResource>,
    /// Español, para las claves que falten. `None` si el idioma ya es español.
    fallback: Option<FluentBundle<FluentResource>>,
}

impl I18n {
    /// Un código desconocido carga español.
    pub fn new(lang: &str) -> Self {
        let lang = LANGUAGES
            .into_iter()
            .find(|&l| l == lang)
            .unwrap_or(LANGUAGES[0]);
        Self {
            lang,
            bundle: bundle(lang),
            fallback: (lang != LANGUAGES[0]).then(|| bundle(LANGUAGES[0])),
        }
    }

    #[cfg(test)]
    pub fn spanish() -> Self {
        Self::new("es")
    }

    pub fn lang(&self) -> &'static str {
        self.lang
    }

    pub fn tr(&self, key: &str) -> String {
        self.format(key, None)
    }

    pub fn tr_args<'a>(&self, key: &str, args: &[(&'a str, FluentValue<'a>)]) -> String {
        let mut fa = FluentArgs::new();
        for (k, v) in args {
            fa.set(*k, v.clone());
        }
        self.format(key, Some(&fa))
    }

    /// Si la clave no está ni en español se muestra la propia clave, para que se note.
    fn format(&self, key: &str, args: Option<&FluentArgs>) -> String {
        for b in std::iter::once(&self.bundle).chain(&self.fallback) {
            if let Some(pattern) = b.get_message(key).and_then(|m| m.value()) {
                let mut errors = Vec::new();
                return b.format_pattern(pattern, args, &mut errors).into_owned();
            }
        }
        key.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_with_args() {
        let i = I18n::spanish();
        assert_eq!(i.tr("no-existe"), "no-existe");
        let s = i.tr_args("status-error", &[("detail", "x".into())]);
        assert!(s.contains('x') && !s.contains('{'), "{s}");
    }

    /// Toda clave literal que el código pasa a `tr` o `tr_args` debe existir en español.
    #[test]
    fn every_used_key_exists() {
        let i = I18n::spanish();
        let mut missing = Vec::new();
        let mut dirs = vec![std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src")];
        while let Some(dir) = dirs.pop() {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    dirs.push(path);
                    continue;
                }
                let code = std::fs::read_to_string(&path).unwrap();
                for chunk in code
                    .split("tr(\"")
                    .skip(1)
                    .chain(code.split("tr_args(\"").skip(1))
                {
                    let key = chunk.split('"').next().unwrap();
                    if key != "no-existe" && i.bundle.get_message(key).is_none() {
                        missing.push(format!("{}: {key}", path.display()));
                    }
                }
            }
        }
        assert!(missing.is_empty(), "claves sin traducir: {missing:?}");
    }

    fn keys(lang: &str) -> Vec<&'static str> {
        let mut k: Vec<_> = source(lang)
            .lines()
            .filter(|l| l.starts_with(|c: char| c.is_ascii_lowercase()))
            .filter_map(|l| l.split_once(" =").map(|(k, _)| k))
            .collect();
        k.sort_unstable();
        k
    }

    /// Todos los idiomas tienen exactamente las mismas claves que el español, y cargan.
    #[test]
    fn languages_match_spanish() {
        let es = keys("es");
        assert!(es.len() > 100);
        for lang in &LANGUAGES[1..] {
            let i = I18n::new(lang);
            assert_eq!(i.lang(), *lang);
            assert_eq!(keys(lang), es, "{lang}/main.ftl no tiene las mismas claves");
            for key in &es {
                assert!(i.bundle.get_message(key).is_some(), "{lang}: {key}");
            }
        }
    }

    #[test]
    fn plurals_and_fallback() {
        let en = I18n::new("en");
        let one = en.tr_args("status-palette-imported", &[("count", 1.into())]);
        let many = en.tr_args("status-palette-imported", &[("count", 5.into())]);
        assert_eq!(
            (one.as_str(), many.as_str()),
            (
                "Added 1 color to the palette.",
                "Added 5 colors to the palette."
            )
        );
        assert_eq!(I18n::new("xx").lang(), "es");
        assert_eq!(en.tr("no-existe"), "no-existe");
    }

    #[test]
    fn escaped_braces_render() {
        let i = I18n::spanish();
        let hint = i.tr("settings-export-name-hint");
        assert!(
            hint.contains("{name}") && hint.contains("{model}"),
            "{hint}"
        );
        assert_eq!(i.tr("unit-minutes"), " min");
    }
}
