## User) rust와 ncurses를 사용해서 간단한 한컴타자 소나기 게임을 구현해줘.
* 랜덤으로 선택된 단어가 위에서 동적으로 떨어지고 사용자는 떨어지는 단어를 단어가 땅에 떨어지기 전에 알맞게 타이핑해야함.
* 사용자가 알맞은 단어를 타이핑하면 그 단어는 지워지고 단어의 길이에 비례해 점수가 오름.
* 단어는 한 화면에 여러개 떨어질 수 있음.
* 단어가 바닥에 닿으면 점수가 깎임.
* 코드는 100줄 내외여야 함.
* 실시간으로 사용자의 입력을 받아야함(입력을 비동기적으로 처리해야함)
* 사용자의 입력이 없더라도 단어는 계속 떨어져야함

## GPT) 
```
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
    x: i32,
    y: i32,
    text: String,
}

struct Game {
    score: i32,
    words: VecDeque<Word>,
    last_spawn_time: Instant,
}

impl Game {
    fn new() -> Self {
        Game {
            score: 0,
            words: VecDeque::new(),
            last_spawn_time: Instant::now(),
        }
    }

    fn update(&mut self, input: Option<char>) -> bool {
        self.spawn_word();
        self.move_words();

        let mut word_completed = false;

        for i in 0..self.words.len() {
            let word = &mut self.words[i];
            word.y += 1;

            if word.y >= HEIGHT {
                self.score -= word.text.len() as i32;
                self.words.remove(i);
                continue;
            }

            if input.is_some() && input.unwrap() == word.text.chars().next().unwrap() {
                word.text.remove(0);

                if word.text.is_empty() {
                    self.score += word.text.len() as i32;
                    word_completed = true;
                }
            }
        }

        word_completed
    }

    fn spawn_word(&mut self) {
        if self.words.len() < MAX_WORDS && self.last_spawn_time.elapsed() > Duration::from_secs(2) {
            let mut rng = rand::thread_rng();
            let word_len = rng.gen_range(MIN_WORD_LEN, MAX_WORD_LEN + 1);
            let word_x = rng.gen_range(0, WIDTH - word_len as i32);
            let word_y = 0;
            let word_text: String = (0..word_len)
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
            word.y += 1;
        }
    }
}

fn main() {
    initscr();
    cbreak();
    noecho();
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
        mvprintw(0, 0, score_str.as_str());

        for word in &game.words {
            mvprintw(word.y, word.x, word.text.as_str());
        }
        // 추가된 부분: 입력 프롬프트 문자열을 출력합니다.
        let input_prompt_str = "> ";
        mvprintw(HEIGHT - 1, 0, input_prompt_str);
        refresh();
        napms(100);
    }

    endwin();
}
```
