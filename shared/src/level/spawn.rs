#[derive(Debug, Clone)]
pub struct LevelSpawn {
    pub x: f32,
    pub y: f32
}

impl LevelSpawn {

    pub fn new(x: f32, y: f32) -> LevelSpawn {
        LevelSpawn {
            x: x,
            y: y
        }
    }

}

