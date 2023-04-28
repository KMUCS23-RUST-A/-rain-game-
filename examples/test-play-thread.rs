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
    let mut socket = tokio::net::TcpStream::connect("127.0.0.1:12345")
        .await
        .unwrap();

    // create channel
    let (mgr_writer, mut mgr_reader) = mpsc::channel(10);
    let (game_writer, mut game_reader) = mpsc::channel(10);

    // create a manager task
    tokio::spawn(async move {
        let mut buf = [0; 1];

        tokio::select! {
            // got message from socket
            srv_msg = socket.read(&mut buf) => {
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
                mgr_writer.send(srv_msg).await.unwrap();
            }

            // got message from game
            msg = game_reader.recv() => {
                println!("GOT GAME MESSAGE: {:?}", msg);
                match msg {
                    Some(msg) => match msg {
                        Message::Waiting => {
                            let message = Message::Waiting as u8;
                            socket.write_all(&[message]).await.unwrap();
                        }
                        Message::GameStart => {
                            let message = Message::GameStart as u8;
                            socket.write_all(&[message]).await.unwrap();
                        }
                        Message::GameOver => {
                            let message = Message::GameOver as u8;
                            socket.write_all(&[message]).await.unwrap();
                        }
                        Message::Attacked => {
                            let message = Message::Attacked as u8;
                            socket.write_all(&[message]).await.unwrap();
                        }
                    },
                    None => {}
                }
            }
        }
    });

    // create a game task
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
            match mgr_reader.try_recv() {
                Ok(msg) => match msg {
                    Message::GameOver => {
                        println!("GOT MANAGER MESSAGE: {:?}", msg);
                        break;
                    }
                    Message::Attacked => {
                        println!("GOT MANAGER MESSAGE: {:?}", msg);
                    }
                    _ => {} // ignore Message::GameStart and Message::Waiting
                },
                Err(TryRecvError::Empty) => {} // no message from manager
                Err(TryRecvError::Disconnected) => break, // manager closed
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
                game_writer.send(Message::Attacked).await.unwrap();
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
