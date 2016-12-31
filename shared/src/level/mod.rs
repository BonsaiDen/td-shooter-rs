// STD Dependencies -----------------------------------------------------------
use std::collections::{HashMap, HashSet};


// Modules --------------------------------------------------------------------
mod collision;
pub use self::collision::*;

mod visibility;
pub use self::visibility::*;

mod light_source;
pub use self::light_source::LightSource;

mod wall;
pub use self::wall::*;


// Statics --------------------------------------------------------------------
pub const MAX_LEVEL_SIZE: f32 = 512.0;


// Level Abstraction ----------------------------------------------------------
#[derive(Debug, Default)]
pub struct Level {
    pub walls: Vec<LevelWall>,
    pub lights: Vec<LightSource>,
    pub bounds: [f32; 4],
    collision_grid: HashMap<(isize, isize), Vec<usize>>,
    visibility_grid: HashMap<(isize, isize), HashSet<usize>>,
    light_sources: Vec<LightSource>
}

impl Level {

    pub fn new() -> Level {
        Level {
            walls: Vec::new(),
            lights: Vec::new(),
            bounds: [1000000.0, 1000000.0, -100000.0, -1000000.0],
            collision_grid: HashMap::new(),
            visibility_grid: HashMap::new(),
            light_sources: Vec::new()
        }
    }

    fn add_wall(&mut self, wall: LevelWall) {

        {
            let aabb = &wall.aabb;
            let (top_left, bottom_right) = (
                self.w2g(aabb[0], aabb[1]),
                self.w2g(aabb[2], aabb[3])
            );

            self.bounds[0] = self.bounds[0].min(aabb[0]);
            self.bounds[1] = self.bounds[1].min(aabb[1]);

            self.bounds[2] = self.bounds[2].max(aabb[2]);
            self.bounds[3] = self.bounds[3].max(aabb[3]);

            for y in (top_left.1 - 1)..bottom_right.1 + 1 {
                for x in (top_left.0 - 1)..bottom_right.0 + 1 {
                    self.collision_grid.entry((x, y)).or_insert_with(Vec::new).push(self.walls.len());
                }
            }
        }

        self.walls.push(wall);

    }

    pub fn get_walls_in_bounds(
        &self,
        bounds: &[f32; 4]

    ) -> HashSet<usize> {

        let (top_left, bottom_right) = (
            self.w2g(bounds[0].min(bounds[2]), bounds[1].min(bounds[3])),
            self.w2g(bounds[2].max(bounds[0]), bounds[3].max(bounds[1]))
        );

        let mut walls = HashSet::new();
        for y in (top_left.1 - 1)..bottom_right.1 + 1 {
            for x in (top_left.0 - 1)..bottom_right.0 + 1 {
                if let Some(indicies) = self.collision_grid.get(&(x, y)) {
                    for i in indicies {
                        walls.insert(*i);
                    }
                }
            }
        }

        walls

    }

    pub fn load() -> Level {

        let mut level = Level::new();

        // Left
        level.add_wall(LevelWall::new(-100.0, -100.0, -100.0, 0.0));
        level.add_wall(LevelWall::new(-100.0, 5.0, -100.0, 100.0));

        level.add_wall(LevelWall::new(100.0, 100.0, -100.0, 100.0));
        level.add_wall(LevelWall::new(-50.0, -100.0, -50.0, 0.0));
        level.add_wall(LevelWall::new(0.0, 0.0, 100.0, -100.0));
        level.add_wall(LevelWall::new(0.0, 0.0, 100.0, 100.0));

        level.add_walls_from_rect(&[
            -VISIBILITY_MAX_DISTANCE, -VISIBILITY_MAX_DISTANCE,
            VISIBILITY_MAX_DISTANCE, VISIBILITY_MAX_DISTANCE
        ]);

        level.add_walls_from_rect(&[
            50.0, -10.0,
            60.0, 10.0
        ]);

        level.add_walls_from_rect(&[
            70.0, -10.0,
            80.0, 10.0
        ]);

        level.add_walls_from_rect(&[
            90.0, -10.0,
            100.0, 10.0
        ]);

        level.add_walls_from_rect(&[
            110.0, -10.0,
            120.0, 10.0
        ]);

        level.lights.push(LightSource::new(140.0, 20.0, 50.0));
        level.lights.push(LightSource::new(-120.0, -120.0, 50.0));

        level.pre_calculate_visibility();
        level
    }

    // Internal ---------------------------------------------------------------
    fn add_walls_from_rect(&mut self, bounds: &[f32; 4]) {

        // Top
        self.add_wall(LevelWall::new(bounds[0], bounds[1], bounds[2], bounds[1]));

        // Right
        self.add_wall(LevelWall::new(bounds[0], bounds[1], bounds[0], bounds[3]));

        // Bottom
        self.add_wall(LevelWall::new(bounds[0], bounds[3], bounds[2], bounds[3]));

        // Left
        self.add_wall(LevelWall::new(bounds[2], bounds[1], bounds[2], bounds[3]));

    }

}

