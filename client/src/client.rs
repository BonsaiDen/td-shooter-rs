// STD Dependencies -----------------------------------------------------------
use std::io::Read;
use std::f32::consts;


// External Dependencies ------------------------------------------------------
use hyper;
use clock_ticks;
use piston::input::*;
use graphics::Transformed;

use cobalt;
use cobalt::ConnectionID;
use hexahydrate;


// Internal Dependencies ------------------------------------------------------
use ::Timer;
use ::level::Level;
use ::camera::Camera;
use ::entity::{Entity, Registry};
use ::effect::{Effect, LaserBeam, LaserBeamHit, ScreenFlash, ParticleSystem};
use ::renderer::{Circle, CircleArc, Renderer, MAX_PARTICLES};

use shared::action::Action;
use shared::color::ColorName;
use shared::level::{Level as SharedLevel, LevelCollision};
use shared::entity::{
    PlayerInput, PlayerData,
    PLAYER_RADIUS, PLAYER_BEAM_FIRE_INTERVAL, PLAYER_MAX_HP
};


// Client Implementation ------------------------------------------------------
pub struct Client {

    // Inputs
    buttons: u8,
    input_angle: f32,
    screen_cursor: (f32, f32),
    world_cursor: (f32, f32),

    // Rendering
    camera: Camera,
    player: LocalPlayerData,
    effects: Vec<Box<Effect>>,
    screen_effects: Vec<Box<Effect>>,
    particle_system: ParticleSystem,
    debug_level: u8,

    // Network
    tick: u8,
    ready: bool,
    addr: String,
    actions: Vec<Action>
}

impl Client {

    pub fn new(_: &mut Renderer, addr: &str, width: u32, height: u32) -> Client {

        Client {

            // Inputs
            buttons: 0,
            input_angle: 0.0,
            screen_cursor: (0.0, 0.0),
            world_cursor: (0.0, 0.0),

            // Rendering
            camera: Camera::new(width, height),
            player: LocalPlayerData::new(),
            effects: Vec::new(),
            screen_effects: Vec::new(),
            particle_system: ParticleSystem::new(MAX_PARTICLES),
            debug_level: 0,

            // Network
            tick: 0,
            ready: false,
            addr: addr.to_string(),
            actions: Vec::new()

        }
    }

