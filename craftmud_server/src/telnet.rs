use crossbeam_channel::{self as channel, Sender, Receiver};
use futures::join;
use telnet_server::TelnetListener;
use tokio::io::{
    AsyncRead,
    AsyncReadExt as _,
    AsyncBufReadExt as _,
    AsyncWriteExt as _,
    Error as TokioIoError,
    BufReader,
};

use std::io::{Cursor, Read};

pub type Input = String;
pub type InputReceiver = Receiver<Input>;
pub type Output = Box<dyn AsyncRead + Sync + Send + std::marker::Unpin>;
pub type OutputSender = tokio::sync::mpsc::UnboundedSender<Output>;

pub struct Connection {
    pub addr: std::net::SocketAddr,
    pub send_output: OutputSender,
    pub recv_input: InputReceiver
}

pub async fn start_telnet_server(send_new_connection: Sender<Connection>) -> Result<(), TokioIoError> {
    println!("Starting telnet server");
    let mut telnet = TelnetListener::bind("127.0.0.1:5431").await.unwrap_or_else(|e| { eprintln!("{:?}", e); std::process::abort(); });

    loop {
        let (socket, addr) = telnet.accept().await?;

        let (send_output, mut recv_output) = tokio::sync::mpsc::unbounded_channel::<Output>();
        let (send_input, recv_input) = channel::unbounded::<Input>();

        let new_connection = Connection {
            addr, send_output, recv_input,
        };

        let _ignore_lack_of_recv = send_new_connection.send(new_connection);

        tokio::spawn(async move {
            let (read, mut write) = tokio::io::split(socket);

            let mut read = BufReader::new(read).lines();

            let read_future = tokio::spawn(async move {
                while let Ok(Some(input)) = read.next_line().await {
                    let _ignore_lack_of_recv = send_input.send(input);
                }
            });

            let write_future = tokio::spawn(async move {
                let mut out_buffer = String::with_capacity(128);
                let mut newline = Cursor::new(vec!['\r' as u8, '\n' as u8]);
                while let Some(mut output) = recv_output.recv().await {
                    // (&mut newline).chain(output)
                    // .read_to_string(&mut out_buffer)
                    // .expect("Readers passed to telnet server should never fail to read.");

                    match tokio::io::copy(&mut (output.chain(&mut newline)), &mut write).await {
                        Ok(_) => {
                            // Unread the newline
                            newline.set_position(0);
                            continue;
                        },
                        Err(_err) => { break; }
                    };

                    // match write.write_all(out_buffer.as_bytes()).await {
                    //     Ok(_) => {
                    //         continue;
                    //     }
                    //     Err(_err) => {
                    //         break;
                    //     }
                    // }
                }
            });

            let _ = join!(read_future, write_future);
        });
    }
}