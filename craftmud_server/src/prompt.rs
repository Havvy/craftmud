//! The prompt is shown when input is requested.

pub struct Prompt {
    format: String,
}

// This is probably wrong; I probably want a prompt that gets prompt-useful info
// such as player health.
impl std::fmt::Display for Prompt {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", &self.format)
    }
}

impl Default for Prompt {
    fn default() -> Self {
        Prompt {
            format: "\r\n> ".to_string(),
        }
    }
}

// TODO(Havvy, 2019-12-27, #generalization): Prompt system