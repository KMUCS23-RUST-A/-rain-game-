use std::collections::VecDeque;
use std::time::{Duration, Instant};

use rand::Rng;

use ncurses::*;

use super::vocab::VocabGenerator;
use super::word::Word;

const MAX_WORDS: usize = 5;

pub struct Game {
    score: i32,
    words: VecDeque<Word>,
    last_spawn_time: Instant,
    time_limit: Duration,
    elapsed_time: Duration,
    speed_factor: f32,
    height: i32,
    width: i32,
    vocab_generator: VocabGenerator,
    input_string: String,
}

impl Game {
    pub fn new(height: i32, width: i32) -> Self {
        Game {
            score: 0,
            words: VecDeque::new(),
            last_spawn_time: Instant::now(),
            time_limit: Duration::from_secs(60),
            elapsed_time: Duration::default(),
            speed_factor: 0.0,
            height: height,
            width: width,
            vocab_generator: VocabGenerator::new(),
            input_string: String::new(),
        }
    }

    pub fn update(&mut self, input: Option<char>) -> bool {
        self.spawn_word();
        self.move_words();

        let mut word_completed = false;

        if input.is_some() {
            if input.unwrap() == '\n' {
                for i in (0..self.words.len()).rev() {
                    let word = &mut self.words[i];
                    if self.input_string == word.get_text().clone() {
                        self.score += word.get_text().len() as i32;
                        self.words.remove(i);
                        word_completed = true;
                        break;
                    }
                }
                self.input_string = String::new();
            } else {
                self.input_string.push(input.unwrap());
            }
        }

        for i in (0..self.words.len()).rev() {
            let word = &mut self.words[i];
            word.set_y(word.get_y() + self.speed_factor);

            if word.get_y() >= self.height as f32 {
                self.score -= word.get_text().len() as i32;
                self.words.remove(i);
            }
        }

        if self.elapsed_time >= self.time_limit {
            return false;
        }

        self.elapsed_time += Duration::from_millis(100);
        self.speed_factor = 0.1 + self.score as f32 / 1000.0;

        word_completed
    }

    fn spawn_word(&mut self) {
        if self.words.len() < MAX_WORDS && self.last_spawn_time.elapsed() > Duration::from_secs(2) {
            let mut rng = rand::thread_rng();
            let word_text = self.vocab_generator.generate();
            let word_x = rng.gen_range(0.0, self.width as f32 - word_text.len() as f32) as f32;
            let word_y = 0.0;
            self.words.push_back(Word::new(word_x, word_y, word_text));

            self.last_spawn_time = Instant::now();
        }
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

    pub fn get_time_left(&self) -> f32 {
        (self.time_limit - self.elapsed_time).as_secs_f32()
    }

    pub fn is_game_over(&self) -> bool {
        self.elapsed_time >= self.time_limit
    }

    pub fn get_input_string(&self) -> String {
        self.input_string.clone()
    }

    pub fn backspace(&mut self) {
        self.input_string.pop();
    }
}
