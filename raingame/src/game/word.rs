pub struct Word {
    x: f32,
    y: f32,
    text: String,
}

impl Word {
    pub fn new(x: f32, y: f32, text: String) -> Self {
        Word { x, y, text }
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

    pub fn get_text_mut(&mut self) -> &mut String {
        &mut self.text
    }
}
