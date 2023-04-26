use tokio::io::AsyncReadExt;

use raingame::Message;

#[tokio::main]
async fn main() {
    // 서버 연결
    let mut socket = tokio::net::TcpStream::connect("127.0.0.1:12345")
        .await
        .unwrap();

    // 게임 시작 대기
    let mut buf = [0; 1];
    loop {
        println!("Waiting for a message...");

        socket.read(&mut buf).await.unwrap();
        let message = Message::from(buf[0]);
        println!("Message: {:?}", message);

        match message {
            Message::GameStart => {
                // do something
                println!("Game started!");
                break;
            }
            _ => {
                // do something
                continue;
            }
        }
    }
    //TODO 게임 시작 시 2가지 태스크 비동기 구성
    // - ncurses를 이용하여 화면을 구성
    // - 사용자 입력을 받아서 단어 처리 및 공격과 게임 오버를 서버로 전송하는 로직 구현
}