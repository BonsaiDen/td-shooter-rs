// STD Dependencies -----------------------------------------------------------
use std::collections::{HashMap, VecDeque};


// External Dependencies ------------------------------------------------------
use hexahydrate;
use cobalt;
use cobalt::ConnectionID;
use netsync::ServerState;


// Internal Dependencies ------------------------------------------------------
use ::entity::Entity;
use shared::entity::PLAYER_RADIUS;
use shared::action::Action;
use shared::level::Level;
use shared::color::ColorName;
use shared::entity::{PlayerInput, PlayerPosition, PlayerEntity};


// Server Implementation ------------------------------------------------------
pub struct Server {
    dt: f64,
    server: hexahydrate::Server<Entity, ConnectionID>,
    connections: HashMap<ConnectionID, (
        hexahydrate::ConnectionSlot<ConnectionID>,
        hexahydrate::ServerEntitySlot,
        ColorName,
        VecDeque<Action>
    )>,
    available_colors: Vec<ColorName>
}

impl Server {

    pub fn new(updates_per_second: u64) -> Server {
        Server {
            dt: 1.0 / updates_per_second as f64,
            server: hexahydrate::Server::<Entity, ConnectionID>::new((updates_per_second * 2) as usize),
            connections: HashMap::new(),
            available_colors: ColorName::all_colored().into_iter().rev().collect()
        }
    }

    pub fn update(&mut self, level: &Level, server: &mut cobalt::ServerStream) {

        let dt = self.dt;

        // Accept and receive connections / messages
        while let Ok(event) = server.accept_receive() {

            match event {
                cobalt::ServerEvent::Bind => {
                    println!("[Server] Now accepting connections...");
                },
                cobalt::ServerEvent::Connection(id) => {
                    if let Some(conn) = server.connection_mut(&id) {
                        self.connect(conn);
                    }
                },
                cobalt::ServerEvent::Message(id, packet) => {
                    if let Some(&mut (ref slot, _, _, ref mut incoming_actions)) = self.connections.get_mut(&id) {
                        match self.server.connection_receive(slot, packet) {
                            Err(hexahydrate::ServerError::InvalidPacketData(bytes)) => {
                                if let Ok(action) = Action::from_bytes(&bytes) {
                                    // TODO limit number of maximum actions?
                                    incoming_actions.push_back(action);
                                }
                            },
                            _ => {}
                        }
                    }
                },
                cobalt::ServerEvent::ConnectionLost(id) => {
                    println!("[Server] Lost connection to client!");
                    self.disconnect(&id);
                },
                cobalt::ServerEvent::ConnectionClosed(id, _) => {
                    println!("[Server] Closed connection to client.");
                    self.disconnect(&id);
                },
                _ => {}
            }

        }

        // Update entities
        self.server.update_with(|_, entity| {
            entity.update(dt, &level);
        });

        // Apply actions
        let mut outgoing_actions = Vec::new();
        for (id, &mut (ref slot, ref entity_slot, _, ref mut incoming_actions)) in &mut self.connections {

            // Apply Actions
            while let Some(action) = incoming_actions.pop_front() {
                println!("[Server] Received action from client: {:?}", action);
                match action {
                    Action::FiredLaserBeam(tick, client_r) => {
                        if let Some(entity) = self.server.entity_get(entity_slot) {

                            // TODO unify with PlayerPosition stuff
                            let mut p = entity.position(tick);
                            p.merge_client_angle(client_r);
                            //let r = client_r - p.r;
                            //let dr = r.sin().atan2(r.cos());

                            //println!("[Server] Shot (server) {} (local) {} (diff) {}", p.r, client_r, dr);

                            // TODO laser beam collision
                            //dr.min(consts::PI * 0.125).max(-consts::PI * 0.125);
                            outgoing_actions.push(Action::CreateLaserBeam(
                                entity.color_name().to_u8(),
                                p.x + p.r.cos() * (PLAYER_RADIUS as f32 + 0.5),
                                p.y + p.r.sin() * (PLAYER_RADIUS as f32 + 0.5),
                                p.r, // TODO apply correction with client angle?
                                100
                            ));

                        }
                    },
                    _ => {}
                }
            }

            // Send updates to clients
            for packet in self.server.connection_send(slot, 512).unwrap() {
                server.send(id, cobalt::MessageKind::Instant, packet).ok();
            }

        }

        // Send new actions
        for (id, _) in &mut self.connections {
            for action in &outgoing_actions {
                server.send(id, cobalt::MessageKind::Reliable, action.to_bytes()).ok();
            }
        }

        // This sleeps to achieve the desired server tick rate
        server.flush().ok();

    }

    fn connect(&mut self, conn: &mut cobalt::Connection) {

        // TODO do not directly create a entity but rather add the connection and then wait for a
        // "JoinGame" Action and create the entity based on that
        if let Ok(slot) = self.server.connection_add(conn.id()) {

            if let Some(color) = self.available_colors.pop() {

                // Create a new player entity for the connected client
                if let Ok(entity_slot) = self.server.entity_create_with(|| {
                    Box::new(PlayerEntity::<ServerState<PlayerPosition, PlayerInput>>::new(Some(conn.id()), false, color, PlayerPosition {
                        x: -50.0,
                        y: 0.0,
                        r: 0.0
                    }))

                }) {
                    println!("[Server] Client connected.");
                    self.connections.insert(
                        conn.id(), (slot, entity_slot, color, VecDeque::new())
                    );

                } else {
                    println!("[Server] No more entity slots.");
                    conn.close()
                }

            } else {
                println!("[Server] No more available colors.");
                conn.close();
            }

        } else {
            println!("[Server] No more connection slots.");
            conn.close();
        }

    }

    fn disconnect(&mut self, id: &ConnectionID) {
        if let Some((slot, entity_slot, color, _)) = self.connections.remove(id) {
            println!("[Server] Client disconnected.");
            self.server.entity_destroy(entity_slot).ok();
            self.server.connection_remove(slot).expect("Connection does not exist.");
            self.available_colors.push(color);
        }
    }

}

