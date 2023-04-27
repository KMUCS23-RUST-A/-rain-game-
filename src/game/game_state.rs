#[derive(Copy, Clone, PartialEq)]
pub enum GameState {
    StartGame = 0,
    Complete = 1,
    CompleteAttack = 2,
    Lose = 3,
    TimeOver = 4,
}
