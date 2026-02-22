use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BodyFormat {
    #[default]
    Markdown,
    Html,
    Text,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SourceConfig {
    pub primary: Option<bool>,
    pub join: Option<HashMap<String, String>>,
    pub many: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Template {
    pub sources: HashMap<String, SourceConfig>,
    pub to: String,
    pub cc: Option<String>,
    pub bcc: Option<String>,
    pub subject: String,
    pub body: String,
    pub attachments: Option<String>,
    pub body_format: Option<BodyFormat>,
    pub stylesheet: Option<String>,
    pub style: Option<String>,
}
