// STD Dependencies -----------------------------------------------------------
use std::f64::consts;


// External Dependencies ------------------------------------------------------
use piston_window::*;
use clock_ticks;
use hexahydrate;
use cobalt;
use cobalt::ConnectionID;


// Internal Dependencies ------------------------------------------------------
use shared::action::Action;
use shared::color::ColorName;
use shared::entity::{PlayerInput, PlayerPosition, PLAYER_RADIUS};
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
                // TODO have a small sparkle / rotation / start effect at the source of the beam
                // TODO create local effect on next render frame to have a better synchronization
                // with the server
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

    pub fn draw_2d(
        &mut self,
        entity_client: &mut hexahydrate::Client<Entity, ConnectionID, Registry>,
        level: &Level,
        window: &mut PistonWindow,
        e: &Event,
        args: &RenderArgs,
    ) {

        let t = clock_ticks::precise_time_ms();
        let u = 1.0 / (1.0 / self.updates_per_second as f64) * (args.ext_dt * 1000000000.0);

        // Get player positions and colors
        let players = entity_client.map_entities::<(PlayerPosition, [[f32; 4]; 2]), _>(|_, entity| {
            if entity.is_local() {
                self.player_position = entity.interpolate(u);
            }
            (entity.interpolate(u), entity.colors())
        });

        // Camera setup
        self.camera.x = (self.player_position.x as f64).max(-200.0).min(200.0);
        self.camera.y = (self.player_position.y as f64).max(-200.0).min(200.0);
        self.camera.update(args);

        // Mouse inputs
        self.world_cursor = self.camera.s2w(self.screen_cursor.0, self.screen_cursor.1);
        self.input_angle = (self.world_cursor.1 - self.player_position.y as f64).atan2(self.world_cursor.0 - self.player_position.x as f64);

        // Bounding Rects
        let world_bounds = self.camera.b2w();
        let player_bounds = [
            -PLAYER_RADIUS * 0.5,
            -PLAYER_RADIUS * 0.5,
            PLAYER_RADIUS, PLAYER_RADIUS

        ].into();

        window.draw_2d(e, |c, g| {

            let m = self.camera.apply(c);

            // Clear to black
            clear([0.0; 4], g);

            // Level
            level.draw_2d(
                m, g, &world_bounds,
                self.player_position.x as f64,
                self.player_position.y as f64,
                PLAYER_RADIUS,
                self.debug_draw
            );

            // Players
            for (p, colors) in players {

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

            // Effects
            for effect in &self.effects {
                effect.draw_2d(m, g, t);
            }

            self.effects.retain(|e| e.alive(t));

            // Visibility overlay
            level.draw_2d_overlay(
                m, g, &world_bounds,
                self.player_position.x as f64,
                self.player_position.y as f64,
                self.debug_draw
            );

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

        });

    }

}

