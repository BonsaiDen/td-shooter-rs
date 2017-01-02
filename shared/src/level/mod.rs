// STD Dependencies -----------------------------------------------------------
use std::collections::{HashMap, HashSet};


// External Dependencies ------------------------------------------------------
use toml;
use rand;
use rand::Rng;


// Modules --------------------------------------------------------------------
mod collision;
pub use self::collision::*;

mod visibility;
pub use self::visibility::*;

mod light_source;
pub use self::light_source::LightSource;

mod wall;
pub use self::wall::*;

mod spawn;
pub use self::spawn::*;


// Statics --------------------------------------------------------------------
pub const MAX_LEVEL_SIZE: f32 = 512.0;


// Level Abstraction ----------------------------------------------------------
#[derive(Debug, Default)]
pub struct Level {
    pub walls: Vec<LevelWall>,
    pub lights: Vec<LightSource>,
    pub spawns: Vec<LevelSpawn>,
    pub bounds: [f32; 4],
    pub solids: Vec<Vec<[f32; 2]>>,
    collision_grid: HashMap<(isize, isize), Vec<usize>>,
    visibility_grid: HashMap<(isize, isize), HashSet<usize>>,
    light_sources: Vec<LightSource>
}

impl Level {

    pub fn new() -> Level {
        Level {
            walls: Vec::new(),
            lights: Vec::new(),
            spawns: vec![LevelSpawn::new(0.0, 0.0)],
            solids: Vec::new(),
            bounds: [1000000.0, 1000000.0, -100000.0, -1000000.0],
            collision_grid: HashMap::new(),
            visibility_grid: HashMap::new(),
            light_sources: Vec::new()
        }
    }

    pub fn randomized_spawns(&self) -> Vec<LevelSpawn> {
        let mut spawns = self.spawns.clone();
        rand::thread_rng().shuffle(&mut spawns);
        spawns
    }

    pub fn from_toml(string: &str) -> Level {

        let mut level = Level::new();
        if let Some(value) = toml::Parser::new(string).parse() {

            // Load Walls
            if let Some(&toml::Value::Array(ref walls)) = value.get("walls") {
                for wall in walls {
                    if let &toml::Value::Table(ref properties) = wall {
                        if let Some(&toml::Value::Array(ref points)) = properties.get("line") {
                            level.add_wall(LevelWall::new(
                                points[0].as_float().unwrap() as f32,
                                points[1].as_float().unwrap() as f32,
                                points[2].as_float().unwrap() as f32,
                                points[3].as_float().unwrap() as f32
                            ));
                        }
                    }
                }
            }

            // Load Lights
            if let Some(&toml::Value::Array(ref lights)) = value.get("lights") {
                for light in lights {
                    if let &toml::Value::Table(ref properties) = light {
                        level.lights.push(LightSource::new(
                            properties.get("x").unwrap().as_integer().unwrap() as f32,
                            properties.get("y").unwrap().as_integer().unwrap() as f32,
                            properties.get("radius").unwrap().as_integer().unwrap() as f32
                        ));
                    }
                }
            }

            // Load Spawns
            if let Some(&toml::Value::Array(ref spawns)) = value.get("spawns") {

                if !spawns.is_empty() {
                    level.spawns.clear();
                }

                for spawn in spawns {
                    if let &toml::Value::Table(ref properties) = spawn {
                        level.spawns.push(LevelSpawn::new(
                            properties.get("x").unwrap().as_integer().unwrap() as f32,
                            properties.get("y").unwrap().as_integer().unwrap() as f32
                        ));
                    }
                }

            }

            // Load solids
            if let Some(&toml::Value::Array(ref solids)) = value.get("solids") {
                // TODO create spatial index for these and allow querying to avoid drawing them all at
                // once
                for solid in solids {
                    if let &toml::Value::Array(ref points) = solid {
                        let points: Vec<f32> = points.into_iter().map(|p| p.as_integer().unwrap() as f32).collect();
                        let mut pairs = Vec::new();
                        for i in 0..points.len() / 2 {
                            pairs.push([points[i * 2], points[i * 2 + 1]]);
                        }
                        level.solids.push(pairs);
                    }
                }
            }

        }

        level.pre_calculate_visibility();
        level

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
        let data = include_str!("../../../editor/map.toml");
        Level::from_toml(data)
    }

}

