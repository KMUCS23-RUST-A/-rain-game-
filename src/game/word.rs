use crate::WordColor;

pub struct Word {
    x: f32,
    y: f32,
    text: String,
    color: WordColor,
}

impl Word {
    pub fn new(x: f32, y: f32, text: String, color: WordColor) -> Self {
        Word { x, y, text, color }
    }

    pub fn get_x(&self) -> f32 {
        self.x
    }

    pub fn get_y(&self) -> f32 {
        self.y
    }

    pub fn set_y(&mut self, y: f32) {
        self.y = y;
    }

    pub fn get_text(&self) -> &String {
        &self.text
    }
}

impl Word {
    pub fn get_color(&self) -> WordColor {
        self.color
    }
}
