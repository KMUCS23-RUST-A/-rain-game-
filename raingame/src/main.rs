use crossterm::{cursor, event, execute, style::Print, terminal, Result};
use rand::Rng;
use std::io::{stdout, Write};
use std::time::{Duration, Instant};

fn main() -> Result<()> {
    let mut rng = rand::thread_rng();
    let mut score = 0;
    let mut incorrect_score = 0;
    let mut correct_word = "".to_string();
    let mut words = vec![generate_word(&mut rng)];
    let mut level = 1;
    let mut last_tick = Instant::now();
    let mut game_over = false;

    // 터미널 초기화
    terminal::enable_raw_mode()?;
    execute!(stdout(), terminal::Clear(terminal::ClearType::All))?;
    execute!(stdout(), cursor::Hide)?;

    loop {
        let elapsed = last_tick.elapsed();
        last_tick += elapsed;

        // 동일한 속도로 단어들이 떨어지기 위해 경과된 시간을 이용
        if elapsed >= Duration::from_millis((1000 / level as u64).saturating_sub(score)) {
            words.push(generate_word(&mut rng));
        }

        // 화면에 출력
        execute!(stdout(), cursor::MoveTo(0, 0), Print(format!("Score: {}", score)))?;
        execute!(stdout(), cursor::MoveTo(0, 1), Print(format!("Level: {}", level)))?;

        for (i, word) in words.iter().enumerate() {
            let y = (i + 3) as u16; // Score, Level 이후 위치
            execute!(stdout(), cursor::MoveTo(0, y), Print("   "))?;
            execute!(
                stdout(),
                cursor::MoveTo(0, y),
                Print(word.chars().map(|c| if c == ' ' { ' ' } else { '_' }).collect::<String>())
            )?;
        }

        if let Some(key_event) = event::poll(Duration::from_millis(10))? {
            if let event::Event::Key(key) = key_event {
                if !game_over && key.code == event::KeyCode::Backspace && !correct_word.is_empty() {
                    correct_word.pop();
                } else if !game_over && key.code == event::KeyCode::Char(c) {
                    correct_word.push(c);
                } else if key.code == event::KeyCode::Char('q') {
                    break;
                }
            }
        }

        let correct = words.iter_mut().any(|word| {
            if let Some(idx) = word.find(correct_word.as_str()) {
                word.remove(idx);
                return true;
            }
            false
        });

        let corrected = correct_word.len() > 0 && !correct;
        let incorrect = incorrect_score > 0 && incorrect_score % 5 == 0;
        incorrect_score += if corrected { correct_word.len() } else { 0 };
        correct_word.clear();

        // 점수 계산
        if corrected {
            score += level * 2;
            level = (score / 50) + 1;
            incorrect_score /= 2;
        }
        if incorrect {
            score -= 10;
        }
        if words.iter().any(|word| word.is_empty()) {
            score -= 20;
            game_over = true;
            execute!(stdout(), cursor::MoveTo(0, 2), Print("You lose!"))?;
        }

        // 최종 출력
        execute!(stdout(), cursor::Hide)?;
        execute!(stdou t(), terminal::Clear(terminal::ClearType::All))?;
        stdout().flush()?;
        if game_over {
            return Ok(());
        }
    }
}

fn generate_word(rng: &mut rand::RngCore) -> String {
    let words = vec![
        "apple", "banana", "cherry", "durian", "elderberry", "fig", "grape", "honey", "kiwi",
        "lemon", "melon", "orange", "pear", "quince", "raspberry", "strawberry", "tangerine",
        "watermelon",
    ];

    words[rng.gen_range(0..words.len())].into()
}