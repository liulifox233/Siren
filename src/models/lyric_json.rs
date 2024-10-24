use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LyricsJSON {
    lines: Vec<Line>,
}

#[derive(Serialize, Deserialize)]
pub struct Line {
    begin: String,
    end: String,
    words: Vec<Word>,
    background: Vec<Word>,
}

#[derive(Serialize, Deserialize)]
pub struct Word {
    begin: String,
    end: String,
    text: String,
}

impl LyricsJSON {
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    pub fn add_line(&mut self, line: Line) {
        self.lines.push(line);
    }
}

impl Line {
    pub fn new(begin: String, end: String) -> Self {
        Self {
            begin,
            end,
            words: Vec::new(),
            background: Vec::new(),
        }
    }

    pub fn add_words(&mut self, word: Word) {
        self.words.push(word);
    }

    pub fn add_background(&mut self, word: Word) {
        self.background.push(word);
    }
}

impl Word {
    pub fn new(begin: String, end: String, text: String) -> Self {
        Self { begin, end, text }
    }
}
