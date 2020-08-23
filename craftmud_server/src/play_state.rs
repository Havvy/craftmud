//! The major state of what the user connected is doing.
//
// This might actually be useless since legion removed tags.

use crate::output::Output;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) enum PlayState {
    Login,
    ChooseCharacter,
    Playing,
    Tutorial,
    Quitting,
}

impl PlayState {
    pub(crate) fn preamble(&self) -> Option<&'static str> {
        match self {
            Self::Login => Some("Connected to CraftMud. Welcome!"),
            Self::ChooseCharacter => None,
            Self::Playing => None,
            Self::Tutorial => Some(crate::tutorial::PREAMBLE),
            Self::Quitting => Some("Bye"),
        }
    }

    pub(crate) fn transition(&self, output: &mut Output) {
        if let Some(preamble) = self.preamble() {
            output.push_static_paragraph(preamble)
        }
    }
}