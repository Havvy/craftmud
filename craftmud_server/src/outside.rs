//! Interfacing with the world outside of the game loop
//! by using tokio.

#![allow(unused)]

use std::sync::Arc;
use std::thread;

use crossbeam_channel::{self as channel, Sender, Receiver};

use tokio::runtime::Handle;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_postgres::{Client, types::ToSql};

use crate::telnet;

type Query = Vec<Box<dyn ToSql + Send + Sync>>;

pub struct Outside {
    pub database: Database,
    pub recv_connection: Receiver<telnet::Connection>,
}

pub fn start_tokio_runtime() -> Outside {
    let (send_connection, recv_connection) = channel::unbounded::<telnet::Connection>();
    let (send_client, recv_client) = channel::unbounded::<Arc<Client>>();

    let mut runtime = tokio::runtime::Builder::new()
    .basic_scheduler()
    .enable_io()
    .build()
    .expect("Unable to build tokio runtime!");

    let handle = runtime.handle().clone();

    thread::spawn(move || {
        runtime.block_on(async {
            let telnet_server = telnet::start_telnet_server(send_connection);
            let database = start_database(send_client);

            futures::join!(telnet_server, database);
        });
    });

    let client = recv_client.recv().expect("Unable to receive database client on startup!");

    Outside { database: Database { handle, client, }, recv_connection, }
}

pub struct Database {
    handle: Handle,
    client: Arc<Client>,
}

// tokio::spawn(async move {
//     let client = std::sync::Arc::new(client);

//     while let Some((query, query_args,)) = recv_query.recv().await {
//         let (send_response, recv_response) = channel::unbounded();
//         send_recv_response.send(recv_response);

//         let client = client.clone();

//         tokio::spawn(async move {
//             let response = client.execute(query, unsafe { std::mem::transmute::<_, &[&_]>(query_args.as_slice()) }).await;
//             send_response.send(response);
//         });
//     }
// });

impl Database {
    pub fn execute(&self,
    query: &'static str,
    params: Query)
    -> Receiver<Result<u64, tokio_postgres::error::Error>> {
        let (sender, recv) = channel::bounded(1);

        // self.handle.clone().spawn(async move {
        //     let execution = {
        //         let params = params.iter_mut().map(|&mut boxed| boxed as Box<dyn ToSql + Sync>).collect::<Vec<_>>();
        //         let params: Vec<&dyn ToSql + Sync> = {
        //             let vec = Vec::with_capacity(params.len());
        //             for mut boxed in params {
        //                 vec.push(&*boxed);
        //             }
        //             vec
        //         };
        //         self.client.execute(query, &*params)
        //     };

        //     sender.send(execution.await);
        // });

        let client = self.client.clone();

        self.handle.clone().spawn(async move {
            // let (send_response, recv_response) = channel::unbounded();
            // send_recv_response.send(recv_response);
    
            tokio::spawn(async move {
                let response = client.execute(query, unsafe { std::mem::transmute::<_, &[&_]>(params.as_slice()) }).await;
                sender.send(response);
            });
        });

        recv
    }
}

async fn start_database(send_client: Sender<Arc<Client>>) {
    let (client, connection) =
    tokio_postgres::connect(crate::db_config::CONFIG, tokio_postgres::NoTls).await.expect("Unable to connect to the database!");

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
            std::process::abort();
        }
    });

    send_client.send(std::sync::Arc::new(client));
}
