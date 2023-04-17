use rand::{thread_rng, Rng};
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct VocabGenerator {
    vocab: Vec<String>,
}

impl VocabGenerator {
    pub fn new() -> Self {
        VocabGenerator {
            vocab: {
                let file = File::open("./src/game/vocab.txt").unwrap();
                let reader = BufReader::new(file);

                let mut vocabs = Vec::<String>::new();
                for line in reader.lines() {
                    if let Ok(vocab) = line {
                        vocabs.push(vocab);
                    }
                }
                vocabs
            },
        }
    }

    pub fn generate(&mut self) -> String {
        let mut rng = thread_rng();
        let random_vocab = self.vocab[rng.gen_range(0, self.vocab.len())].clone();
        random_vocab
    }
}
