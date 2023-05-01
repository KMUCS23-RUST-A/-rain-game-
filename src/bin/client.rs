use clap::Parser;

use raingame::{Game, Message};
use raingame::{GameState, WordColor};

use chrono::{Utc, TimeZone};
use std::net::TcpListener;
use std::fs;
use std::path::Path;

use std::time::SystemTime;
use std::fs::File;
use std::io::{Write, Result};


use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc::{self, error::TryRecvError, Receiver, Sender},
    task::yield_now,
    fs::OpenOptions,
    
};

use ncurses::*;

const HEIGHT: i32 = 20;
const WIDTH: i32 = 80;

const DEBUG: bool = false;

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
    // 커맨드라인 파싱
    let opts = Opts::parse();
    let addr = format!("{}:{}", opts.host, opts.port);

    // 서버 TCP 연결
    let mut socket = tokio::net::TcpStream::connect(addr)
        .await
        .expect("Server should be running");
    if DEBUG {
        println!("[Client] Connected to server");
    }

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
                if DEBUG {
                    println!("[Client] GOT message from server: {:?}", msg);
                }
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
                if DEBUG {
                    println!("[Client] Server disconnected before game start");
                }
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

    if DEBUG {
        println!("[Client] main exited");
    }
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
                        if DEBUG { println!("[GameManager] GOT message from server: {:?}", srv_msg); }
                        match srv_msg {
                            Message::Attacked => {
                                mgr_writer.send(srv_msg).await.unwrap();   // 게임에게 서버 메세지 전달
                            }
                            Message::GameOver => {
                                mgr_writer.send(srv_msg).await.unwrap();   // 게임에게 서버 메세지 전달
                                break;
                            }
                            _ => {} // 위 메세지 외에는 무시
                        }
                    }
                    Err(_) => {
                        if DEBUG { println!("[GameManager] Server disconnected"); }
                        break;
                    }
                }
            }

            // 게임 메세지를 서버에게 전달
            // Game -(channel)> GameManager -(TCP)> MyClientHandler
            msg = game_reader.recv() => {
                match msg {
                    Some(msg) => {
                        if DEBUG { println!("[GameManager] GOT message from game: {:?}", msg); }
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
                        if DEBUG { println!("[GameManager] Game channel closed"); }
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
    if DEBUG {
        println!("[GameManager] Closed");
    }
}

fn write_game_result(result_string: &str) {
    let now = Utc::now();
    let filename = format!("./log/{}.log", now.format("%Y-%m-%d_%H-%M-%S"));

    let path = Path::new("log");
    if !path.exists() {
        fs::create_dir(path).expect("Failed to create log directory");
    }

    let mut file = File::create(&filename).unwrap();
    file.write_all(result_string.as_bytes()).unwrap();
}

// 게임 쓰레드
async fn spawn_game(game_writer: Sender<Message>, mut mgr_reader: Receiver<Message>) {
    initscr();
    cbreak();
    noecho();
    timeout(0);
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    keypad(stdscr(), true);

    // Color 세팅
    start_color();
    use_default_colors();
    ncurses::init_pair(WordColor::Red as i16, ncurses::COLOR_RED, -1);
    ncurses::init_pair(WordColor::Green as i16, ncurses::COLOR_GREEN, -1);
    ncurses::init_pair(WordColor::Yellow as i16, ncurses::COLOR_YELLOW, -1);

    let mut game = Game::new(HEIGHT, WIDTH);
    let line = "-".repeat(WIDTH as usize);
    let mut game_state = GameState::StartGame;

    loop {
        // GameManager로부터 메세지 non-blocking으로 받기
        match mgr_reader.try_recv() {
            Ok(msg) => {
                if DEBUG {
                    println!("[Game] GOT message from manager: {:?}", msg);
                }
                match msg {
                    Message::GameOver => {
                        game.set_game_state(GameState::Win);
                        break;
                    }
                    Message::Attacked => {
                        game.spawn_word(WordColor::Red);
                    }
                    _ => {} // 위 메세지 타입 외에는 무시
                }
            }
            Err(TryRecvError::Empty) => {} // 읽을 메세지 없음
            Err(TryRecvError::Disconnected) => {
                if DEBUG {
                    println!("[Game] Manager channel closed");
                }
                break;
            }
        }

        erase();

        let input = getch();
        let input_char = char::from_u32(input as u32);
        match (input, input_char) {
            (KEY_BACKSPACE, _)
            | (KEY_DC, _)
            | (127, _)
            | (_, Some('\u{0008}'))
            | (_, Some('='))
            | (_, Some('\x7f')) => {
                game.pop_input_string();
            }
            (KEY_ENTER, _) | (KEY_SEND, _) | (_, Some('\n')) => {
                game_state = game.enter_input_string();
            }
            (_, Some(c)) => {
                game.push_input_string(c);
            }
            _ => {}
        }

        if game_state == GameState::CompleteAttackWord {
            let result = game_writer.send(Message::Attacked).await;
            yield_now().await;
            match result {
                Ok(()) => {
                    if DEBUG {
                        println!("[Game] Sent message to manager: {:?}", Message::Attacked);
                    }
                }
                Err(e) => {
                    if DEBUG {
                        println!("[Game] Failed to send message to manager: {:?}", e);
                    }
                }
            }
        }

        game_state = game.update(); // game_state = InProgress or Lose

        if game_state == GameState::Lose {
            let result = game_writer.send(Message::GameOver).await;
            yield_now().await;
            match result {
                Ok(()) => {
                    if DEBUG {
                        println!("[Game] Sent message to manager: {:?}", Message::GameOver);
                    }
                }
                Err(e) => {
                    if DEBUG {
                        println!("[Game] Failed to send message to manager: {:?}", e);
                    }
                }
            }
            break;
        };

        addstr(&format!("Score: {}\n", game.get_score()));
        game.draw_words();

        // Print input prompt
        let input_prompt = format!("> {}", game.get_input_string());
        let life_string = format!("LIFE: {}", game.get_life());
        let attack_string = format!("ATTACK: {}", game.get_attack_string());

        attron(COLOR_PAIR(WordColor::Green as i16));
        mvprintw(0, WIDTH - life_string.len() as i32, &life_string);
        attroff(COLOR_PAIR(2));

        attron(COLOR_PAIR(WordColor::Red as i16));
        mvprintw(1, WIDTH - attack_string.len() as i32, &attack_string);
        attroff(COLOR_PAIR(1));

        attron(COLOR_PAIR(WordColor::Yellow as i16));
        mvprintw(HEIGHT - 2, 0, &line);
        attroff(COLOR_PAIR(3));

        mvprintw(HEIGHT - 1, 0, input_prompt.as_str());
        refresh();
        yield_now().await;
        napms(100);
    }

    let game_result = match game.get_game_state() {
        GameState::Lose => "YOU LOSE!\n",
        GameState::Win => "YOU WIN!\n",
        _ => "ERROR\n",
    };

    erase();
    addstr(game_result);
    addstr(&format!("Final Score: {}\n", game.get_score()));
    refresh();

    napms(1000);
    addstr("Press any key to exit...");
    refresh();

    loop {
        let input = getch();
        if input > -1 {
            break;
        }
        napms(100);
    }
    endwin();

    // Teardown
    let result_string = format!("{} with a score of {}", game_result, game.get_score());
    write_game_result(&result_string);
    mgr_reader.close();
    while let Some(_) = mgr_reader.recv().await {}

    if DEBUG {
        println!("[Game] Closed");
    }
}
