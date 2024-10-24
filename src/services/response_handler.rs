use crate::models::apple_music::AppleMusic;
use crate::models::lyric_json::{Line, LyricsJSON, Word};
use crate::models::lyric_xml::LyricXML;
use crate::models::synced_lyric_xml::SynedLyricXML;
use std::char;
use std::{fs::File, io::Write};

pub struct Response {}

impl Response {
    /// Create and save data to json file
    pub(crate) fn create_file(data: &str, name: &str, extension: &str) {
        let name: String = format!("{}.{}", name, extension);
        let mut file = File::create(name).unwrap();
        file.write_all(data.as_bytes())
            .expect("Unable write data to file");
    }

    pub(crate) fn extract_lyrics_to_json(text: &str, space: bool) -> Result<LyricsJSON, String> {
        println!("{}", text);
        let mut lyrics = LyricsJSON::new();
        let response_json: AppleMusic =
            serde_json::from_str(text).map_err(|error| error.to_string())?;

        let synced_lyric_xml: SynedLyricXML = Self::extract_syned_lyric_xml(&response_json)?;
        let lyric_xml: LyricXML = Self::extract_lyric_xml(&response_json)?;

        if space {
            Self::convert_to_json_with_space(&synced_lyric_xml, &lyric_xml, &mut lyrics);
        } else {
            Self::convert_to_json(&synced_lyric_xml, &mut lyrics);
        }

        Ok(lyrics)
    }

    pub(crate) fn extract_lyrics_to_lrc(text: &str) -> Result<String, String> {
        let mut lrc: String = String::new();
        let response_json: AppleMusic =
            serde_json::from_str(text).map_err(|error| error.to_string())?;

        let synced_lyric_xml: SynedLyricXML = Self::extract_syned_lyric_xml(&response_json)?;

        Self::convert_to_lrc(&synced_lyric_xml, &mut lrc);
        Ok(lrc)
    }

    fn convert_to_lrc(synced_lyric_xml: &SynedLyricXML, lrc: &mut String) {
        let synced_lyric_array = &synced_lyric_xml.body.div;
        for div in synced_lyric_array {
            for p in &div.p {
                let line_start_time = Self::convert_time_to_lrc_format(&p.begin);
                let mut line: String = format!("[{}] ", line_start_time);
                for span in &p.span {
                    let span = span.clone();
                    match span.span {
                        Some(background) => {
                            for word in background {
                                let word_start_time =
                                    Self::convert_time_to_lrc_format(&word.begin.unwrap());
                                let word: String =
                                    format!("<{}> {} ", word_start_time, word.word.unwrap());
                                line.push_str(&word);
                            }
                        }
                        None => {
                            let word_start_time =
                                Self::convert_time_to_lrc_format(&span.begin.unwrap());
                            let word: String =
                                format!("<{}> {} ", word_start_time, span.word.unwrap());
                            line.push_str(&word);
                        }
                    }
                }
                line.push('\n');
                lrc.push_str(&line);
            }
        }
    }

    fn convert_time_to_lrc_format(time: &str) -> String {
        let mut min: String = String::new();
        let mut sec: String = String::new();
        let mut ms: String = String::new();

        let mut temp: String = String::new();
        for char in time.chars() {
            match char {
                ':' => {
                    min = temp.clone();
                    temp.clear();
                }
                '.' => {
                    sec = temp.clone();
                    temp.clear();
                }
                _ => temp.push(char),
            }
        }
        if !temp.is_empty() {
            ms = temp.clone();
        }

        if min.len() < 2 && !min.is_empty() {
            min.insert(0, '0');
        } else if min.is_empty() {
            min = "00".to_string();
        }
        if sec.len() < 2 && !sec.is_empty() {
            sec.insert(0, '0');
        } else if sec.is_empty() {
            sec = "00".to_string();
        }
        if ms.len() > 2 {
            ms.pop();
        }
        format!("{}:{}.{}", min, sec, ms)
    }