    pub fn input(
        &mut self,
        _: &mut Timer,
        renderer: &mut Renderer,
        _: &mut hexahydrate::Client<Entity, ConnectionID, Registry>,
        _: &Level,
        e: &Input
    ) {

        let t = clock_ticks::precise_time_ms();

        if let Some(Button::Mouse(button)) = e.press_args() {
            if button == MouseButton::Left {

                if t >= self.player.last_beam_fire + PLAYER_BEAM_FIRE_INTERVAL {

                    // TODO play laser SFX
                    self.actions.push(Action::FiredLaserBeam(self.tick, self.player.data.r));

                    if self.debug_level == 1 {
                        self.effects.push(Box::new(LaserBeam::from_point(
                            &mut self.particle_system,
                            ColorName::Grey,
                            self.player.data.x,
                            self.player.data.y,
                            self.player.data.r,
                            PLAYER_RADIUS + 0.5,
                            100.0,
                            None
                        )));
                    }

                    self.player.last_beam_fire = t;

                }

            }
        }

        if let Some(value) = e.mouse_scroll_args() {
            if self.debug_level > 0 {
                self.camera.z = (self.camera.z + (value[1] as f32) * 0.1).min(10.0).max(0.0);
            }
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {

            if key == Key::G {
                self.debug_level += 1;
                if self.debug_level == 5 {
                    self.debug_level = 0;
                }
            }

            if key == Key::P {
                let enabled = renderer.wireframe();
                renderer.set_wireframe(!enabled);
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

    pub fn reset(
        &mut self,
        entity_client: &mut hexahydrate::Client<Entity, ConnectionID, Registry>,
        client: &mut cobalt::ClientStream,
    ) {
        self.ready = false;
        self.player.data.hp = 0;
        entity_client.reset();
        client.close().ok();
    }

    pub fn update(
        &mut self,
        timer: &mut Timer,
        entity_client: &mut hexahydrate::Client<Entity, ConnectionID, Registry>,
        client: &mut cobalt::ClientStream,
        level: &mut Level,
        dt: f32
    ) {

        // Receive messages
        let mut actions = Vec::new();
        while let Ok(event) = client.receive() {
            match event {
                cobalt::ClientEvent::Connection => {
                    println!("[Client] Now connected to server.");

                    if let Ok(toml) = download_map(self.addr.as_str()) {
                        println!("[Client] Map downloaded");
                        level.load(SharedLevel::from_toml_string(toml.as_str()));
                        self.actions.push(Action::JoinGame);
                        self.ready = true;

                    } else {
                        println!("[Client] Map download failed!");
                    }
                },
                cobalt::ClientEvent::Message(packet) => {
                    match entity_client.receive(packet) {
                        Err(hexahydrate::ClientError::InvalidPacketData(bytes)) => {
                            if let Ok(action) = Action::from_bytes(&bytes) {
                                actions.push(action);
                            }
                        },
                        _ => {

                        }
                    }
                },
                cobalt::ClientEvent::ConnectionLost => {
                    println!("[Client] Connection to server was lost!");
                    self.reset(entity_client, client);
                    client.connect(self.addr.as_str()).ok();
                },
                cobalt::ClientEvent::ConnectionClosed(_) => {
                    println!("[Client] Connection to server was closed.");
                    self.reset(entity_client, client);
                    client.connect(self.addr.as_str()).ok();
                },
                cobalt::ClientEvent::ConnectionFailed => {
                    println!("[Client] Failed to connect to server!");
                    self.reset(entity_client, client);
                    timer.schedule(|client, _, network, _| {
                        println!("[Client] Trying to reconnect...");
                        network.connect(client.addr.as_str()).ok();

                    }, 1000);
                },
                _ => {}
            }
        }

        // Prevent any updates if not ready to play
        if self.ready {
            self.update_connected(timer, entity_client, client, level, dt, actions);
        }

        // Send actions to server
        for action in self.actions.drain(0..) {
            client.send(cobalt::MessageKind::Reliable, action.to_bytes()).ok();
        }

        client.flush().ok();
        self.tick = self.tick.wrapping_add(1);

    }

    fn update_connected(
        &mut self,
        _: &mut Timer,
        entity_client: &mut hexahydrate::Client<Entity, ConnectionID, Registry>,
        client: &mut cobalt::ClientStream,
        level: &Level,
        dt: f32,
        mut actions: Vec<Action>
    ) {

        // Ignore inputs when player is currently dead
        let input = if self.player.data.hp == 0 {
            PlayerInput::new(self.tick, 0, self.input_angle, dt)

        } else {
            PlayerInput::new(self.tick, self.buttons, self.input_angle, dt)
        };

        // Update entities
        let t = clock_ticks::precise_time_ms();
        entity_client.update_with(|_, entity| {
            if entity.is_local() {
                if entity.is_new() {
                    self.player.colors = entity.colors();
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

                Action::LaserBeamHit(hit_color, shooter_color, x, y) => {

                    let hit_color = ColorName::from_u8(hit_color);
                    self.effects.push(Box::new(LaserBeamHit::from_point(
                        &mut self.particle_system,
                        hit_color,
                        x, y,
                        1.0
                    )));

                    if self.player.color == hit_color {
                        self.screen_effects.push(Box::new(ScreenFlash::new(
                            ColorName::from_u8(shooter_color),
                            500
                        )));
                    }

                },

                Action::LaserBeamKill(hit_color, shooter_color, x, y) => {

                    let hit_color = ColorName::from_u8(hit_color);

                    self.effects.push(Box::new(LaserBeamHit::from_point(
                        &mut self.particle_system,
                        hit_color,
                        x, y,
                        2.0
                    )));

                    if self.player.color == hit_color {
                        self.screen_effects.push(Box::new(ScreenFlash::new(
                            ColorName::from_u8(shooter_color),
                            2000
                        )));
                    }

                },

                _ => {}
            }
        }

        // Send client inputs to server
        for packet in entity_client.send(512) {
            client.send(cobalt::MessageKind::Instant, packet).ok();
        }

    }

    pub fn render(
        &mut self,
        renderer: &mut Renderer,
        entity_client: &mut hexahydrate::Client<Entity, ConnectionID, Registry>,
        level: &Level
    ) {

        // Clear
        renderer.clear_stencil(0);
        renderer.clear_color([0.0; 4]);

        // Prevent any rendering if not ready to play
        if !self.ready {
            return;
        }

        // Get player positions, colors and visibility
        let (t, u) = (renderer.t(), renderer.u());
        let players = entity_client.map_entities::<(PlayerData, [[f32; 4]; 2], f32), _>(|_, entity| {

            let p = entity.interpolate(u);
            if entity.is_local() {
                self.player.color = entity.color_name();
                self.player.data = p.clone();
                (p, entity.colors(), if entity.is_alive() { 1.0 } else { 0.0 })

            } else {
                let visibility = entity.update_visibility(
                    level,
                    &self.player.data,
                    &p,
                    t
                );
                (p, entity.colors(), visibility)
            }

        });

        // Camera setup
        self.camera.center(self.player.data.x, self.player.data.y);
        self.camera.limit(level.bounds());
        self.camera.apply(renderer);

        // Mouse inputs
        self.world_cursor = self.camera.s2w(self.screen_cursor.0, self.screen_cursor.1);
        self.input_angle = (
            self.world_cursor.1 - self.player.data.y

        ).atan2(self.world_cursor.0 - self.player.data.x);

        // Level Background
        level.render_background(
            renderer ,
            &self.camera,
            self.player.data.x,
            self.player.data.y,
            self.debug_level
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
                    self.player.circle.render(renderer, &q.scale(1.1, 1.1));

                    renderer.set_color(colors[0]);
                    self.player.circle.render(renderer, &q);

                    renderer.set_color(colors[1]);
                    self.player.cone.render(renderer, &q);

                }
            }

        }

        // World Effects
        for effect in &self.effects {
            effect.render(renderer, &self.camera);
        }

        self.effects.retain(|e| e.alive(t));

        // Particles
        {
            let (scale, context) = (self.camera.scalar(0.4), self.camera.context());
            self.particle_system.render(scale, &context.transform, renderer);
        }

        // Lights
        level.render_lights(
            renderer,
            &self.camera,
            self.debug_level
        );

        // Visibility / shadows
        level.render_shadow(
            renderer,
            &self.camera,
            &self.player.data,
            self.debug_level
        );

        // Level Walls
        level.render_walls(
            renderer,
            &self.camera,
            self.player.data.x,
            self.player.data.y,
            self.debug_level
        );

        // Screen Effects
        for effect in &self.screen_effects {
            effect.render(renderer, &self.camera);
        }

        self.screen_effects.retain(|e| e.alive(t));

        // HUD
        self.render_hud(renderer);

    }

    pub fn render_hud(&mut self, renderer: &mut Renderer) {

        let context = renderer.context().clone();
        renderer.set_color(self.player.colors[0]);
        renderer.rectangle(&context, &[
            self.screen_cursor.0 - 2.0, self.screen_cursor.1 - 2.0,
            4.0, 4.0
        ]);

        let (w, h) = (renderer.width(), renderer.height());
        renderer.line(&context, &[0.0, 0.0,   w, 0.0], 2.0);
        renderer.line(&context, &[0.0,   h,   w,   h], 2.0);
        renderer.line(&context, &[0.0, 0.0, 0.0,   h], 2.0);
        renderer.line(&context, &[w,   0.0,   w,   h], 2.0);

        let lh = (h - 40.0) * (1.0 - 1.0 / PLAYER_MAX_HP as f32 * self.player.data.hp as f32);
        renderer.line(&context, &[
            w - 30.0,
            20.0 + lh,
            w - 30.0,
            (h - 20.0)

        ], 10.0);

    }

}


// Helpers --------------------------------------------------------------------
struct LocalPlayerData {

    // State
    data: PlayerData,
    color: ColorName,
    last_beam_fire: u64,

    // Rendering
    colors: [[f32; 4]; 2],
    circle: Circle,
    cone: CircleArc
}

impl LocalPlayerData {
    fn new() -> LocalPlayerData {
        LocalPlayerData {

            // State
            data: PlayerData::default(),
            color: ColorName::Black,
            last_beam_fire: 0,

            // Rendering
            colors: [[0f32; 4]; 2],
            circle: Circle::new(10, 0.0, 0.0, PLAYER_RADIUS),
            cone: CircleArc::new(10, 0.0, 0.0, PLAYER_RADIUS, 0.0, consts::PI * 0.25),

        }
    }
}

fn download_map(addr: &str) -> Result<String, String> {

    let client = hyper::Client::new();
    client.get(format!("http://{}", addr).as_str())
        .header(hyper::header::Connection::close())
        .send()
        .map_err(|err| err.to_string())
        .and_then(|mut res| {
            let mut body = String::new();
            res.read_to_string(&mut body)
               .map_err(|err| err.to_string())
               .map(|_| body)

        }).and_then(|body| {
            Ok(body.to_string())
        })

}

