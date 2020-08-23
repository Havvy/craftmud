// TODO(Havvy, 2020-08-22, #sec): Proper password security.

use super::*;
use derive_more::From as DeriveFrom;

use crate::models::{Account, AccountPasswordInsert, AccountInsert, UniqueAccountError};

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
    const PREAMBLE: Option<&'static str> = None;
    const WAITING_ON_DB: bool = false;

    type Previous: State + Into<Machine>;

    // Warning: This function should not be overriden. Each state should
    // override handle_input_impl instead.
    fn handle_input(self, input: String, db: &Database) -> HandledBy {
        match &*input {
            "back" => HandledBy { machine: self.previous().into(), action: HandledByAction::InputStateTrans },
            "quit" => HandledBy { machine: Terminal.into(), action: HandledByAction::PlayStateTrans(PlayState::Quitting) },
            "" => HandledBy { machine: self.into(), action: HandledByAction::DoNothing },
            _ => self.handle_input_impl(input, db)
        }
    }

    fn handle_input_impl(self, input: String, db: &Database) -> HandledBy;

    fn handle_db_response(self) -> HandledBy {
        panic!("Trying to handle a db response on a state that isn't waiting for a db response.");
    }

    fn previous(self) -> Self::Previous;
}

#[derive(Debug)]
pub(super) struct JustConnected;

impl State for JustConnected {
    const PREAMBLE: Option<&'static str> = Some("To log in, please state your account name. Othewrise, `new` or `tutorial`\r\n\
    to create a new account or start a tutorial if this is your first MUD.\r\n
    You can also use `quit` at any time to have the server disconnect you.\r\n
    If you want to go back a step, use `back`.");

    type Previous = JustConnected;

    fn handle_input_impl(self, input: String, _db: &Database) -> HandledBy {
        match &*input {
            "new" => HandledBy { machine: RegisterRequestName.into(), action: HandledByAction::InputStateTrans, },

            "tutorial" => HandledBy { machine: Terminal.into(), action: HandledByAction::PlayStateTrans(PlayState::Tutorial), },

            _ => HandledBy { machine: LoginRequestPassword(AccountName(input)).into(), action: HandledByAction::InputStateTrans, },
        }
    }

    fn previous(self) -> <Self as State>::Previous {
        self
    }
}

#[derive(Debug)]
pub(super) struct RegisterRequestName;

impl RegisterRequestName {
    const NAME_BANNED_MESSAGE: &'static str = "Disallowed account name. Try again.\r\n";
}

impl State for RegisterRequestName {
    const PREAMBLE: Option<&'static str> = Some("What account name do you want?\r\n\
    Note: This is not a character name. This is the name to refer to the real you.\r\n");

    type Previous = JustConnected;

    fn handle_input_impl(self, input: String, _db: &Database) -> HandledBy {
        if AccountName::is_banned(&input) {
            HandledBy { machine: self.into(), action: HandledByAction::OutputMessage(Self::NAME_BANNED_MESSAGE.to_string()), }
        } else {
            HandledBy { machine: RegisterRequestEmail(AccountName(input)).into(), action: HandledByAction::InputStateTrans, }
        }
    }

    fn previous(self) -> <Self as State>::Previous {
        JustConnected
    }
}

#[derive(Debug)]
pub(super) struct RegisterRequestEmail(AccountName);

impl RegisterRequestEmail {
    const EMAIL_NO_AT_MESSAGE: &'static str = "Email address must have an \"@\". Try again.\r\n";
}

impl State for RegisterRequestEmail {
    const PREAMBLE: Option<&'static str> = Some("What is your email address?\r\n");

    type Previous = RegisterRequestName;

    fn handle_input_impl(self, input: String, db: &Database) -> HandledBy {
        if !Email::has_at_symbol(&input) {
            return HandledBy { machine: self.into(), action: HandledByAction::OutputMessage(Self::EMAIL_NO_AT_MESSAGE.to_string()), };
        }

        let acct_name = self.0;
        let email = Email(input);
        let recv = Account::insert_account(db, acct_name.clone(), Some(email.clone()));

        HandledBy { machine: RegisterCheckNameEmailUnique(acct_name.clone(), email, recv).into(), action: HandledByAction::InputStateTrans, }
    }

