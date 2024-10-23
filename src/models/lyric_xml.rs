use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename = "tt")]
pub struct LyricXML {
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
    #[serde(rename = "$text")]
    pub line: String,
}

