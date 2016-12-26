// STD Dependencies -----------------------------------------------------------
use std::f64::consts;


// External Dependencies ------------------------------------------------------
use piston_window::*;
use hexahydrate;
use cobalt;
use cobalt::ConnectionID;


// Internal Dependencies ------------------------------------------------------
use shared::entity::{PlayerInput, PlayerPosition, PLAYER_RADIUS};
use ::entity::{Entity, Registry};
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
    updates_per_second: u64,
    draw_state: DrawState,
    camera: Camera,

    // Network
    client: hexahydrate::Client<Entity, ConnectionID, Registry>,
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
            updates_per_second: updates_per_second,
            draw_state: DrawState::default(),
            camera: Camera::new(width, height),

            // Network
            client: hexahydrate::Client::<Entity, ConnectionID, Registry>::new(Registry, (updates_per_second * 2) as usize),
            tick: 0

        }
    }

    pub fn input(&mut self, e: &Input) {

        if let Some(Button::Mouse(button)) = e.press_args() {
            println!("Pressed mouse button '{:?}'", button);
        }

        if let Some(Button::Mouse(button)) = e.release_args() {
            println!("Released mouse button '{:?}'", button);
        }

        if let Some(value) = e.mouse_scroll_args() {
            self.camera.z = (self.camera.z + value[1] * 0.1).min(10.0).max(0.0);
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
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

    pub fn update(&mut self, dt: f64, level: &Level, client: &mut cobalt::ClientStream) {

        while let Ok(event) = client.receive() {
            match event {
                cobalt::ClientEvent::Connection => {
                    println!("Now connected to server.");
                },
                cobalt::ClientEvent::Message(packet) => {
                    self.client.receive(packet).expect("Invalid packet received.");
                },
                cobalt::ClientEvent::ConnectionLost => {
                    println!("Lost connection to server!");
                    self.client.reset();
                    client.close().ok();
                },
                cobalt::ClientEvent::ConnectionClosed(_) => {
                    println!("Closed connection to server.");
                    self.client.reset();
                    client.close().ok();
                },
                cobalt::ClientEvent::ConnectionFailed => {
                    self.client.reset();
                    println!("Failed to connect to server!");
                },
                _ => {}
            }
        }

        // Update entities
        let input = PlayerInput::new(self.tick, self.buttons, self.input_angle as f32, dt as f32);
        self.client.update_with(|_, entity| {
            if entity.is_local() {
                entity.update_local(level, input.clone());

            } else {
                entity.update_remote();
            }
        });

        // Send client inputs to server
        for packet in self.client.send(512) {
            client.send(cobalt::MessageKind::Instant, packet).ok();
        }

        client.flush().ok();
        self.tick = self.tick.wrapping_add(1);

    }

    pub fn draw_2d(&mut self, window: &mut PistonWindow, e: &Event, args: &RenderArgs, level: &Level) {

        let u = 1.0 / (1.0 / self.updates_per_second as f64) * (args.ext_dt * 1000000000.0);

        // Get player positions and colors
        let (mut p, mut colors) = (PlayerPosition::default(), [[0f32; 4]; 2]);
        let players = self.client.map_entities::<(PlayerPosition, [[f32; 4]; 2]), _>(|_, entity| {
            if entity.is_local() {
                p = entity.interpolate(u);
                colors = entity.colors();
            }
            (entity.interpolate(u), entity.colors())
        });

        // Camera setup
        self.camera.x = (p.x as f64).max(-200.0).min(200.0);
        self.camera.y = (p.y as f64).max(-200.0).min(200.0);
        self.camera.update(args);

        // Mouse inputs
        self.world_cursor = self.camera.s2w(self.screen_cursor);
        self.input_angle = (self.world_cursor.1 - p.y as f64).atan2(self.world_cursor.0 - p.x as f64);

        let outline_radius = PLAYER_RADIUS + 0.5;
        window.draw_2d(e, |c, g| {

            let m = self.camera.apply(c);

            // Clear to black
            clear([0.0; 4], g);

            // Background Box
            rectangle(
                [0.1, 0.1, 0.1, 1.0],
                [-100.0, -100.0, 200.0, 200.0],
                m.transform, g
            );

            // Level
            level.draw_2d(m, g, p.x as f64, p.y as f64, PLAYER_RADIUS);

            // Players
            let bounds = [
                -PLAYER_RADIUS * 0.5,
                -PLAYER_RADIUS * 0.5,
                PLAYER_RADIUS, PLAYER_RADIUS

            ].into();

            for (p, colors) in players {

                // TODO optimize circle drawing with pre calculated triangles
                let q = m.trans(p.x as f64, p.y as f64).rot_rad(p.r as f64);

                // Outline
                g.tri_list(
                    &self.draw_state,
                    &[0.0, 0.0, 0.0, 0.5],
                    |f| triangulation::with_arc_tri_list(
                        0.0,
                        consts::PI * 1.999,
                        12,
                        q.transform,
                        bounds,
                        PLAYER_RADIUS * 0.65,
                        |vertices| f(vertices)
                    )
                );

                // Body
                g.tri_list(
                    &self.draw_state,
                    &colors[0],
                    |f| triangulation::with_arc_tri_list(
                        0.0,
                        consts::PI * 1.999,
                        12,
                        q.transform,
                        bounds,
                        PLAYER_RADIUS * 0.5,
                        |vertices| f(vertices)
                    )
                );

                // Cone of sight
                g.tri_list(
                    &self.draw_state,
                    &colors[1],
                    |f| triangulation::with_arc_tri_list(
                        -consts::PI * 0.25,
                        -consts::PI * 1.75,
                        12,
                        q.transform,
                        bounds,
                        PLAYER_RADIUS * 0.55,
                        |vertices| f(vertices)
                    )
                );

            }

            // Cursor marker
            rectangle(
                colors[0],
                [
                    self.world_cursor.0 - 2.0, self.world_cursor.1 - 2.0,
                    4.0, 4.0
                ],
                m.transform, g
            );

            // Top / Bottom / Left / Right Border
            let (w, h) = (args.width as f64, args.height as f64);
            line(colors[0], 2.0, [0.0, 0.0, w, 0.0], c.transform, g);
            line(colors[0], 2.0, [0.0, h, w, h], c.transform, g);
            line(colors[0], 2.0, [0.0, 0.0, 0.0, h], c.transform, g);
            line(colors[0], 2.0, [w, 0.0, w, h], c.transform, g);

        });

    }

}

