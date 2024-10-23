use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename = "tt")]
pub struct SynedLyricXML {
    pub body: Body,
}

#[derive(Serialize, Deserialize)]
pub struct Body {
    pub div: Vec<Div>,
}

#[derive(Serialize, Deserialize)]
pub struct Div {
    pub p: Vec<P>,
}

#[derive(Serialize, Deserialize)]
pub struct P {
    #[serde(rename = "@begin")]
    pub begin: String,
    #[serde(rename = "@end")]
    pub end: String,
    pub span: Vec<Span>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Span {
    #[serde(rename = "@begin")]
    pub begin: Option<String>,
    #[serde(rename = "@end")]
    pub end: Option<String>,
    #[serde(rename = "$text")]
    pub word: Option<String>,
    pub span: Option<Vec<Span>>,
}
