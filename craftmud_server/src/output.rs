//! Output of the game server to an individual player.

use std::borrow::Cow;

pub struct Output {
    paragraphs: Vec<Cow<'static, str>>,
    prompt: Option<String>,
}

impl Output {
    pub fn new() -> Self {
        Self {
            paragraphs: Vec::with_capacity(4),
            prompt: None,
        }
    }

    pub fn push_static_paragraph(&mut self, paragraph: &'static str) {
        self.paragraphs.push(paragraph.into());
    }

    pub fn push_paragraph(&mut self, paragraph: String) {
        self.paragraphs.push(paragraph.into());
    }

    pub fn set_prompt(&mut self, prompt: String) {
        self.prompt = Some(prompt);
    }
}

pub trait OptionOutputExt {
    fn push_static_paragraph(&mut self, paragraph: &'static str);
    fn push_paragraph(&mut self, paragraph: String);
}

impl OptionOutputExt for Option<Output> {
    fn push_static_paragraph(&mut self, paragraph: &'static str) {
        let output = self.get_or_insert_with(Default::default);
        output.push_static_paragraph(paragraph);
    }

    fn push_paragraph(&mut self, paragraph: String) {
        let output = self.get_or_insert_with(Default::default);
        output.push_paragraph(paragraph);
    }
}

impl Default for Output {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("\r\n");

        for paragraph in &self.paragraphs {
            f.write_str(&*paragraph);
            f.write_str("\r\n");
        }

        if let Some(ref prompt) = self.prompt {
            f.write_str(prompt);
        }

        Ok(())
    }
}