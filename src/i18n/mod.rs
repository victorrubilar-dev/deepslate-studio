//! Traducciones con Fluent. En el hito 1 solo existe el español, embebido en el binario.

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};

const ES: &str = include_str!("../../assets/i18n/es/main.ftl");

pub struct I18n {
    bundle: FluentBundle<FluentResource>,
}

impl I18n {
    pub fn spanish() -> Self {
        let res =
            FluentResource::try_new(ES.to_owned()).expect("es/main.ftl tiene errores de sintaxis");
        let mut bundle = FluentBundle::new(vec![
            "es".parse::<unic_langid::LanguageIdentifier>()
                .expect("es es un código válido"),
        ]);
        // Sin marcas de aislamiento Unicode: egui las mostraría como caracteres raros.
        bundle.set_use_isolating(false);
        bundle
            .add_resource(res)
            .expect("claves repetidas en es/main.ftl");
        Self { bundle }
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

    /// Si falta la clave se muestra la propia clave, para que se note sin romper la app.
    fn format(&self, key: &str, args: Option<&FluentArgs>) -> String {
        let Some(pattern) = self.bundle.get_message(key).and_then(|m| m.value()) else {
            return key.to_owned();
        };
        let mut errors = Vec::new();
        self.bundle
            .format_pattern(pattern, args, &mut errors)
            .into_owned()
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
