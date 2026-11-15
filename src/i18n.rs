use std::{collections::HashMap, sync::OnceLock};

pub fn text(key: &str, french: bool) -> &str {
    static ENGLISH: OnceLock<HashMap<String, String>> = OnceLock::new();
    static FRENCH: OnceLock<HashMap<String, String>> = OnceLock::new();
    let english = ENGLISH.get_or_init(|| {
        serde_json::from_str(include_str!("../locales/en.json")).expect("validated English catalog")
    });
    let french_catalog = FRENCH.get_or_init(|| {
        serde_json::from_str(include_str!("../locales/fr.json")).expect("validated French catalog")
    });
    if french {
        french_catalog
            .get(key)
            .or_else(|| english.get(key))
            .map(String::as_str)
            .unwrap_or(key)
    } else {
        english.get(key).map(String::as_str).unwrap_or(key)
    }
}

pub fn template(key: &str, french: bool, values: &[(&str, String)]) -> String {
    let mut output = text(key, french).to_string();
    for (name, value) in values {
        output = output.replace(&format!("{{{name}}}"), value);
    }
    output
}
