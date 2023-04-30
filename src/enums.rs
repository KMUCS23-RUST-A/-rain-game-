#[derive(Clone, Copy)]
pub enum WordColor {
    White = 0,
    Black = 1,
    Yellow = 2,
    Red = 3,
    Green = 4,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GameState {
    StartGame = 0,
    CompleteWord = 1,
    CompleteAttackWord = 2,
    Lose = 3,
    InProgress = 4,
    Win = 5,
}

#[derive(Debug)]
pub enum Message {
    Waiting = 0,
    GameStart = 1,
    GameOver = 2,
    Attacked = 3,
}

impl From<u8> for Message {
    fn from(byte: u8) -> Self {
        match byte {
            0 => Message::Waiting,
            1 => Message::GameStart,
            2 => Message::GameOver,
            3 => Message::Attacked,
            _ => panic!("unexpected message"),
        }
    }
}
