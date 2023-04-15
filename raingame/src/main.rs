extern crate rand;
use crate::rand::Rng;

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use ncurses::*;

const MAX_WORD_LEN: usize = 10;
const MIN_WORD_LEN: usize = 3;
const MAX_WORDS: usize = 5;
const WIDTH: i32 = 80;
const HEIGHT: i32 = 24;

struct Word {
    x: f32,
    y: f32,  //단어가 떨어지는 속도를 더 정밀하게 하기 위해 float형으로 수정,
    text: String,
}

struct Game {
    score: i32,
    words: VecDeque<Word>,
    last_spawn_time: Instant,
    word_len: usize,
    time_limit: Duration,
    elapsed_time: Duration,
    speed_factor: f32,  // 추가된 부분
}

impl Game {
    fn new() -> Self {
        Game {
            score: 0,
            words: VecDeque::new(),
            last_spawn_time: Instant::now(),
            word_len: 0,
            time_limit: Duration::from_secs(60),
            elapsed_time: Duration::default(),
            speed_factor: 0.0,  // 추가된 부분
        }
    }

    fn update(&mut self, input: Option<char>) -> bool {
        self.spawn_word();
        self.move_words();

        let mut word_completed = false;

        for i in (0..self.words.len()).rev() {
            let word = &mut self.words[i];
            word.y += self.speed_factor;  // 수정된 부분

            if word.y >= HEIGHT as f32 {
                self.score -= word.text.len() as i32;
                if !self.words.is_empty() {
                    self.words.remove(i);
                }
                continue;
            }

            if input.is_some()
                && input.unwrap_or_default() == word.text.chars().next().unwrap_or_default()
                && !word.text.is_empty()
            {
                word.text.remove(0);

                if word.text.is_empty() {
                    self.score += self.word_len as i32;
                    word_completed = true;
                }
            }
        }

        if self.elapsed_time >= self.time_limit {
            return false;
        }

        self.elapsed_time += Duration::from_millis(100);
        self.speed_factor = 0.1 + self.score as f32 / 1000.0;  // 추가된 부분

        word_completed
    }

    fn spawn_word(&mut self) {
        if self.words.len() < MAX_WORDS && self.last_spawn_time.elapsed() > Duration::from_secs(2) {
            let mut rng = rand::thread_rng();
            self.word_len = rng.gen_range(MIN_WORD_LEN, MAX_WORD_LEN + 1);
            let word_x = rng.gen_range(0.0, WIDTH as f32 - self.word_len as f32) as f32;
            let word_y = 0.0;
            let word_text: String = (0..self.word_len)
                .map(|_| (rng.gen_range(b'a', b'z' + 1) as char))
                .collect();

            self.words.push_back(Word {
                x: word_x,
                y: word_y,
                text: word_text,
            });

            self.last_spawn_time = Instant::now();
        }
    }

    fn move_words(&mut self) {
        for word in &mut self.words {
            word.y += 0.1;
        }
    }
}


fn main() {
    initscr();
    cbreak();
    //noecho();
    timeout(0);
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    keypad(stdscr(), true);

    let mut game = Game::new();
    let start_time = Instant::now();
    let time_limit = Duration::from_secs(60);

    loop {
        let input = getch();

        if input == KEY_F1 || start_time.elapsed() >= time_limit {
            break;
        }

        erase();

        let input_char = if (input >= 0) && (input <= 255) {
            char::from_u32(input as u32)
        } else {
            None
        };
        let word_completed = game.update(input_char);
        if word_completed {
            beep();
        }

        let score_str = format!("Score: {}", game.score);
        let time_left_str = format!("Time left: {:.1}s", (time_limit - start_time.elapsed()).as_secs_f32());
        mvprintw(0 as i32, 0 as i32, score_str.as_str());
        mvprintw(0 as i32, (WIDTH / 2) as i32, time_left_str.as_str());

        for word in &game.words {
            mvprintw(word.y as i32, word.x as i32, word.text.as_str());
        }
        // 추가된 부분: 입력 프롬프트 문자열을 출력합니다.
        let input_prompt_str = "> ";
        mvprintw(HEIGHT - 1, 0, input_prompt_str);
        refresh();
        napms(100);
    }

    let final_score_str = format!("Final Score: {}", game.score);
    mvprintw((HEIGHT / 2) as i32, (WIDTH / 2 - final_score_str.len() as i32 / 2) as i32, final_score_str.as_str());
    mvprintw((HEIGHT / 2 + 1) as i32, (WIDTH / 2 - 5) as i32, "Press any key to exit...");
    refresh();
    getch();
    endwin();
}
