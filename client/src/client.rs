// STD Dependencies -----------------------------------------------------------
use std::f32::consts;


// External Dependencies ------------------------------------------------------
use clock_ticks;
use piston::input::*;
use graphics::Transformed;

use cobalt;
use cobalt::ConnectionID;
use hexahydrate;


// Internal Dependencies ------------------------------------------------------
use shared::action::Action;
use shared::color::ColorName;
use shared::level::LevelCollision;
use shared::entity::{PlayerInput, PlayerData, PLAYER_RADIUS, PLAYER_BEAM_FIRE_INTERVAL};

use ::level::Level;
use ::camera::Camera;
use ::entity::{Entity, Registry};
use ::particle_system::ParticleSystem;
use ::effect::{Effect, LaserBeam, LaserBeamHit};
use ::renderer::{Circle, CircleArc, Renderer, MAX_PARTICLES};


// Client Implementation ------------------------------------------------------
pub struct Client {

    // Inputs
    buttons: u8,
    input_angle: f32,
    screen_cursor: (f32, f32),
    world_cursor: (f32, f32),

    // Rendering
    camera: Camera,
    effects: Vec<Box<Effect>>,
    particle_system: ParticleSystem,
    debug_draw: bool,

    // Player Colors
    // TODO optimize these?
    player_colors: [[f32; 4]; 2],
    player_data: PlayerData,
    player_circle: Circle,
    player_cone: CircleArc,
    player_last_beam_fire: u64,

    // Network
    actions: Vec<Action>,
    tick: u8
}

impl Client {

    pub fn new(width: u32, height: u32) -> Client {

        Client {

            // Inputs
            buttons: 0,
            input_angle: 0.0,
            screen_cursor: (0.0, 0.0),
            world_cursor: (0.0, 0.0),

            // Rendering
            camera: Camera::new(width, height),
            effects: Vec::new(),
            particle_system: ParticleSystem::new(MAX_PARTICLES),
            debug_draw: false,

            // Colors
            player_colors: [[0f32; 4]; 2],
            player_data: PlayerData::default(),
            player_circle: Circle::new(10, 0.0, 0.0, PLAYER_RADIUS),
            player_cone: CircleArc::new(10, 0.0, 0.0, PLAYER_RADIUS, 0.0, consts::PI * 0.25),
            player_last_beam_fire: 0,

            // Network
            actions: Vec::new(),
            tick: 0

        }
    }

