#[derive(Debug)]
pub struct FileB {
    pub x: i32,
    pub y: i32,
}

impl FileB {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x: x, y: y }
    }
}
