// External Dependencies ------------------------------------------------------
use graphics::Transformed;


// Internal Dependencies ------------------------------------------------------
use ::camera::Camera;
use ::renderer::{Renderer, CircleArc, Polygon, Line, StencilMode, Texture};
use shared::entity::{PlayerData, PLAYER_VISBILITY_CONE, PLAYER_VISBILITY_CONE_OFFSET};
use shared::collision::aabb_intersect;
use shared::level::{
    Level as SharedLevel,
    LevelCollision,
    LevelVisibility,
    LEVEL_MAX_VISIBILITY_DISTANCE
};


// Modules --------------------------------------------------------------------
mod light_source;
use self::light_source::LightSource;


// Client Level ---------------------------------------------------------------
pub struct Level {
    level: SharedLevel,
    visibility_circle: CircleArc,
    lights: Vec<LightSource>,
    solids: Vec<Polygon>,
    walls: Vec<Line>
}

impl Level {

    pub fn new(level: SharedLevel) -> Level {

        // Generate light visualizations
        let cached_lights = level.lights.iter().map(|l| {
            LightSource::from_light(&level, l)

        }).collect();

        // Generate walls visualizations
        let wall_width = 0.75;
        let cached_walls = level.walls.iter().map(|w| {

            let p = &w.points;

            // Adjust horizonal endpoints to meetup at edges
            let line = if p[0] == p[2] {
                [
                    p[0],
                    p[1] - wall_width * 0.75,
                    p[2],
                    p[3] + wall_width * 0.75,
                ]

            // Adjust vertical endpoints to meetup at edges
            } else if p[1] == p[3] {
                [
                    p[0] - wall_width * 0.75,
                    p[1],
                    p[2] + wall_width * 0.75,
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

        let cached_solids = level.solids.iter().map(|s| {
            Polygon::new(&s)

        }).collect();

        Level {
            level: level,
            visibility_circle: CircleArc::new(
                36, 0.0, 0.0, LEVEL_MAX_VISIBILITY_DISTANCE,
                0.0, PLAYER_VISBILITY_CONE
            ),
            lights: cached_lights,
            solids: cached_solids,
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
        _: u8
    ) {

        // Background layer
        let bounds = self.level.bounds;
        let context = camera.context();
        renderer.set_texture(Texture::Floor(0.023));
        renderer.set_color([0.25, 0.25, 0.25, 1.0]);
        renderer.rectangle(
            context,
            &[
                bounds[0],
                bounds[1],
                bounds[2] - bounds[0],
                bounds[3] - bounds[1]
            ]
        );

        renderer.set_texture(Texture::None);

    }

    pub fn render_lights(
        &self,
        renderer: &mut Renderer,
        camera: &Camera,
        debug_level: u8
    ) {

        // TODO there are potential issues with lights that are very close
        // which might cause the light circle from one light to overlap with
        // the visibility cone of another light

        // Render light clipping visibility cones into stencil
        if debug_level != 3 {
            renderer.set_stencil_mode(StencilMode::Replace(254));
        }
        for light in &self.lights {
            light.render_visibility_stencil(renderer, camera);
        }

        // Render light circles into stencil combining with the cones
        if debug_level != 3 {
            renderer.set_stencil_mode(StencilMode::Add);
        }

        for light in &self.lights {
            light.render_light_stencil(renderer, camera);
        }

        // Render light color circles based on stencil
        let bounds = camera.b2w();
        let s = 1.0 - ((renderer.t() as f32 * 0.0015).cos() * 0.03).abs();
        if debug_level != 3 {
            renderer.set_stencil_mode(StencilMode::InsideLightCircle);
            renderer.set_color([0.95 * s, 0.8, 0.0, 0.025]);
            renderer.rectangle(
                camera.context(),
                &[bounds[0], bounds[1], bounds[2] - bounds[0], bounds[3] - bounds[1]],
            );
        }

        // Render inner light circles
        renderer.set_color([0.95 * s, 0.7, 0.0, 0.05]);
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
        debug_level: u8
    ) {

        let bounds = camera.b2w();
        let context = camera.context();
        let endpoints = self.visibility_polygon(
            data.x, data.y,
            LEVEL_MAX_VISIBILITY_DISTANCE
        );

        // Only render visibility cone if local player is alive
        if data.hp > 0 {

            // Render player visibility cone but only where there
            if debug_level != 4 {
                renderer.set_stencil_mode(StencilMode::ReplaceNonLightCircle);

            } else {
                renderer.set_stencil_mode(StencilMode::None);
                renderer.set_color([1.0, 1.0, 0.0, 0.5]);
            }
            renderer.polygon(&context, &endpoints);

            // Render player visibility circle
            let q = context.trans(data.x as f64, data.y as f64).rot_rad(data.r as f64).trans(
                -PLAYER_VISBILITY_CONE_OFFSET as f64,
                0.0
            );

            if debug_level != 4 {
                renderer.set_stencil_mode(StencilMode::Add);
            }
            self.visibility_circle.render(renderer, &q);

        }

        // Render shadows
        if debug_level != 4 {
            let t = renderer.t();
            renderer.set_stencil_mode(StencilMode::OutsideVisibleArea);
            renderer.set_texture(Texture::Static(t as f32 * 0.0001));
            renderer.set_color([0.0, 0.0, 0.0, 0.85]);
            renderer.rectangle(
                camera.context(),
                &[bounds[0], bounds[1], bounds[2] - bounds[0], bounds[3] - bounds[1]],
            );
            renderer.set_texture(Texture::None);
        }

        renderer.set_stencil_mode(StencilMode::None);

    }

    pub fn render_walls(
        &self,
        renderer: &mut Renderer,
        camera: &Camera,
        _: f32,
        _: f32,
        debug_level: u8
    ) {

        if debug_level == 2 {
            renderer.set_color([1.0, 0.0, 1.0, 1.0]);

        } else {
            renderer.set_color([0.75, 0.75, 0.75, 1.0]);
        }

        let bounds = camera.b2w();
        let context = camera.context();

        let walls = self.level.get_walls_indicies();
        for i in walls {
            let wall = &self.walls[*i];
            if aabb_intersect(&wall.aabb, &bounds) {
                wall.render(renderer, &context);
            }
        }

        // Solids
        renderer.set_texture(Texture::Floor(0.0125));
        for solid in &self.solids {
            if aabb_intersect(&solid.aabb, &bounds) {
                solid.render(renderer, context);
            }
        }
        renderer.set_texture(Texture::None);

    }

}


// Traits ---------------------------------------------------------------------
impl LevelVisibility for Level {

    fn visibility_polygon(&self, x: f32, y: f32, radius: f32) -> Vec<f32> {
        self.level.visibility_polygon(x, y, radius)
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

    fn collide(&self, x: &mut f32, y: &mut f32, radius: f32, active: bool) {
        self.level.collide(x, y, radius, active);
    }

    fn collide_beam(&self, x: f32, y: f32, r: f32, l: f32) -> Option<(usize, [f32; 3])> {
        self.level.collide_beam(x, y, r, l)
    }

    fn collide_beam_wall(&self, x: f32, y: f32, r: f32, l: f32) -> Option<f32> {
        self.level.collide_beam_wall(x, y, r, l)
    }

    fn collide_line(&self, line: &[f32; 4]) -> Option<(usize, [f32; 3])> {
        self.level.collide_line(line)
    }

}

