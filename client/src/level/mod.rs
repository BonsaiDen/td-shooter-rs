// External Dependencies ------------------------------------------------------
use graphics::Transformed;


// Internal Dependencies ------------------------------------------------------
use ::renderer::{Renderer, CircleArc, Line, StencilMode};
use ::camera::Camera;
use shared::entity::{PlayerData, PLAYER_VISBILITY_CONE, PLAYER_VISBILITY_CONE_OFFSET};
use shared::level::{
    Level as SharedLevel,
    LevelCollision,
    LevelVisibility,
    LEVEL_MAX_VISIBILITY_DISTANCE
};


// Modules --------------------------------------------------------------------
mod cached_light_source;
use self::cached_light_source::CachedLightSource;


// Client Level ---------------------------------------------------------------
#[derive(Debug)]
pub struct Level {
    level: SharedLevel,
    visibility_circle: CircleArc,
    lights: Vec<CachedLightSource>,
    walls: Vec<Line>
}

impl Level {

    pub fn new(level: SharedLevel) -> Level {

        let cached_lights = level.lights.iter().map(|l| {
            CachedLightSource::from_light(&level, l)

        }).collect();

        let wall_width = 1.0;
        let cached_walls = level.walls.iter().map(|w| {

            let p = &w.points;

            // Adjust horizonal endpoints to meetup at edges
            let line = if p[0] == p[2] {
                [
                    p[0],
                    p[1] - wall_width * 0.5,
                    p[2],
                    p[3] + wall_width * 0.5,
                ]

            // Adjust vertical endpoints to meetup at edges
            } else if p[1] == p[3] {
                [
                    p[0] - wall_width * 0.5,
                    p[1],
                    p[2] + wall_width * 0.5,
                    p[3]
                ]

            // Diagonal endpoints are left untouched as they will integrate
            // nicely with the rest
            } else {
                [
                    p[0],
                    p[1],
                    p[2],
                    p[3]
                ]
            };

            Line::new(&line, wall_width)

        }).collect();

        Level {
            level: level,
            visibility_circle: CircleArc::new(
                36, 0.0, 0.0, LEVEL_MAX_VISIBILITY_DISTANCE,
                0.0, PLAYER_VISBILITY_CONE
            ),
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

        // TODO there are potential issues with lights that are very close
        // which might cause the light circle from one light to overlap with
        // the visibility cone of another light

        // Render light clipping visibility cones into stencil
        renderer.set_stencil_mode(StencilMode::Replace(254));
        for light in &self.lights {
            light.render_visibility_stencil(renderer, camera);
        }

        // Render light circles into stencil combining with the cones
        renderer.set_stencil_mode(StencilMode::Add);
        for light in &self.lights {
            light.render_light_stencil(renderer, camera);
        }

        // Render light color circles based on stencil
        let bounds = camera.b2w();
        let s = 1.0 - ((renderer.t() as f32 * 0.003).cos() * 0.03).abs();
        renderer.set_stencil_mode(StencilMode::InsideLightCircle);
        renderer.set_color([0.9 * s, 0.7, 0.0, 0.15]);
        renderer.rectangle(
            camera.context(),
            &[bounds[0], bounds[1], bounds[2] - bounds[0], bounds[3] - bounds[1]],
        );

        // Render inner light circles
        renderer.set_color([0.9 * s, 0.6, 0.0, 0.2]);
        for light in &self.lights {
            light.render_light_circle(renderer, camera);
        }

        // Remove all light clipping cones and leave only the clipped light
        // circles in the stencil with a value of 255
        renderer.set_stencil_mode(StencilMode::ClearLightCones);
        renderer.rectangle(
            camera.context(),
            &[bounds[0], bounds[1], bounds[2] - bounds[0], bounds[3] - bounds[1]],
        );

    }

    pub fn render_shadow(
        &self,
        renderer: &mut Renderer,
        camera: &Camera,
        data: &PlayerData,
        _: bool
    ) {

        let bounds = camera.b2w();
        let context = camera.context();
        let endpoints = self.calculate_visibility(
            data.x, data.y,
            LEVEL_MAX_VISIBILITY_DISTANCE
        );

        // Only render visibility cone if local player is alive
        if data.hp > 0 {

            // Render player visibility cone but only where there
            renderer.set_stencil_mode(StencilMode::ReplaceNonLightCircle);
            renderer.light_polygon(&context, data.x, data.y, &endpoints);

            // Render player visibility circle
            let q = context.trans(data.x as f64, data.y as f64).rot_rad(data.r as f64).trans(
                -PLAYER_VISBILITY_CONE_OFFSET as f64,
                0.0
            );
            renderer.set_stencil_mode(StencilMode::Add);
            self.visibility_circle.render(renderer, &q);

        }

        // Render shadows
        renderer.set_stencil_mode(StencilMode::OutsideVisibleArea);
        renderer.set_color([0.0, 0.0, 0.0, 0.75]);
        renderer.rectangle(
            camera.context(),
            &[bounds[0], bounds[1], bounds[2] - bounds[0], bounds[3] - bounds[1]],
        );

        renderer.set_stencil_mode(StencilMode::None);

    }

    pub fn render_walls(
        &self,
        renderer: &mut Renderer,
        camera: &Camera,
        _: f32,
        _: f32,
        _: bool
    ) {

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

    fn calculate_visibility(&self, x: f32, y: f32, radius: f32) -> Vec<(usize, (f32, f32), (f32, f32))> {
        self.level.calculate_visibility(x, y, radius)
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

    fn player_within_visibility(&self, a: &PlayerData, b: &PlayerData) -> bool {
        self.level.player_within_visibility(a, b)
    }

}

impl LevelCollision for Level {

    fn collision_bounds(&self, x: f32, y: f32) -> [f32; 4] {
        self.level.collision_bounds(x, y)
    }

    fn collide(&self, x: &mut f32, y: &mut f32, radius: f32) {
        self.level.collide(x, y, radius);
    }

    fn collide_beam(&self, x: f32, y: f32, r: f32, l: f32) -> Option<(usize, [f32; 3])> {
        self.level.collide_beam(x, y, r, l)
    }

    fn collide_line(&self, line: &[f32; 4]) -> Option<(usize, [f32; 3])> {
        self.level.collide_line(line)
    }

}

