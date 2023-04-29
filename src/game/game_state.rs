#[derive(Copy, Clone, PartialEq)]
pub enum GameState {
    StartGame = 0,
    CompleteWord = 1,
    CompleteAttackWord = 2,
    Lose = 3,
    InProgress = 4,
}
