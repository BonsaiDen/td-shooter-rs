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
    updates_per_second: u64,
    camera: Camera,
    draw_state: DrawState,
    player_color: ColorName,
    player_position: PlayerPosition,
    effects: Vec<Box<Effect>>,

    // Network
    client: hexahydrate::Client<Entity, ConnectionID, Registry>,
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
            updates_per_second: updates_per_second,
            camera: Camera::new(width, height),
            draw_state: DrawState::default(),
            player_color: ColorName::Black,
            player_position: PlayerPosition::default(),
            effects: Vec::new(),

            // Network
            client: hexahydrate::Client::<Entity, ConnectionID, Registry>::new(Registry, (updates_per_second * 2) as usize),
            actions: Vec::new(),
            tick: 0

        }
    }

    pub fn input(&mut self, e: &Input) {

        if let Some(Button::Mouse(button)) = e.press_args() {
            // Limit shot rate
            if button == MouseButton::Left {
                self.actions.push(Action::FiredLaserBeam(self.tick, self.player_position.r));

                // TODO create laser on server but already play
                // TODO have a small sparkle / rotation / start effect at the source of the beam
                self.effects.push(Box::new(LaserBeam::from_point(
                    ColorName::Grey,
                    self.player_position.x as f64,
                    self.player_position.y as f64,
                    self.player_position.r as f64,
                    PLAYER_RADIUS + 0.5,
                    100.0,
                    400
                )));

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

        let input = PlayerInput::new(self.tick, self.buttons, self.input_angle as f32, dt as f32);

        // Receive messages
        let mut actions = Vec::new();
        while let Ok(event) = client.receive() {
            match event {
                cobalt::ClientEvent::Connection => {
                    println!("[Client] Now connected to server.");
                },
                cobalt::ClientEvent::Message(packet) => {
                    match self.client.receive(packet) {
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
                    self.client.reset();
                    client.close().ok();
                },
                cobalt::ClientEvent::ConnectionClosed(_) => {
                    println!("[Client] Closed connection to server.");
                    self.client.reset();
                    client.close().ok();
                },
                cobalt::ClientEvent::ConnectionFailed => {
                    self.client.reset();
                    println!("[Client] Failed to connect to server!");
                },
                _ => {}
            }
        }

        // Update entities
        self.client.update_with(|_, entity| {
            if entity.is_local() {
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
        for packet in self.client.send(512) {
            client.send(cobalt::MessageKind::Instant, packet).ok();
        }

        // Send actions to server
        for action in self.actions.drain(0..) {
            client.send(cobalt::MessageKind::Reliable, action.to_bytes()).ok();
        }

        client.flush().ok();
        self.tick = self.tick.wrapping_add(1);

    }

    pub fn draw_2d(&mut self, window: &mut PistonWindow, e: &Event, args: &RenderArgs, level: &Level) {

        let t = clock_ticks::precise_time_ms();
        let u = 1.0 / (1.0 / self.updates_per_second as f64) * (args.ext_dt * 1000000000.0);

        // Get player positions and colors
        // TODO optimize
        let (mut p, mut colors) = (PlayerPosition::default(), [[0f32; 4]; 2]);
        let mut color_name = ColorName::Black;
        let players = self.client.map_entities::<(PlayerPosition, [[f32; 4]; 2]), _>(|_, entity| {
            if entity.is_local() {
                p = entity.interpolate(u);
                colors = entity.colors();
                color_name = entity.color_name();
            }
            (entity.interpolate(u), entity.colors())
        });

        self.player_color = color_name;

        // TODO get from update() method instead?
        self.player_position = p;

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

            // Background Box
            rectangle(
                [0.1, 0.1, 0.1, 1.0],
                [-100.0, -100.0, 200.0, 200.0],
                m.transform, g
            );

            // Level
            level.draw_2d(
                m, g, &world_bounds,
                self.player_position.x as f64,
                self.player_position.y as f64,
                PLAYER_RADIUS
            );

            // Players
            for (p, colors) in players {

                // TODO Further optimize circle drawing with pre-generated
                // textures?
                let q = m.trans(p.x as f64, p.y as f64);

                // Outline
                g.tri_list(
                    &self.draw_state,
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
                    &self.draw_state,
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
                    &self.draw_state,
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

