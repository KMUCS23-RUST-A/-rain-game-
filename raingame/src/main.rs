mod game;

use game::game::Game;

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

    loop {
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
            beep();
        }

        addstr(&format!("Score: {}\n", game.get_score()));
        addstr(&format!("Time left: {:.1}s\n", game.get_time_left()));
        game.draw_words();

        // Print input prompt
        let input_prompt = format!("> {}", game.get_input_string());

        mvprintw(HEIGHT - 1, 0, input_prompt.as_str());
        refresh();
        napms(100);
    }

    addstr(&format!("Final Score: {}\n", game.get_score()));
    addstr("Press any key to exit...");
    refresh();
    getch();
    endwin();
}
