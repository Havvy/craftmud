use std::io::{Cursor};

use legion::prelude::*;

use crate::output::{Output, OptionOutputExt};
use crate::place::{Place, PlaceId, Realm};
use crate::play_state::{PlayState};
use crate::prompt::Prompt;
use crate::telnet::{InputReceiver};

mod machine;
mod messages;

use machine::{HandledBy, Machine,};

pub use messages::PREAMBLE;

pub struct TutorialRealm(Realm);

/// Sets up the tutorial realm, returning the realm and the id to the
/// starting room.
pub fn initialize_tutorial(world: &mut World) -> (TutorialRealm, PlaceId) {
    let mut realm = Realm::new();

    let pid_sr = realm.next_id();
    let pid_nr = realm.next_id();

    let mut starting_room = Place {
        description: messages::ROOM_DESCS[0].into(),
        exits: vec![("forward".into(), pid_nr)],
    };

    let mut next_room = Place {
        description: messages::ROOM_DESCS[1].into(),
        exits: vec![("back".into(), pid_sr)]
    };

    realm.set(pid_sr, starting_room);
    realm.set(pid_nr, next_room);

    (TutorialRealm(realm), pid_sr)
}

pub struct Tutorial {
    data: Data,
    machine: Machine,
}

struct Data {
    place: PlaceId,
}

impl Tutorial {
    pub fn new(starting_room: PlaceId) -> Self {
        Self {
            data: Data { place: starting_room, },
            machine: Machine::new(),
        }
    }
}

pub fn tutorial_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("tutorial")
    .read_resource::<TutorialRealm>()
    .with_query(<(Write<Tutorial>, Write<InputReceiver>, Write<Option<Output>>, Write<PlayState>, Read<Prompt>)>::query())
    .build(|commands, world, realm, query| {
        for (entity, (mut tutorial, input, mut output, mut play_state, prompt,),) in query.iter_entities_mut(world) {
            if let Ok(input) = input.try_recv() {
                let Tutorial {
                    ref mut machine,
                    ref mut data
                } = *tutorial;

                let prev_machine = machine.take();
                let HandledBy{machine: next_machine, action,} = prev_machine.handle_input(input, data, realm);
                machine.untake(next_machine);

                match action {
                    machine::HandledByAction::OutputMessage(message) => {
                        output.push_paragraph(message);
                    },

                    machine::HandledByAction::PlayStateTrans(new_play_state) => {
                        if let Some(preamble) = new_play_state.preamble() {
                            output.push_static_paragraph(preamble);
                        }
    
                        match new_play_state {
                            PlayState::Login => {
                                *play_state = new_play_state;
                                commands.remove_component::<Tutorial>(entity);
                                crate::login::add_machine(entity, commands);
                            },

                            PlayState::Quitting => {
                                todo!()
                            },

                            _ => unreachable!("Can only transition to Login and Quitting from Tutorial")
                        }
                    }

                    _ => todo!()
                }
            }
        }
    })
}