    fn previous(self) -> <Self as State>::Previous {
        RegisterRequestName
    }
}

#[derive(Debug)]
pub(super) struct RegisterCheckNameEmailUnique(AccountName, Email, AccountInsert);

impl State for RegisterCheckNameEmailUnique {
    const WAITING_ON_DB: bool = true;

    type Previous = Self;

    fn handle_input_impl(self, _input: String, _db: &Database) -> HandledBy {
        HandledBy { machine: self.into(), action: HandledByAction::DoNothing, }
    }

    fn handle_db_response(self) -> HandledBy {
        if let Ok(response) = self.2.try_recv() {
            match response {
                Ok(()) => {
                    HandledBy { machine: RegisterRequestPassword(self.0, self.1).into(), action: HandledByAction::InputStateTrans, }
                },

                Err(UniqueAccountError::AcctNameAlreadyExists) => {
                    HandledBy {
                        machine: RegisterRequestName.into(),
                        action: HandledByAction::InputStateTransWithMessage("Account name already exists. Choose another.\r\n".to_string()),
                    }
                },

                Err(UniqueAccountError::EmailAlreadyExists) => {
                    HandledBy {
                        machine: RegisterRequestEmail(self.0).into(),
                        action: HandledByAction::InputStateTransWithMessage("Email already in use. Choose another.\r\n".to_string()),
                    }
                },
            }
        } else {
            HandledBy { machine: self.into(), action: HandledByAction::DoNothing, }
        }
    }

    fn previous(self) -> <Self as State>::Previous {
        self
    }
}

// TODO(Havvy, 2019-12-29, #sec): It's possible to get this far, and then
// to create a new acct with the same name or email and then continue on here.
// The continued registration will fail, probably with a panic.

#[derive(Debug)]
pub(super) struct RegisterRequestPassword(AccountName, Email);

impl State for RegisterRequestPassword {
    const PREAMBLE: Option<&'static str> = Some("What will be your password?\r\n");

    type Previous = RegisterRequestEmail;

    fn handle_input_impl(self, password: String, db: &Database) -> HandledBy {
        let insert = Account::insert_password(db, self.0, password);

        HandledBy { machine: RegisterWaitPasswordInsert(insert).into(), action: HandledByAction::InputStateTrans, }
    }

    fn previous(self) -> <Self as State>::Previous {
        RegisterRequestEmail(self.0)
    }
}

#[derive(Debug)]
pub(super) struct RegisterWaitPasswordInsert(AccountPasswordInsert);

impl State for RegisterWaitPasswordInsert {
    const PREAMBLE: Option<&'static str> = None;
    const WAITING_ON_DB: bool = true;

    type Previous = Self;

    fn handle_input_impl(self, _input: String, _db: &Database) -> HandledBy {
        HandledBy { machine: self.into(), action: HandledByAction::DoNothing, }
    }

    fn handle_db_response(self) -> HandledBy {
        println!("db");
        if let Ok(res) = self.0.try_recv() {
            println!("ok");
            res.expect("Inserting password into database failed.");
            HandledBy { machine: JustConnected.into(), action: HandledByAction::InputStateTransWithMessage("Registration successful!".into()) }
        } else {
            HandledBy { machine: self.into(), action: HandledByAction::DoNothing, }
        }
    }

    fn previous(self) -> Self::Previous {
        self
    }
}

#[derive(Debug)]
pub(super) struct LoginRequestPassword(AccountName);

impl State for LoginRequestPassword {
    const PREAMBLE: Option<&'static str> = Some("What is your password?\r\n");

    type Previous = JustConnected;

    fn handle_input_impl(self, input: String, db: &Database) -> HandledBy {
        todo!()
    }

    fn previous(self) -> <Self as State>::Previous {
        JustConnected
    }
}

#[derive(Debug)]
pub(super) struct Terminal;

// Following https://hoverbear.org/blog/rust-state-machine-pattern/
/// The state of a user when they first connect and try to log in or register.
#[derive(Debug, DeriveFrom)]
pub(super) enum Machine {
    /// Default state.
    JustConnected(JustConnected),

