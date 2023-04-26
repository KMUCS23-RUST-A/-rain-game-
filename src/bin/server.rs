use tokio::{
    io::{AsyncWriteExt, AsyncReadExt},
    net::{TcpListener, TcpStream},
};

use raingame::Message;

const MAX_CLIENTS: usize = 2;

#[tokio::main]
async fn main() {
    let mut clients = Vec::new();

    let listener = TcpListener::bind("127.0.0.1:12345").await.unwrap();

    // 클라이언트 접속 대기
    for _ in 0..MAX_CLIENTS {
        let (mut socket, _) = listener.accept().await.unwrap();

        let message = Message::Waiting as u8;
        socket.write_all(&[message]).await.unwrap();

        // socket 소유권이 clients vector로 이동
        clients.push(socket);
    }

    // 클라이언트 2명을 각각의 비동기 프로세스로 분리 
    for (index, socket) in clients.into_iter().enumerate() {
        // make clone socket for each client
        tokio::spawn(async move {
            process(socket, index).await;
        });
    }
}

// 클라이언트 메세지 처리 비동기 함수
async fn process(mut socket: TcpStream, client_no: usize) {
    let mut buf = [0; 1];

    // 게임이 시작되었다는 메시지를 클라이언트에게 전송
    let message = Message::GameStart as u8;
    socket.write_all(&[message]).await.unwrap();
    
    // 이후 클라이언트로부터 수신받은 메세지를 종류에 따라 처리
    loop {
        socket.read(&mut buf).await.unwrap();
        let message = Message::from(buf[0]);
        println!("Client {} Message: {:?}", client_no, message);

        match message {
            Message::GameOver => {
                // do something
                break;
            }
            Message::Attacked => {
                // do something
                continue;
            }
            _ => {
                // do something
                println!("Client {} Message: {:?}", client_no, message);
                continue;
            }
        }
    }
    //TODO 상대 클라이언트로부터 넘어오는 메세지 처리
    // - 공격: 대상 클라이언트에게 공격 메세지 forward
    // - 게임 오버: 대상 클라이언트에게 게임 오버 메세지 forward 및 커넥션 종료
}
