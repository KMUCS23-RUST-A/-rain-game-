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
}

impl Game {
    fn new() -> Self {
        Game {
            score: 0,
            words: VecDeque::new(),
            last_spawn_time: Instant::now(),
            word_len: 0,
        }
    }

    fn update(&mut self, input: Option<char>) -> bool {
        self.spawn_word();
        self.move_words();

        let mut word_completed = false;
        /* 
        for i in 0..self.words.len() {
            let word = &mut self.words[i];
            word.y += 0.1;

            if word.y >= HEIGHT as f32{
                self.score -= word.text.len() as i32;
                self.words.remove(i);
                continue;
            }

            if input.is_some() && input.unwrap_or_default() == word.text.chars().next().unwrap_or_default()&& !word.text.is_empty() {
                word.text.remove(0);

                if word.text.is_empty() {
                    self.score += word.text.len() as i32;
                    word_completed = true;
                }
            }
        }
        */
        for i in (0..self.words.len()).rev() {
            let word = &mut self.words[i];
            word.y += 0.1;
        
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
        
                if word.text.is_empty() { // 수정된 부분: word_completed 계산 방식 변경
                    self.score += self.word_len as i32;
                    word_completed = true;
                }
            }
        }

        word_completed
    }

    fn spawn_word(&mut self) {
        if self.words.len() < MAX_WORDS && self.last_spawn_time.elapsed() > Duration::from_secs(2) {
            let mut rng = rand::thread_rng();
            self.word_len = rng.gen_range(MIN_WORD_LEN, MAX_WORD_LEN + 1); // 수정된 부분
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

    loop {
        let input = getch();

        if input == KEY_F1 {
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
        mvprintw(0 as i32, 0 as i32, score_str.as_str());

        for word in &game.words {
            mvprintw(word.y as i32, word.x as i32, word.text.as_str());
        }
        // 추가된 부분: 입력 프롬프트 문자열을 출력합니다.
        let input_prompt_str = "> ";
        mvprintw(HEIGHT - 1, 0, input_prompt_str);
        refresh();
        napms(100);
    }

    endwin();
}