    /// User has requested to register. As for an account name.
    RegisterRequestName(RegisterRequestName),

    /// User has given an account name. Now ask for an email.
    RegisterReqEmail(RegisterRequestEmail),

    /// Checking the database that the account name and email are unique.
    RegisterCheckNameEmailUnique(RegisterCheckNameEmailUnique),

    /// User has given account name and email. Now ask for password.
    RegisterRequestPassword(RegisterRequestPassword),

    /// Wait on database 
    RegisterWaitPasswordInsert(RegisterWaitPasswordInsert),

    /// User has requested to log in with the specified account name.
    LoginRequestPassword(LoginRequestPassword),

    /// Terminal and dummy state.
    Terminal(Terminal),
}

impl Default for Machine {
    fn default() -> Self {
        JustConnected.into()
    }
}

impl Machine {
    pub fn preamble(&self) -> Option<&'static str> {
        match self {
            Machine::JustConnected(_state) => JustConnected::PREAMBLE,
            Machine::RegisterRequestName(_state) => RegisterRequestName::PREAMBLE,
            Machine::RegisterReqEmail(_state) => RegisterRequestEmail::PREAMBLE,
            Machine::RegisterCheckNameEmailUnique(_state) => RegisterCheckNameEmailUnique::PREAMBLE,
            Machine::RegisterRequestPassword(state) => RegisterRequestPassword::PREAMBLE,
            Machine::RegisterWaitPasswordInsert(_state) => RegisterWaitPasswordInsert::PREAMBLE,
            Machine::LoginRequestPassword(_state) => LoginRequestPassword::PREAMBLE,
            Machine::Terminal(_state) => panic!("Methods should not be called on terminal login state!"),
        }
    }

    pub fn waiting_on_db(&self) -> bool {
        match self {
            Machine::JustConnected(_state) => JustConnected::WAITING_ON_DB,
            Machine::RegisterRequestName(_state) => RegisterRequestName::WAITING_ON_DB,
            Machine::RegisterReqEmail(state) => RegisterRequestEmail::WAITING_ON_DB,
            Machine::RegisterCheckNameEmailUnique(state) => RegisterCheckNameEmailUnique::WAITING_ON_DB,
            Machine::RegisterRequestPassword(state) => RegisterRequestPassword::WAITING_ON_DB,
            Machine::RegisterWaitPasswordInsert(_state) => RegisterWaitPasswordInsert::WAITING_ON_DB,
            Machine::LoginRequestPassword(state) => LoginRequestPassword::WAITING_ON_DB,
            Machine::Terminal(state) => panic!("Methods should not be called on terminal login state!"),
        }
    }

    pub fn handle_input(self, input: String, db: &Database) -> HandledBy {
        match self {
            Machine::JustConnected(state) => State::handle_input(state, input, db),
            Machine::RegisterRequestName(state) => State::handle_input(state, input, db),
            Machine::RegisterReqEmail(state) => State::handle_input(state, input, db),
            Machine::RegisterCheckNameEmailUnique(state) => State::handle_input(state, input, db),
            Machine::RegisterRequestPassword(state) => State::handle_input(state, input, db),
            Machine::RegisterWaitPasswordInsert(state) => State::handle_input(state, input, db),
            Machine::LoginRequestPassword(state) => State::handle_input(state, input, db),
            Machine::Terminal(state) => panic!("Methods should not be called on terminal login state!"),
        }
    }

    pub fn handle_db_response(self) -> HandledBy {
        match self {
            Machine::JustConnected(state) => State::handle_db_response(state),
            Machine::RegisterRequestName(state) => State::handle_db_response(state),
            Machine::RegisterReqEmail(state) => State::handle_db_response(state),
            Machine::RegisterCheckNameEmailUnique(state) => State::handle_db_response(state),
            Machine::RegisterRequestPassword(state) => State::handle_db_response(state),
            Machine::RegisterWaitPasswordInsert(state) => State::handle_db_response(state),
            Machine::LoginRequestPassword(state) => State::handle_db_response(state),
            Machine::Terminal(state) => panic!("Methods should not be called on terminal login state!"),
        }
    }
}