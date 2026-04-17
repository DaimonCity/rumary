use rust_embed::RustEmbed;
use serde_json::Value;
use std::collections::HashMap;

#[derive(RustEmbed)]
#[folder = "translations/"]
struct TranslationAsset;

#[derive(Clone)]
pub struct Translator {
    current_lang: String,
    values: HashMap<String, String>,
}

impl Translator {
    pub fn new(default_lang: &str) -> Self {
        let mut translator = Self {
            current_lang: default_lang.to_string(),
            values: HashMap::new(),
        };
        translator.load();
        translator
    }

    pub fn set_language(&mut self, lang: &str) {
        self.current_lang = lang.to_string();
        self.load();
    }

    pub fn t(&self, key: &str) -> String {
        self.values
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }

    // Эта функция способна распаковать из json случай: "foo": {...}
    fn load(&mut self) {
        let filename = format!("{}.json", self.current_lang);
        if let Some(file) = TranslationAsset::get(&filename) &&
            let Ok(value) = serde_json::from_slice::<Value>(&file.data) && // может, это можно переписать под наши utils
            let Value::Object(map) = value
        {
            self.values = map
                .into_iter()
                .map(|(key, value)| (key, value.as_str().unwrap_or_default().to_string()))
                .collect();
        }
    }

    // Эта функция нужна только для формата {"foo": "value", "foo1": "value1"...}
    fn _load(&mut self) {
        let filename = format!("{}.json", self.current_lang);

        if let Some(file) = TranslationAsset::get(&filename)
            && let Ok(map) = serde_json::from_slice::<HashMap<String, String>>(&file.data)
        {
            self.values = map;
        }
    }
}
