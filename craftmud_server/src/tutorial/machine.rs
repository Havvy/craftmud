use derive_more::From as DeriveFrom;

use crate::play_state::PlayState;
use super::{Data, TutorialRealm};
use super::messages;

pub(super) struct HandledBy {
    pub machine: Machine,
    pub action: HandledByAction,
}

pub(super) enum HandledByAction {
    PlayStateTrans(PlayState),
    InputStateTrans,
    InputStateTransWithMessage(String),
    DoNothing,
    OutputMessage(String),
}

pub(super) trait State: std::fmt::Debug + Sized + Into<Machine> {
    fn handle_input(self, input: String, data: &mut Data, realm: &TutorialRealm) -> HandledBy;
    fn allow_command(&self, command: &str) -> bool;
}

#[derive(Debug, DeriveFrom)]
pub(super) enum Machine {
    Intro(Intro),
    Terminal(Terminal),
}

impl Machine {
    pub fn new() -> Self {
        Self::Intro(Intro::new())
    }

    pub fn handle_input(self, input: String, data: &mut Data, realm: &TutorialRealm) -> HandledBy {
        match &*input {
            "quitout" => HandledBy { machine: Terminal.into(), action: HandledByAction::PlayStateTrans(PlayState::Quitting) },
            "logout" => HandledBy { machine: Terminal.into(), action: HandledByAction::PlayStateTrans(PlayState::Login) },
            "" => HandledBy { machine: self.into(), action: HandledByAction::DoNothing },
            _ => match self {
                Machine::Intro(state) => State::handle_input(state, input, data, realm),
                Machine::Terminal(state) => State::handle_input(state, input, data, realm),
            },
        }
    }

    pub fn take(&mut self) -> Self {
        std::mem::replace(self, Self::Terminal(Terminal))
    }

    pub fn untake(&mut self, untaken: Machine) {
        std::mem::replace(self, untaken);
    }
}

#[derive(Debug)]
pub struct Intro(usize);

impl State for Intro {
    fn handle_input(self, input: String, data: &mut Data, realm: &TutorialRealm) -> HandledBy {
        match &*input {
            "next" => HandledBy {
                machine: Intro(1).into(),
                action: HandledByAction::OutputMessage(Self::MESSAGES[0].into()),
            },

            _ => {
                if self.0 == 0 {
                    HandledBy {
                        machine: self.into(),
                        action: HandledByAction::OutputMessage(Self::USE_NEXT.into()),
                    }
                } else {
                    HandledBy {
                        machine: self.into(),
                        action: HandledByAction::OutputMessage(look_enabled(data, realm)),
                    }
                }
            },
        }
    }

    fn allow_command(&self, command: &str) -> bool {
        match command {
            "next" => true,
            "look" if self.0 > 0 => true,
            _ => false,
        }
    }
}

impl Intro {
    const MESSAGES: [&'static str; messages::INTRO_LEN] = messages::INTRO;

    const USE_NEXT: &'static str = "You sent a command, but it wasn't `next`.\r\n\
    Try again, but this time, use the `next` command.";

    fn new() -> Self {
        Self(0)
    }
}

#[derive(Debug)]
pub struct Terminal;

impl State for Terminal {
    fn handle_input(self, _input: String, _data: &mut Data, _realm: &TutorialRealm) -> HandledBy {
        panic!("Method call on terminal state in Tutorial Machine");
    }

    fn allow_command(&self, command: &str) -> bool {
        panic!("Method call on terminal state in Tutorial Machine");
    }
}

fn look_enabled(data: &Data, realm: &TutorialRealm) -> String {
    let mut s = String::new();
    realm.0[data.place].look(&mut s);
    s
}