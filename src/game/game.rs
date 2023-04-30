use std::collections::VecDeque;
use std::time::{Duration, Instant};

use rand::Rng;

use ncurses::*;

use super::vocab::VocabGenerator;
use super::word::Word;
use crate::GameState;

const WIDTH: i32 = 80;
const HEIGHT: i32 = 24;

pub struct Game {
    score: i32,
    words: VecDeque<Word>,
    last_spawn_time: Instant,
    speed_factor: f32,
    height: i32,
    width: i32,
    vocab_generator: VocabGenerator,
    input_string: String,
    life: i32,
    game_state: GameState,
    attack_string: String,
    latest_spawned_word: Word,
}

impl Game {
    pub fn new(height: i32, width: i32) -> Self {
        Game {
            score: 0,
            words: VecDeque::new(),
            last_spawn_time: Instant::now(),
            speed_factor: 0.0,
            height: height,
            width: width,
            vocab_generator: VocabGenerator::new(),
            input_string: String::new(),
            life: 5,
            game_state: GameState::StartGame,
            attack_string: String::new(),
            latest_spawned_word: Word::new(0.0, 0.0, String::new()),
        }
    }

    pub fn update(&mut self, input: Option<char>) -> GameState {
        if self.last_spawn_time.elapsed() > Duration::from_secs(2) {
            self.spawn_word();
            self.last_spawn_time = Instant::now();
        }
        self.move_words();

        // 공격 단어 갱신
        if self.attack_string.is_empty() {
            self.attack_string = self.vocab_generator.generate();
        }

        if input.is_some() && input.unwrap() != '\n' && input.unwrap() != '=' {
            self.input_string.push(input.unwrap());
        }

        // 각 단어 별로 Deadline을 넘었는지 판정
        let line_height = (self.height - 2) as f32;
        for i in (0..self.words.len()).rev() {
            let word = &mut self.words[i];
            word.set_y(word.get_y() + self.speed_factor);

            if word.get_y() >= line_height {
                self.score -= word.get_text().len() as i32;
                self.words.remove(i);
                self.life -= 1;
            }
        }

        self.speed_factor = 0.1 + (self.score as f32) / 1000.0;
        self.game_state = if self.life <= 0 {
            GameState::Lose
        } else {
            GameState::InProgress
        };

        self.game_state
    }

    pub fn spawn_word(&mut self) {
        let mut rng = rand::thread_rng();
        let word_text = self.vocab_generator.generate();
        let mut word_x = rng.gen_range(0.0, self.width as f32 - word_text.len() as f32) as f32;
        let latest_x_min = self.latest_spawned_word.get_x() - 1.0;
        let latest_x_max = self.latest_spawned_word.get_x()
            + self.latest_spawned_word.get_text().len() as f32
            + 1.0;
        loop {
            if (latest_x_min < word_x && word_x < latest_x_max)
                || (latest_x_min < (word_x + word_text.len() as f32)
                    && (word_x + word_text.len() as f32) < latest_x_max)
            {
                word_x = rng.gen_range(0.0, self.width as f32 - word_text.len() as f32) as f32;
            } else {
                break;
            }
        }
        let word_y = 0.0;
        self.words
            .push_back(Word::new(word_x, word_y, word_text.clone()));
        self.latest_spawned_word = Word::new(word_x, word_y, word_text.clone());
    }

    pub fn move_words(&mut self) {
        for word in &mut self.words {
            word.set_y(word.get_y() + 0.1);
        }
    }

    pub fn draw_words(&self) {
        for word in &self.words {
            mvprintw(
                word.get_y() as i32,
                word.get_x() as i32,
                word.get_text().as_str(),
            );
        }
    }

    pub fn get_score(&self) -> i32 {
        self.score
    }

    pub fn get_input_string(&self) -> String {
        self.input_string.clone()
    }

    pub fn get_attack_string(&self) -> String {
        self.attack_string.clone()
    }

    pub fn get_life(&self) -> i32 {
        self.life
    }

    pub fn get_game_state(&self) -> GameState {
        self.game_state
    }

    pub fn set_game_state(&mut self, game_state: GameState) {
        self.game_state = game_state;
    }

    pub fn pop_input_string(&mut self) {
        self.input_string.pop();
    }

    pub fn enter_input_string(&mut self) -> GameState {
        for i in (0..self.words.len()).rev() {
            let word = &mut self.words[i];
            if self.input_string.trim() == word.get_text().clone() {
                self.score += word.get_text().len() as i32;
                self.words.remove(i);
                self.game_state = GameState::CompleteWord;
                break;
            }
        }
        if self.input_string == self.attack_string {
            self.score += self.attack_string.len() as i32;
            self.attack_string = self.vocab_generator.generate();
            self.game_state = GameState::CompleteAttackWord;
        }
        self.input_string = String::new();
        self.game_state
    }
}

pub fn play() {
    initscr();
    cbreak();
    timeout(0);
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    keypad(stdscr(), true);

    let mut game = Game::new(HEIGHT, WIDTH);
    let line = "-".repeat(WIDTH as usize);
    let mut game_state = GameState::StartGame;

    loop {
        erase();
        let input = getch();
        let input_char = if (input >= 0) && (input <= 255) {
            char::from_u32(input as u32)
        } else {
            None
        };

        if input_char.is_some() {
            if input == KEY_BACKSPACE
                || input == KEY_DC
                || input == 127
                || input_char.unwrap() == '\u{0008}'
                || input_char.unwrap() == '='
            {
                game.pop_input_string();
            }
            if input == KEY_ENTER || input == KEY_SEND || input_char.unwrap() == '\n' {
                game_state = game.enter_input_string();
            }
        }

        game_state = game.update(input_char); // game_state = InProgress or Lose

        if game_state == GameState::Lose {
            break;
        };

        addstr(&format!("Score: {}\n", game.get_score()));
        game.draw_words();

        // Print input prompt
        let input_prompt = format!("> {}", game.get_input_string());
        let life_string = format!("LIFE: {}", game.get_life());
        let attack_string = format!("ATTACK: {}", game.get_attack_string());

        mvprintw(0, WIDTH - life_string.len() as i32, &life_string);
        mvprintw(1, WIDTH - attack_string.len() as i32, &attack_string);
        mvprintw(HEIGHT - 2, 0, &line);
        mvprintw(HEIGHT - 1, 0, input_prompt.as_str());
        refresh();
        napms(100);
    }

    let game_result_str = if game.get_game_state() == GameState::Lose {
        "YOU LOSE!\n"
    } else {
        "YOU WIN!\n"
    };

    addstr(game_result_str);
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
}
