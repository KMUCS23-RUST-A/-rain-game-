use raingame::{Game, Message};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::mpsc::{self, error::TryRecvError},
};

use ncurses::*;

const HEIGHT: i32 = 20;
const WIDTH: i32 = 80;

#[tokio::main]
async fn main() {
    // 서버 TCP 연결
    let mut socket = tokio::net::TcpStream::connect("127.0.0.1:12345")
        .await
        .unwrap();

    // 채널 생성
    let (mgr_writer, mut mgr_reader) = mpsc::channel(10);
    let (game_writer, mut game_reader) = mpsc::channel(10);

    // GameManager 쓰레드
    tokio::spawn(async move {
        let mut buf = [0; 1];

        // I/O Multiplexing
        tokio::select! {

            // 서버에서 메세지 1개(read_exact) 읽기
            srv_msg = socket.read_exact(&mut buf) => {
                let srv_msg = Message::from(srv_msg.unwrap() as u8);
                println!("GOT SERVER MESSAGE: {:?}", srv_msg);
                match srv_msg {
                    Message::Waiting => {
                        println!("waiting");
                    }
                    Message::GameStart => {
                        println!("game start");
                    }
                    Message::Attacked => {
                        println!("attacked");
                    }
                    Message::GameOver => {
                        println!("game over");
                    }
                }
                game_writer.send(srv_msg).await.unwrap();   // 게임에게 서버 메세지 전달
            }

            // 게임 메세지 1개 읽기
            msg = game_reader.recv() => {
                println!("GOT GAME MESSAGE: {:?}", msg);
                let msg = msg.unwrap();
                match msg {
                    Message::Attacked | Message::GameOver => {
                        socket.write_all(&[msg as u8]).await.unwrap();  // 서버에게 게임 메세지 전달
                    }
                    _ => {} // 위 메세지 외에는 무시
                }
            }
        }
    });

    // Ncurses 게임 실행 쓰레드
    tokio::spawn(async move {
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
                Ok(msg) => match msg {
                    Message::GameOver => {
                        println!("GOT MANAGER MESSAGE: {:?}", msg);
                        break;
                    }
                    Message::Attacked => {
                        println!("GOT MANAGER MESSAGE: {:?}", msg);
                    }
                    _ => {} // 위 메세지 외에는 무시
                },
                Err(TryRecvError::Empty) => {} // 읽을 메세지 없음
                Err(TryRecvError::Disconnected) => break, // GameManager 닫힘
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

            let word_completed = game.update(input_char);
            if word_completed {
                mgr_writer.send(Message::Attacked).await.unwrap();  // GameManager에게 게임 메세지 전달
            }

            addstr(&format!("Score: {}\n", game.get_score()));
            addstr(&format!("Time left: {:.1}s\n", game.get_time_left()));
            game.draw_words();

            // Print input prompt
            let input_prompt = format!("> {}", game.get_input_string());

            mvprintw(HEIGHT - 2, 0, &line);
            mvprintw(HEIGHT - 1, 0, input_prompt.as_str());
            refresh();
            napms(100);
        }

        addstr(&format!("Final Score: {}\n", game.get_score()));
        addstr("Press any key to exit...");
        refresh();
        getch();
        endwin();
    });
}
