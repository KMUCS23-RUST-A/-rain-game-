mod game;

use game::game::Game;
use game::game_state::GameState;

use ncurses::*;

const WIDTH: i32 = 80;
const HEIGHT: i32 = 24;

fn main() {
    initscr();
    cbreak();
    timeout(0);
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    keypad(stdscr(), true);

    let mut game = Game::new(HEIGHT, WIDTH);
    let line = "-".repeat(WIDTH as usize);

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
            {
                game.pop_input_string();
            }
            if input == KEY_ENTER || input == KEY_SEND || input_char.unwrap() == '\n' {
                game.enter_input_string();
            }
        }

        game.update(input_char);
        let game_state = game.get_game_state();

        if game_state == GameState::Lose {
            break;
        };
        if game_state == GameState::TimeOver {
            break;
        };

        addstr(&format!("Score: {}\n", game.get_score()));
        addstr(&format!("Time left: {:.1}s\n", game.get_time_left()));
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
        "TIME OVER!\n"
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
