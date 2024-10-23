use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AppleMusic {
    pub data: Vec<AppleMusicData>,
}

#[derive(Serialize, Deserialize)]
pub struct AppleMusicData {
    pub relationships: Relationships,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Relationships {
    pub syllable_lyrics: Lyrics,
    pub lyrics: Lyrics,
}

#[derive(Serialize, Deserialize)]
pub struct Lyrics {
    pub href: String,
    pub data: Vec<LyricsDatum>,
}

#[derive(Serialize, Deserialize)]
pub struct LyricsDatum {
    pub id: String,
    #[serde(rename = "type")]
    pub datum_type: String,
    pub attributes: TentacledAttributes,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TentacledAttributes {
    pub ttml: String,
}
