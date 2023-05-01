use clap::Parser;

use raingame::Message;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::mpsc::{self, Receiver, Sender},
};


const MAX_CLIENTS: usize = 2;

#[derive(Parser, Debug)]
struct Opts {
    // Address of the server to connect to
    #[arg(short = 'a', long, default_value = "0.0.0.0")]
    host: String,

    // Port of the server to connect to
    #[arg(short, long, default_value = "22345")]
    port: String,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    loop {
        println!("[Server] Setting up a new game...");

        let mut client_sockets = Vec::new();
        let mut client_handler = Vec::new();

        // 핸들러 채널 생성
        let (client1_writer, client1_reader) = mpsc::channel::<Message>(8);
        let (client2_writer, client2_reader) = mpsc::channel::<Message>(8);

        let channelsets = vec![
            (client1_writer, client2_reader),
            (client2_writer, client1_reader),
        ];

        // 서버 소켓 생성
        let addr = format!("{}:{}", opts.host, opts.port);
        let listener = TcpListener::bind(addr).await.unwrap();

        // 클라이언트 접속 대기
        println!("[Server] Waiting for clients...");
        for _ in 0..MAX_CLIENTS {
            let (mut socket, _) = listener.accept().await.unwrap(); // 클라이언트 연결 대기
            println!(
                "[Server] Client connected from {}",
                socket.peer_addr().unwrap()
            );

            let msg = Message::Waiting;
            socket.write_all(&[msg as u8]).await.unwrap(); // 클라이언트에게 상대방 접속 대기

            // socket 소유권이 clients vector로 이동
            client_sockets.push(socket);
        }

        if client_sockets.len() == MAX_CLIENTS {
            println!("[Server] All clients connected");

            // 클라이언트 핸들러를 각각의 쓰레드로 분리
            for (index, (socket, channelset)) in client_sockets
                .into_iter()
                .zip(channelsets.into_iter())
                .enumerate()
            {
                let handle = tokio::spawn(async move {
                    handler(socket, index + 1, channelset).await;
                });
                client_handler.push(handle);
            }

            // 클라이언트 핸들러 쓰레드 종료 대기
            for handler in client_handler {
                handler.await.unwrap();
            }
        }

        println!("[Server] Game Finished!")
    }
}

// 클라이언트 핸들러
async fn handler(
    mut socket: TcpStream,
    client_no: usize,
    channel: (Sender<Message>, Receiver<Message>),
) {
    let mut buf = [0; 1];
    let (opponent_writer, mut my_reader) = channel;

    // 게임 시작 메세지를 각 클라이언트에게 전송
    let msg = Message::GameStart;
    socket
        .write_all(&[msg as u8])
        .await
        .expect("Client should be connected");

    println!(
        "[Server] [Client{} Handler] SENT Message::GameStart to Client{}",
        client_no, client_no
    );

    loop {
        tokio::select! {
            // 내 클라이언트에서 발생한 메세지를 상대 클라이언트에게 전달
            // MyClient -(TCP)> MyClientHandler -(channel)> OpponentClientHandler
            nbytes = socket.read_exact(&mut buf) => {
                match nbytes {
                    Ok(_) => {
                        let client_msg = Message::from(buf[0]);
                        println!("[Server] [Client{} Handler] RELAY message from Client{} to opponent client handler: {:?}", client_no, client_no, client_msg);

                        match client_msg {
                            Message::Attacked => {
                                opponent_writer.send(client_msg).await
                                    .expect("Opponent channel should be opened");
                            }
                            Message::GameOver => {
                                opponent_writer.send(client_msg).await
                                    .expect("Opponent channel should be opened");

                                break;  // 클라이언트 핸들러 종료
                            }
                            _ => {} // 위 메세지 타입 외에는 무시
                        }
                    }
                    Err(e) => {
                        println!("[Server] [Client{} Handler] GOT read error from CLIENT{}: {:?}", client_no, client_no, e);
                        break;  // 클라이언트 핸들러 종료
                    }
                }
            }

            // 상대 클라이언트에서 발생한 메세지를 내 클라이언트에게 전달
            // OpponentClientHandler -(channel)> MyClientHandler -(TCP)> MyClient
            msg = my_reader.recv() => {
                match msg {
                    Some(msg) => {
                        println!("[Server] [Client{} Handler] GOT message from opponent client handler: {:?}", client_no, msg);
                        match msg {
                            Message::Attacked => {
                                socket.write_all(&[msg as u8]).await
                                    .expect("Opponent TcpStream should be opened");
                            }
                            Message::GameOver => {
                                socket.write_all(&[msg as u8]).await
                                    .expect("Opponent TcpStream should be opened");
                                break;  // 클라이언트 핸들러 종료
                            }
                            _ => {} // 위 메세지 타입 외에는 무시
                        }
                    }
                    None => {
                        println!("[Server] [Client{} Handler] Opponent client handler channel closed", client_no);
                        break;  // 클라이언트 핸들러 종료
                    }
                }
            }
        }
    }

    // Teardown

    // tokio channel clean shutdown
    // see https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html#clean-shutdown
    my_reader.close();
    while let Some(_) = my_reader.recv().await {}

    // TcpStream shutdown
    socket.shutdown().await.unwrap();

    println!("[Server] [Client{} Handler] Closed", client_no);
}