    /// Convert syned lyrics xml to json format
    fn convert_to_json(synced_lyric_xml: &SynedLyricXML, lyrics: &mut LyricsJSON) {
        let synced_lyric_array = &synced_lyric_xml.body.div;
        for div in synced_lyric_array {
            for p in &div.p {
                let mut line: Line = Line::new(p.begin.clone(), p.end.clone());
                for span in &p.span {
                    let span = span.clone();
                    match span.span {
                        Some(background) => {
                            for word in background {
                                let word: Word = Word::new(
                                    word.begin.unwrap(),
                                    word.end.unwrap(),
                                    word.word.unwrap(),
                                );
                                line.add_background(word);
                            }
                        }
                        None => {
                            let word: Word = Word::new(
                                span.begin.unwrap(),
                                span.end.unwrap(),
                                span.word.unwrap(),
                            );
                            line.add_words(word);
                        }
                    }
                }
                lyrics.add_line(line);
            }
        }
    }

    /// Convert syned lyrics xml to json format with space after space
    fn convert_to_json_with_space(
        synced_lyric_xml: &SynedLyricXML,
        lyric_xml: &LyricXML,
        lyrics: &mut LyricsJSON,
    ) {
        let synced_lyric_array = &synced_lyric_xml.body.div;
        let lyric_array = &lyric_xml.body.div;
        for (div_index, div) in synced_lyric_array.iter().enumerate() {
            for (p_index, p) in div.p.iter().enumerate() {
                let mut line: Line = Line::new(p.begin.clone(), p.end.clone());
                let lyric_chars: Vec<char> = lyric_array[div_index].p[p_index]
                    .line
                    .clone()
                    .chars()
                    .collect();
                let mut lyric_char_index: usize = 0;
                for span in &p.span {
                    let span = span.clone();
                    match span.span {
                        Some(background) => {
                            for span in background {
                                let mut word = span.word.clone().unwrap();
                                let chars: Vec<char> = word.chars().collect();
                                Self::add_spaces(
                                    chars,
                                    &lyric_chars,
                                    &mut lyric_char_index,
                                    &mut word,
                                );
                                let word: Word =
                                    Word::new(span.begin.unwrap(), span.end.unwrap(), word);
                                line.add_background(word);
                            }
                        }
                        None => {
                            let mut word = span.word.clone().unwrap();
                            let chars: Vec<char> = word.chars().collect();
                            Self::add_spaces(chars, &lyric_chars, &mut lyric_char_index, &mut word);
                            let word: Word =
                                Word::new(span.begin.unwrap(), span.end.unwrap(), word);
                            line.add_words(word);
                        }
                    }
                }
                lyrics.add_line(line);
            }
        }
    }

    /// Add space after word
    fn add_spaces(
        synced_lyric_chars: Vec<char>,
        lyric_chars: &[char],
        lyric_char_index: &mut usize,
        word: &mut String,
    ) {
        for c in synced_lyric_chars {
            if *lyric_char_index >= lyric_chars.len() {
                println!("{:}", word);
                return;
            }
            if c == lyric_chars[*lyric_char_index] {
                *lyric_char_index += 1;
                continue;
            }
        }
        if *lyric_char_index >= lyric_chars.len() {
            return;
        }
        if lyric_chars[*lyric_char_index] == ' ' {
            *lyric_char_index += 1;
            word.push(' ');
        }
    }

    /// Extract syned lyric xml response
    fn extract_syned_lyric_xml(json: &AppleMusic) -> Result<SynedLyricXML, String> {
        let data = match json.data.first() {
            Some(data) => data,
            None => Err("empty data".to_string())?,
        };
        let ttml = match data.relationships.syllable_lyrics.data.first() {
            Some(lyric_data) => &lyric_data.attributes.ttml,
            None => Err("empty data".to_string())?,
        };
        let output: SynedLyricXML = match quick_xml::de::from_str(ttml) {
            Ok(data) => data,
            Err(error) => Err(error.to_string())?,
        };
        Ok(output)
    }

    /// Extract lyric xml response
    fn extract_lyric_xml(json: &AppleMusic) -> Result<LyricXML, String> {
        let data = match json.data.first() {
            Some(data) => data,
            None => Err("empty data".to_string())?,
        };
        let ttml = match data.relationships.lyrics.data.first() {
            Some(lyric_data) => &lyric_data.attributes.ttml,
            None => Err("empty data".to_string())?,
        };
        let output: LyricXML = match quick_xml::de::from_str(ttml) {
            Ok(data) => data,
            Err(error) => Err(error.to_string())?,
        };
        Ok(output)
    }
}
