use crate::urls::types::Url;

pub trait BuildUrl {
    fn build_url(&self) -> String;
}

impl BuildUrl for Url {
    fn build_url(&self) -> String {
        let base_url = std::env::var("DOMAIN").expect("DOMAIN");
        let schema: &str = if base_url.starts_with("localhost") {
            "http://"
        } else {
            "https://"
        };
        format!("{}{}/{}", schema, base_url, &self.id)
    }
}
