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