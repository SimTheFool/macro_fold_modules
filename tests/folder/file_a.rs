#[derive(Debug)]
pub struct FileA {
    pub x: i32,
    pub y: i32,
}

impl FileA {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x: x + 3, y: y + 3 }
    }
}
