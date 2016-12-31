// Internal Dependencies ------------------------------------------------------
use ::renderer::{Renderer, Line, StencilMode};
use ::camera::Camera;
use shared::level::{
    Level as SharedLevel,
    LevelCollision,
    LevelVisibility
};


// Modules --------------------------------------------------------------------
mod cached_light_source;
use self::cached_light_source::CachedLightSource;


// Client Level ---------------------------------------------------------------
#[derive(Debug, Default)]
pub struct Level {
    level: SharedLevel,
    lights: Vec<CachedLightSource>,
    walls: Vec<Line>
}

impl Level {

    pub fn new(level: SharedLevel) -> Level {

        let cached_lights = level.lights.iter().map(|l| {
            CachedLightSource::from_light(&level, l)

        }).collect();

        let cached_walls = level.walls.iter().map(|w| {
            Line::new(&w.points, 1.0)

        }).collect();

        Level {
            level: level,
            lights: cached_lights,
            walls: cached_walls
        }

    }

    pub fn bounds(&self) -> &[f32; 4] {
        &self.level.bounds
    }

    pub fn render_background(
        &self,
        renderer: &mut Renderer,
        camera: &Camera,
        _: f32,
        _: f32,
        _: bool
    ) {
        let bounds = self.level.bounds;
        renderer.set_color([0.3, 0.3, 0.3, 1.0]);
        renderer.rectangle(
            camera.context(),
            &[bounds[0], bounds[1], bounds[2] - bounds[0], bounds[3] - bounds[1]],
        );
    }

    pub fn render_lights(
        &self,
        renderer: &mut Renderer,
        camera: &Camera,
        _: bool
    ) {

        // TODO fix random crack lines
        // TODO pre-render stencil value into a buffer in order to speed up
        // rendering

        // Render light visibility cones into stencil
        renderer.set_stencil_mode(StencilMode::Replace(254));
        for light in &self.lights {
            light.render_visibility_stencil(renderer, camera);
        }

        // Render light cirlces into stencil combining with the cones
        renderer.set_stencil_mode(StencilMode::Add(1));
        for light in &self.lights {
            light.render_light_stencil(renderer, camera);
        }

        // Render light color circles based on stencil
        let bounds = camera.b2w();
        let s = 1.0 - ((renderer.t() as f32 * 0.003).cos() * 0.03).abs();
        renderer.set_stencil_mode(StencilMode::Inside(255));
        renderer.set_color([0.9 * s, 0.7, 0.0, 0.2]);
        renderer.rectangle(
            camera.context(),
            &[bounds[0], bounds[1], bounds[2] - bounds[0], bounds[3] - bounds[1]],
        );

    }

    pub fn render_shadow(
        &self,
        renderer: &mut Renderer,
        camera: &Camera,
        x: f32,
        y: f32,
        _: bool
    ) {

        let bounds = camera.b2w();
        let context = camera.context();
        let endpoints = self.calculate_visibility(x, y);

        // Render player visibility cone
        renderer.set_stencil_mode(StencilMode::Replace(255));
        renderer.light_polygon(&context, x, y, &endpoints);

        // Render shadows
        renderer.set_stencil_mode(StencilMode::Outside(255));
        renderer.set_color([0.0, 0.0, 0.0, 0.75]);
        renderer.rectangle(
            camera.context(),
            &[bounds[0], bounds[1], bounds[2] - bounds[0], bounds[3] - bounds[1]],
        );

    }

    pub fn render_walls(
        &self,
        renderer: &mut Renderer,
        camera: &Camera,
        _: f32,
        _: f32,
        _: bool
    ) {

        renderer.set_stencil_mode(StencilMode::None);
        renderer.set_color([0.8, 0.8, 0.8, 1.0]);

        let bounds = camera.b2w();
        let context = camera.context();
        let walls = self.level.get_walls_in_bounds(&bounds);

        for i in &walls {
            let wall = &self.walls[*i];
            wall.render(renderer, &context);
        }

    }

}


// Traits ---------------------------------------------------------------------
impl LevelVisibility for Level {

    fn calculate_visibility(&self, x: f32, y: f32) -> Vec<(usize, (f32, f32), (f32, f32))> {
        self.level.calculate_visibility(x, y)
    }

    fn visibility_bounds(&self, x: f32, y: f32) -> [f32; 4] {
        self.level.visibility_bounds(x, y)
    }

    fn circle_visible_from(&self, cx: f32, cy: f32, radius: f32, x: f32, y: f32) -> bool {
        self.level.circle_visible_from(cx, cy, radius, x, y)
    }

    fn circle_in_light(&self, x: f32, y: f32, radius: f32) -> bool {
        self.level.circle_in_light(x, y, radius)
    }

}

impl LevelCollision for Level {

    fn collision_bounds(&self, x: f32, y: f32) -> [f32; 4] {
        self.level.collision_bounds(x, y)
    }

    fn collide(&self, x: &mut f32, y: &mut f32, radius: f32) {
        self.level.collide(x, y, radius);
    }

    fn collide_beam(&self, x: f32, y: f32, r: f32, l: f32) -> Option<[f32; 5]> {
        self.level.collide_beam(x, y, r, l)
    }

    fn collide_line(&self, line: &[f32; 4]) -> Option<[f32; 5]> {
        self.level.collide_line(line)
    }

}

