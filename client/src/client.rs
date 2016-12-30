// STD Dependencies -----------------------------------------------------------
use std::f64::consts;


// External Dependencies ------------------------------------------------------
use cobalt;
use cobalt::ConnectionID;
use hexahydrate;
use clock_ticks;

use piston::input::*;
use graphics::{Context, Transformed};


// Internal Dependencies ------------------------------------------------------
use shared::action::Action;
use shared::color::ColorName;
use shared::level::LevelCollision;
use shared::entity::{PlayerInput, PlayerPosition, PLAYER_RADIUS};
use ::renderer::Renderer;
use ::entity::{Entity, Registry};
use ::effect::{Effect, LaserBeam};
use ::camera::Camera;
use ::level::Level;


// Client Implementation ------------------------------------------------------
pub struct Client {

    // Inputs
    buttons: u8,
    input_angle: f64,
    screen_cursor: (f64, f64),
    world_cursor: (f64, f64),

    // Rendering
    camera: Camera,
    updates_per_second: u64,
    effects: Vec<Box<Effect>>,
    debug_draw: bool,

    // Player Colors
    player_colors: [[f32; 4]; 2],
    player_position: PlayerPosition,

    // Network
    actions: Vec<Action>,
    tick: u8
}

impl Client {

    pub fn new(updates_per_second: u64, width: f64, height: f64) -> Client {

        Client {

            // Inputs
            buttons: 0,
            input_angle: 0.0,
            screen_cursor: (0.0, 0.0),
            world_cursor: (0.0, 0.0),

            // Rendering
            camera: Camera::new(width, height),
            updates_per_second: updates_per_second,
            effects: Vec::new(),
            debug_draw: false,

            // Colors
            player_colors: [[0f32; 4]; 2],
            player_position: PlayerPosition::default(),

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

        if let Some(Button::Mouse(button)) = e.press_args() {
            // Limit shot rate
            if button == MouseButton::Left {
                self.actions.push(Action::FiredLaserBeam(self.tick, self.player_position.r));

                // TODO create laser on server but already play sfx
                // TODO play laser SFX
                // TODO limit laser firing rate on client and server
                if self.debug_draw {
                    self.effects.push(Box::new(LaserBeam::from_point(
                        ColorName::Grey,
                        self.player_position.x as f64,
                        self.player_position.y as f64,
                        self.player_position.r as f64,
                        PLAYER_RADIUS + 0.5,
                        100.0,
                        400
                    )));
                }

                self.buttons |= 16;

            }
        }

        if let Some(Button::Mouse(button)) = e.release_args() {
            if button == MouseButton::Left {
                self.buttons &= !16;
            }
        }

        if let Some(value) = e.mouse_scroll_args() {
            self.camera.z = (self.camera.z + value[1] * 0.1).min(10.0).max(0.0);
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {

            if key == Key::G {
                self.debug_draw = !self.debug_draw;
            }

            self.buttons |= match key {
                Key::W => 1,
                Key::A => 8,
                Key::S => 4,
                Key::D => 2,
                _ => 0

            } as u8;
        }

        if let Some(Button::Keyboard(key)) = e.release_args() {
            self.buttons &= !(match key {
                Key::W => 1,
                Key::A => 8,
                Key::S => 4,
                Key::D => 2,
                _ => 0

            } as u8);
        }

        e.mouse_cursor(|x, y| {
            self.screen_cursor = (x, y);
        });

    }

    pub fn update(
        &mut self,
        entity_client: &mut hexahydrate::Client<Entity, ConnectionID, Registry>,
        client: &mut cobalt::ClientStream,
        level: &Level,
        dt: f64
    ) {

        let input = PlayerInput::new(
            self.tick, self.buttons, self.input_angle as f32, dt as f32
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
        entity_client.update_with(|_, entity| {
            if entity.is_local() {
                if entity.is_new() {
                    self.player_colors = entity.colors();
                }
                entity.update_local(level, input.clone());

            } else {
                entity.update_remote();
            }
        });

        // Apply actions
        for action in actions.drain(0..) {
            println!("[Client] Received action from server: {:?}", action);
            match action {
                Action::CreateLaserBeam(color, x, y, r, l) => {
                    // TODO have a small sparkle / rotation / start effect at the source of the beam
                    self.effects.push(Box::new(LaserBeam::from_point(
                        ColorName::from_u8(color),
                        x as f64,
                        y as f64,
                        r as f64,
                        0.0,
                        l as f64,
                        400
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
        let t = renderer.get_t();
        let players = entity_client.map_entities::<(PlayerPosition, [[f32; 4]; 2], f64), _>(|_, entity| {

            if entity.is_local() {
                self.player_position = entity.interpolate(renderer.get_u());
                (self.player_position.clone(), entity.colors(), 1.0)

            } else {

                let p = entity.interpolate(renderer.get_u());
                let visibility = entity.update_visibility(
                    self.player_position.x as f64,
                    self.player_position.y as f64,
                    level,
                    &p,
                    t
                );
                (p, entity.colors(), visibility)

            }

        });

        // Camera setup
        self.camera.center(self.player_position.x as f64, self.player_position.y as f64);
        self.camera.limit(level.bounds());
        self.camera.apply(renderer);

        // Mouse inputs
        self.world_cursor = self.camera.s2w(self.screen_cursor.0, self.screen_cursor.1);
        self.input_angle = (
            self.world_cursor.1 - self.player_position.y as f64

        ).atan2(self.world_cursor.0 - self.player_position.x as f64);

        // Clear to black
        renderer.clear([0.0; 4]);

        // Level Background
        level.render_background(
            renderer ,
            &self.camera,
            self.player_position.x as f64,
            self.player_position.y as f64,
            self.debug_draw
        );

        // Players
        for (p, mut colors, visibility) in players {
            if visibility > 0.0 {
                colors[0][3] = visibility as f32;
                colors[1][3] = visibility as f32;
            }
        }

        // Effects
        for effect in &self.effects {
            effect.render(renderer, &self.camera);
        }

        self.effects.retain(|e| e.alive(t));

        // Visibility overlay
        level.render_overlay(
            renderer,
            &self.camera,
            self.player_position.x as f64,
            self.player_position.y as f64,
            self.debug_draw
        );

        // Level Walls
        level.render_walls(
            renderer,
            &self.camera,
            self.player_position.x as f64,
            self.player_position.y as f64,
            self.debug_draw
        );

        // HUD
        self.render_hud(renderer);

    }

    pub fn render_hud(&mut self, renderer: &mut Renderer) {

        /*

        // Cursor marker
        rectangle(
            self.player_colors[0],
            [
                self.world_cursor.0 - 2.0, self.world_cursor.1 - 2.0,
                4.0, 4.0
            ],
            m.transform, g
        );

        // Top / Bottom / Left / Right Border
        let (w, h) = (args.width as f64, args.height as f64);
        line(self.player_colors[0], 2.0, [0.0, 0.0, w, 0.0], c.transform, g);
        line(self.player_colors[0], 2.0, [0.0, h, w, h], c.transform, g);
        line(self.player_colors[0], 2.0, [0.0, 0.0, 0.0, h], c.transform, g);
        line(self.player_colors[0], 2.0, [w, 0.0, w, h], c.transform, g);
        */

    }

    /*
        let m = self.camera.apply(c);

        // Level Background
        level.draw_2d_background(
            m, g, &world_bounds,
            self.player_position.x as f64,
            self.player_position.y as f64,
            PLAYER_RADIUS,
            self.debug_draw
        );

        // Players
        for (p, mut colors, visibility) in players {

            if visibility > 0.0 {

                colors[0][3] = visibility as f32;
                colors[1][3] = visibility as f32;

                // TODO Further optimize circle drawing with pre-generated
                // textures?
                let q = m.trans(p.x as f64, p.y as f64);

                // Outline
                g.tri_list(
                    &DrawState::default(),
                    &[0.0, 0.0, 0.0, 0.5],
                    |f| triangulation::with_arc_tri_list(
                        0.0,
                        consts::PI * 1.999,
                        12,
                        q.transform,
                        player_bounds,
                        PLAYER_RADIUS * 0.65,
                        |vertices| f(vertices)
                    )
                );

                // Body
                let q = q.rot_rad(p.r as f64);
                g.tri_list(
                    &DrawState::default(),
                    &colors[0],
                    |f| triangulation::with_arc_tri_list(
                        0.0,
                        consts::PI * 1.999,
                        12,
                        q.transform,
                        player_bounds,
                        PLAYER_RADIUS * 0.5,
                        |vertices| f(vertices)
                    )
                );

                // Cone of sight
                g.tri_list(
                    &DrawState::default(),
                    &colors[1],
                    |f| triangulation::with_arc_tri_list(
                        -consts::PI * 0.25,
                        -consts::PI * 1.75,
                        12,
                        q.transform,
                        player_bounds,
                        PLAYER_RADIUS * 0.55,
                        |vertices| f(vertices)
                    )
                );

            }

            // Visibility debug lines
            if self.debug_draw {

                let lines = [[
                     p.x as f64 - PLAYER_RADIUS,
                     p.y as f64,
                     self.player_position.x as f64,
                     self.player_position.y as f64

                ], [
                     p.x as f64 + PLAYER_RADIUS,
                     p.y as f64,
                     self.player_position.x as f64,
                     self.player_position.y as f64

                ], [
                     p.x as f64,
                     p.y as f64 - PLAYER_RADIUS,
                     self.player_position.x as f64,
                     self.player_position.y as f64

                ], [
                     p.x as f64,
                     p.y as f64 + PLAYER_RADIUS,
                     self.player_position.x as f64,
                     self.player_position.y as f64
                ]];

                let mut color = self.player_colors[0];
                if visibility > 0.0 && !p.visible {
                    color = [0.0, 1.0, 0.0, 1.0];
                }

                for i in 0..4 {
                    if level.collide_line(&lines[i]).is_none() {
                        line(color, 0.25, lines[i], m.transform, g);
                    }
                }

            }

        }
    */

}

