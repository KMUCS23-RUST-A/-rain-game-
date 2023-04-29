use raingame::{Game, Message};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc::{self, error::TryRecvError, Receiver, Sender}, task::yield_now,
};

use ncurses::*;

const HEIGHT: i32 = 20;
const WIDTH: i32 = 80;

#[tokio::main]
async fn main() {
    // 서버 TCP 연결
    let mut socket = tokio::net::TcpStream::connect("127.0.0.1:12345")
        .await
        .expect("Server should be running");
    println!("[Client] Connected to server");

    // 채널 생성`
    let (mgr_writer, mgr_reader) = mpsc::channel::<Message>(10);
    let (game_writer, game_reader) = mpsc::channel::<Message>(10);

    // 상대 클라이언트 대기 및 게임 시작 메세지 수신
    let mut buf = [0; 1];
    loop {
        let nbytes = socket.read_exact(&mut buf).await;
        match nbytes {
            Ok(_) => {
                let msg = Message::from(buf[0]);
                println!("[Client] GOT message from server: {:?}", msg);
                match msg {
                    Message::Waiting => {
                        continue;
                    }
                    Message::GameStart => {
                        break;
                    }
                    _ => {} // 위 메세지 타입 외에는 무시
                }
            }
            Err(_) => {
                println!("[Client] Server disconnected before game start");
                return;
            }
        }
    }

    // GameManager 쓰레드
    let mgr_handle = tokio::spawn(async move {
        spawn_manager(socket, mgr_writer, game_reader).await;
    });

    // Game 쓰레드
    let game_handle = tokio::spawn(async move {
        spawn_game(game_writer, mgr_reader).await;
    });

    // 쓰레드 종료 대기
    mgr_handle.await.unwrap();
    game_handle.await.unwrap();

    println!("[Client] main exited");
}

// GameManager 쓰레드
async fn spawn_manager(
    mut socket: TcpStream,
    mgr_writer: Sender<Message>,
    mut game_reader: Receiver<Message>,
) {
    let mut buf = [0; 1];

    // I/O Multiplexing
    loop {
        tokio::select! {

            // 서버 메세지를 게임에게 전달 상대 클라이언트에게 전달
            // MyClientHandler -(TCP)> GameManager -(channel)> Game
            nbytes = socket.read_exact(&mut buf) => {
                match nbytes {
                    Ok(_) => {
                        let srv_msg = Message::from(buf[0]);
                        println!("[GameManager] GOT message from server: {:?}", srv_msg);
                        match srv_msg {
                            Message::Attacked => {
                                mgr_writer.send(srv_msg).await.unwrap();   // 게임에게 서버 메세지 전달
                            }
                            Message::GameOver => {
                                break;
                            }
                            _ => {} // 위 메세지 외에는 무시
                        }
                    }
                    Err(_) => {
                        println!("[GameManager] Server disconnected");
                        break;
                    }
                }
            }

            // 게임 메세지를 서버에게 전달
            // Game -(channel)> GameManager -(TCP)> MyClientHandler
            msg = game_reader.recv() => {
                match msg {
                    Some(msg) => {
                        println!("[GameManager] GOT message from game: {:?}", msg);
                        match msg {
                            Message::Attacked => {
                                socket.write_all(&[msg as u8]).await.unwrap();
                            }
                            Message::GameOver => {
                                socket.write_all(&[msg as u8]).await.unwrap();
                                break;  // GameManager 종료
                            }
                            _ => {} // 위 메세지 외에는 무시
                        }
                    }
                    None => {
                        println!("[GameManager] Game channel closed");
                        break;  // GameManager 종료
                    }
                }
            }
        }
    }

    // Tear down
    game_reader.close();
    while let Some(_) = game_reader.recv().await {}

    socket.shutdown().await.unwrap();
    println!("[GameManager] Closed");
}

// 게임 쓰레드
async fn spawn_game(game_writer: Sender<Message>, mut mgr_reader: Receiver<Message>) {
    // run game
    initscr();
    cbreak();
    timeout(0);
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    keypad(stdscr(), true);

    let mut game = Game::new(HEIGHT, WIDTH);
    let line = "-".repeat(WIDTH as usize);

    loop {
        // GameManager로부터 메세지 non-blocking으로 받기
        match mgr_reader.try_recv() {
            Ok(msg) => {
                println!("[Game] GOT message from manager: {:?}", msg);
                match msg {
                    Message::GameOver => {
                        break;
                    }
                    Message::Attacked => {
                        // TODO: 공격 단어 수신 시 처리
                    }
                    _ => {} // 위 메세지 타입 외에는 무시
                }
            }
            Err(TryRecvError::Empty) => {} // 읽을 메세지 없음
            Err(TryRecvError::Disconnected) => {
                println!("[Game] Manager channel closed");
                break;
            }
        }

        let input = getch();

        if input == KEY_F1 || game.is_game_over() {
            break;
        }
        if input == KEY_BACKSPACE {
            game.backspace();
        }
        erase();

        let input_char = if (input >= 0) && (input <= 255) {
            char::from_u32(input as u32)
        } else {
            None
        };

        // TODO: 업데이트 및 단어 완성에 대한 이벤트 매니저에게 전송 (channel)

        let word_completed = game.update(input_char);
        if word_completed {
            // Game에서 GameManager로 메세지 전달
            let result = game_writer.send(Message::Attacked).await;
            match result {
                Ok(()) => {
                    println!("[Game] Sent message to manager: {:?}", Message::Attacked);
                }
                Err(e) => {
                    println!("[Game] Failed to send message to manager: {:?}", e);
                }
            }
        }

        addstr(&format!("Score: {}\n", game.get_score()));
        addstr(&format!("Time left: {:.1}s\n", game.get_time_left()));
        game.draw_words();

        // Print input prompt
        let input_prompt = format!("> {}", game.get_input_string());

        mvprintw(HEIGHT - 2, 0, &line);
        mvprintw(HEIGHT - 1, 0, input_prompt.as_str());
        refresh();
        yield_now().await;
        napms(100);
    }

    addstr(&format!("Final Score: {}\n", game.get_score()));
    addstr("Press any key to exit...");
    refresh();
    getch();
    endwin();

    // Teardown
    match game_writer.send(Message::GameOver).await {
        Ok(()) => {
            println!("[Game] Sent message to manager: {:?}", Message::GameOver);
        }
        Err(e) => {
            // 상대가 먼저 GameOver를 호출하여 channel이 닫힌 경우
            println!("[Game] manager channel already closed: {:?}", e);
        }
    }

    mgr_reader.close();
    while let Some(_) = mgr_reader.recv().await {}

    println!("[Game] Closed");
}
