#![allow(unused)]

use crossbeam_channel::{self as channel};
use legion::prelude::*;

mod models;

mod db_config;
mod login;
mod place;
mod play_state;
mod prompt;
mod telnet;
mod tutorial;
mod output;
mod outside;

// TODO(Havvy, 2019-12-22, #wrong): Parse out whitespace in commands. Take inspiration from Tennu.

pub fn main() {
    // Start Telnet
    // let (send_connection, recv_reconnection) = channel::unbounded::<telnet::Connection>();
    // telnet::start_telnet_server(send_connection);

    // let db = outside::Database {};

    // Start Tokio-driven things.
    let outside::Outside { database, recv_connection } = outside::start_tokio_runtime();

    println!("Tokio-driven systems are go.");

    // Start Legion
    let mut world = &mut World::new();
    let mut resources = &mut Resources::default();
    resources.insert(recv_connection);
    resources.insert(database);

    let (tutorial_realm, tutorial_starting_room) = tutorial::initialize_tutorial(world);
    resources.insert(tutorial_realm);

    // Build Legion Schedule
    let mut schedule = Schedule::builder()
    .add_system(login::add_connection_system())
    .flush()
    .add_system(login::login_system(tutorial_starting_room))
    .add_system(tutorial::tutorial_system())
    .flush()
    .add_system(login::output_system())
    .build();

    // Run the world.
    println!("Run the world.");
    loop {
        std::thread::sleep(std::time::Duration::from_millis(100));
        schedule.execute(world, resources);
    }
}