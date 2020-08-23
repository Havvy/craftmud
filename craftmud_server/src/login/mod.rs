//! System and state handling related to logging into CraftMud.

// TODO(Havvy, 2019-12-22, #opt): These methods should take in an &mut MudOutput, not create strings.

use std::io::{Cursor};
use std::ops::DerefMut;

use crossbeam_channel::Receiver;
use legion::prelude::*;

use crate::models::{Account, UniqueAccountError};
use crate::output::{Output, OptionOutputExt};
use crate::outside::Database;
use crate::place::{PlaceId};
use crate::play_state::{PlayState};
use crate::prompt::Prompt;
use crate::telnet::{Connection, OutputSender, InputReceiver};

mod machine;

use machine::{
    HandledBy, HandledByAction, Terminal,
};

use machine::Machine as LoginMachine;

#[derive(Debug, Clone)]
pub struct AccountName(pub String);

impl AccountName {
    /// Whether or not a name is allowed.
    // TODO(Havvy, 2019-12-21) This should probably be done via a db query of some sort placed into a legion Resource.
    fn is_banned(possible_name: &str) -> bool {
        match possible_name {
            "new" |
            "back" | // Hopefully never checked.
            "quit" | // Hopefully never checked.
            "logout" |
            "quitout" |
            "tutorial" | // Hopefully never checked.
            "admin" |
            "help" => true,
            _ => false
        }
    }
}

#[derive(Debug, Clone)]
pub struct Email(pub String);

impl Email {
    fn has_at_symbol(possible_email: &str) -> bool {
        possible_email.contains('@')
    }
}

/// System that handles new connections.
/// 
/// 1. Checks for new connections, and when it has one,
/// 2. Adds Player Archetype entity to the world
/// 3. Sends the on connection message
pub fn add_connection_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("add_connections")
    .write_resource::<Receiver<Connection>>()
    .build(|commands, _world, resources, _query| {
        let recv = resources.deref_mut();

        while let Ok(conn) = recv.try_recv() {
            println!("Setting up new connection");
            let Connection { addr, send_output, recv_input } = conn;
            let login = LoginMachine::default();
            let prompt = Prompt::default();
            let play_state = PlayState::Login;
            let mut output = Output::new();

            output.push_static_paragraph(play_state.preamble().expect("Login play state must have a preamble."));
            output.push_static_paragraph(login.preamble().expect("Default login state must have a preamble."));

            commands.insert((), vec![(addr, send_output, Some(output), recv_input, login, prompt, play_state,)]);
        }
    })
}

pub fn output_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("output")
    .with_query(<(Write<Option<Output>>, Write<OutputSender>, Read<Prompt>)>::query())
    .build(|_commands, world, _resources, query| {
        for (mut output, output_sender, prompt) in query.iter_mut(world) {
            if let Some(mut output) = std::mem::take(&mut *output) {
                output.set_prompt(prompt.to_string());
                output_sender.send(Box::new(Cursor::new(output.to_string())));
            }
        }
    })
}

pub fn login_system(tutorial_starting_room: PlaceId) -> Box<dyn Schedulable> {
    SystemBuilder::new("login")
    .read_resource::<Database>()
    .with_query(<(Write<LoginMachine>, Write<Option<Output>>, Write<InputReceiver>, Write<PlayState>, Read<Prompt>)>::query())
    .build(move |commands, world, db, query| {
        for (entity, (mut login_machine_storage, mut output, input_receiver, mut play_state, prompt,),) in query.iter_entities_mut(world) {
            let login_machine = std::mem::replace(&mut* login_machine_storage, Terminal.into());

            let HandledBy { machine: login_machine, action } = if login_machine.waiting_on_db() {
                // Discard any input while waiting on the database.
                let _ = input_receiver.try_recv();
                login_machine.handle_db_response()
            } else if let Ok(input) = input_receiver.try_recv() {
                login_machine.handle_input(input, db)
            } else {
                HandledBy { machine: login_machine, action: HandledByAction::DoNothing }
            };

            match action {
                HandledByAction::DoNothing => {},

                HandledByAction::InputStateTrans => {
                    if let Some(preamble) = login_machine.preamble() {
                        output.push_static_paragraph(preamble);
                    }
                },

                HandledByAction::InputStateTransWithMessage(message) => {
                    output.push_paragraph(message);
                    if let Some(preamble) = login_machine.preamble() {
                        output.push_static_paragraph(preamble);
                    }
                },

                HandledByAction::OutputMessage(message) => {
                    output.push_paragraph(message);
                },

                HandledByAction::PlayStateTrans(new_play_state) => {
                    if let Some(preamble) = new_play_state.preamble() {
                        output.push_static_paragraph(preamble);
                    }

                    match new_play_state {
                        PlayState::Tutorial => {
                            *play_state = new_play_state;
                            commands.remove_component::<LoginMachine>(entity);
                            commands.add_component(entity, crate::tutorial::Tutorial::new(tutorial_starting_room))
                        },
                        _ => todo!()
                    }
                }
            };

            std::mem::replace(&mut* login_machine_storage, login_machine);
        }
    })
}

pub(crate) fn add_machine(entity: Entity, commands: &mut CommandBuffer) {
    commands.add_component(entity, LoginMachine::default())
}