    pub fn input(
        &mut self,
        _: &mut hexahydrate::Client<Entity, ConnectionID, Registry>,
        _: &Level,
        e: &Input
    ) {

        let t = clock_ticks::precise_time_ms();

        if let Some(Button::Mouse(button)) = e.press_args() {
            if button == MouseButton::Left {

                if t >= self.player_last_beam_fire + PLAYER_BEAM_FIRE_INTERVAL {

                    // TODO create laser on server but already play sfx
                    // TODO play laser SFX
                    self.actions.push(Action::FiredLaserBeam(self.tick, self.player_data.r));

                    if self.debug_draw {
                        self.effects.push(Box::new(LaserBeam::from_point(
                            &mut self.particle_system,
                            ColorName::Grey,
                            self.player_data.x,
                            self.player_data.y,
                            self.player_data.r,
                            PLAYER_RADIUS + 0.5,
                            100.0,
                            None
                        )));
                    }

                    self.player_last_beam_fire = t;

                }

            }
        }
        if let Some(value) = e.mouse_scroll_args() {
            if self.debug_draw {
                self.camera.z = (self.camera.z + (value[1] as f32) * 0.1).min(10.0).max(0.0);
            }
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {

            if key == Key::G {
                self.debug_draw = !self.debug_draw;

                self.effects.push(Box::new(LaserBeamHit::from_point(
                    &mut self.particle_system,
                    ColorName::Red,
                    self.player_data.x,
                    self.player_data.y
                )));

            }

            self.buttons |= match key {
                Key::W => 1,
                Key::A => 8,
                Key::S => 4,
                Key::D => 2,
                Key::LShift => 16,
                _ => 0

            } as u8;
        }

        if let Some(Button::Keyboard(key)) = e.release_args() {
            self.buttons &= !(match key {
                Key::W => 1,
                Key::A => 8,
                Key::S => 4,
                Key::D => 2,
                Key::LShift => 16,
                _ => 0

            } as u8);
        }

        e.mouse_cursor(|x, y| {
            self.screen_cursor = (x as f32, y as f32);
        });

    }

    pub fn update(
        &mut self,
        entity_client: &mut hexahydrate::Client<Entity, ConnectionID, Registry>,
        client: &mut cobalt::ClientStream,
        level: &Level,
        dt: f32
    ) {

        let input = PlayerInput::new(
            self.tick,
            self.buttons,
            self.input_angle,
            dt
        );

        // Receive messages
        let mut actions = Vec::new();
        while let Ok(event) = client.receive() {
            match event {
                cobalt::ClientEvent::Connection => {
                    println!("[Client] Now connected to server.");
                },
                cobalt::ClientEvent::Message(packet) => {
                    match entity_client.receive(packet) {
                        Err(hexahydrate::ClientError::InvalidPacketData(bytes)) => {
                            if let Ok(action) = Action::from_bytes(&bytes) {
                                actions.push(action);
                            }
                        },
                        _ => {}
                    }
                },
                cobalt::ClientEvent::ConnectionLost => {
                    println!("[Client] Lost connection to server!");
                    entity_client.reset();
                    client.close().ok();
                },
                cobalt::ClientEvent::ConnectionClosed(_) => {
                    println!("[Client] Closed connection to server.");
                    entity_client.reset();
                    client.close().ok();
                },
                cobalt::ClientEvent::ConnectionFailed => {
                    entity_client.reset();
                    println!("[Client] Failed to connect to server!");
                },
                _ => {}
            }
        }

        // Update entities
        let t = clock_ticks::precise_time_ms();
        entity_client.update_with(|_, entity| {
            if entity.is_local() {
                if entity.is_new() {
                    self.player_colors = entity.colors();
                }
                entity.update_local(level, input.clone());

            } else {
                entity.update_remote(level, t);
            }
        });

        // Apply actions
        for action in actions.drain(0..) {
            println!("[Client] Received action from server: {:?}", action);
            match action {
                Action::CreateLaserBeam(color, x, y, r, l) => {
                    self.effects.push(Box::new(LaserBeam::from_point(
                        &mut self.particle_system,
                        ColorName::from_u8(color),
                        x, y, r,
                        0.0, l,
                        level.collide_beam_wall(x, y, r, l + 1.0)
                    )));
                },
                Action::LaserBeamHit(color, x, y) => {
                    self.effects.push(Box::new(LaserBeamHit::from_point(
                        &mut self.particle_system,
                        ColorName::from_u8(color),
                        x, y
                    )));
                },
                _ => {}
            }
        }

        // Send client inputs to server
        for packet in entity_client.send(512) {
            client.send(cobalt::MessageKind::Instant, packet).ok();
        }

        // Send actions to server
        for action in self.actions.drain(0..) {
            client.send(cobalt::MessageKind::Reliable, action.to_bytes()).ok();
        }

        client.flush().ok();
        self.tick = self.tick.wrapping_add(1);

    }

    pub fn render(
        &mut self,
        renderer: &mut Renderer,
        entity_client: &mut hexahydrate::Client<Entity, ConnectionID, Registry>,
        level: &Level
    ) {

        // Get player positions, colors and visibility
        let (t, u) = (renderer.t(), renderer.u());
        let players = entity_client.map_entities::<(PlayerData, [[f32; 4]; 2], f32), _>(|_, entity| {

            let p = entity.interpolate(u);
            if entity.is_local() {
                self.player_data = p.clone();
                (p, entity.colors(), 1.0)

            } else {
                let visibility = entity.update_visibility(
                    level,
                    &self.player_data,
                    &p,
                    t
                );
                (p, entity.colors(), visibility)
            }

        });

        // Camera setup
        self.camera.center(self.player_data.x, self.player_data.y);
        self.camera.limit(level.bounds());
        self.camera.apply(renderer);

        // Mouse inputs
        self.world_cursor = self.camera.s2w(self.screen_cursor.0, self.screen_cursor.1);
        self.input_angle = (
            self.world_cursor.1 - self.player_data.y

        ).atan2(self.world_cursor.0 - self.player_data.x);

        // Clear
        renderer.clear_stencil(0);
        renderer.clear_color([0.0; 4]);

        // Level Background
        level.render_background(
            renderer ,
            &self.camera,
            self.player_data.x,
            self.player_data.y,
            self.debug_draw
        );

        {
            // Players
            let context = self.camera.context();
            for (p, mut colors, visibility) in players {
                if visibility > 0.0 {

                    colors[0][3] = visibility;
                    colors[1][3] = visibility;

                    let q = context.trans(p.x as f64, p.y as f64).rot_rad(p.r as f64);
                    renderer.set_color([0.0, 0.0, 0.0, 0.5]);
                    self.player_circle.render(renderer, &q.scale(1.1, 1.1));

                    renderer.set_color(colors[0]);
                    self.player_circle.render(renderer, &q);

                    renderer.set_color(colors[1]);
                    self.player_cone.render(renderer, &q);

                }
            }
        }

        // Effects
        for effect in &self.effects {
            effect.render(renderer, &self.camera);
        }

        self.effects.retain(|e| e.alive(t));

        // Particles
        {
            let context = self.camera.context();
            self.particle_system.render(&context.transform, renderer);
        }

        // Lights
        level.render_lights(
            renderer,
            &self.camera,
            self.debug_draw
        );

        // Visibility / shadows
        level.render_shadow(
            renderer,
            &self.camera,
            &self.player_data,
            self.debug_draw
        );

        // Level Walls
        level.render_walls(
            renderer,
            &self.camera,
            self.player_data.x,
            self.player_data.y,
            self.debug_draw
        );

        // HUD
        self.render_hud(renderer);

    }

    pub fn render_hud(&mut self, renderer: &mut Renderer) {

        let context = renderer.context().clone();
        renderer.set_color(self.player_colors[0]);
        renderer.rectangle(&context, &[
            self.screen_cursor.0 - 2.0, self.screen_cursor.1 - 2.0,
            4.0, 4.0
        ]);

        let (w, h) = (renderer.width(), renderer.height());
        renderer.line(&context, &[0.0, 0.0,   w, 0.0], 2.0);
        renderer.line(&context, &[0.0,   h,   w,   h], 2.0);
        renderer.line(&context, &[0.0, 0.0, 0.0,   h], 2.0);
        renderer.line(&context, &[w,   0.0,   w,   h], 2.0);

    }

}

