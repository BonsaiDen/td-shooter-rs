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
use shared::level::{Level, LevelCollision, LevelVisibility};
use shared::color::ColorName;
use shared::entity::{PlayerInput, PlayerPosition, PlayerEntity};


// Statics --------------------------------------------------------------------
const LASER_BEAM_LENGTH: f32 = 100.0;


// Server Implementation ------------------------------------------------------
pub struct Server {
    dt: f64,
    addr: String,
    connections: HashMap<ConnectionID, (
        hexahydrate::ConnectionSlot<ConnectionID>,
        hexahydrate::ServerEntitySlot,
        ColorName,
        VecDeque<Action>
    )>,
    available_colors: Vec<ColorName>
}

impl Server {

    pub fn new(addr: String, updates_per_second: u64) -> Server {
        Server {
            dt: 1.0 / updates_per_second as f64,
            addr: addr,
            connections: HashMap::new(),
            available_colors: ColorName::all_colored().into_iter().rev().collect()
        }
    }

    pub fn update(
        &mut self,
        entity_server: &mut hexahydrate::Server<Entity, ConnectionID>,
        server: &mut cobalt::ServerStream,
        level: &Level
    ) {

        let dt = self.dt;

        // Accept and receive connections / messages
        while let Ok(event) = server.accept_receive() {

            match event {
                cobalt::ServerEvent::Bind => {
                    println!("[Server] Now accepting connections on {}", self.addr);
                },
                cobalt::ServerEvent::Connection(id) => {
                    if let Some(conn) = server.connection_mut(&id) {
                        self.connect(entity_server, conn);
                    }
                },
                cobalt::ServerEvent::Message(id, packet) => {
                    if let Some(&mut (ref slot, _, _, ref mut incoming_actions)) = self.connections.get_mut(&id) {
                        match entity_server.connection_receive(slot, packet) {
                            Err(hexahydrate::ServerError::InvalidPacketData(bytes)) => {
                                if let Ok(action) = Action::from_bytes(&bytes) {
                                    // TODO limit number of maximum actions per second?
                                    incoming_actions.push_back(action);
                                }
                            },
                            _ => {}
                        }
                    }
                },
                cobalt::ServerEvent::ConnectionLost(id) => {
                    println!("[Server] Lost connection to client!");
                    self.disconnect(entity_server, &id);
                },
                cobalt::ServerEvent::ConnectionClosed(id, _) => {
                    println!("[Server] Closed connection to client.");
                    self.disconnect(entity_server, &id);
                },
                _ => {}
            }

        }

        // Update entities
        entity_server.update_with(|_, entity| {
            entity.update(dt, &level);
        });

        // Get current entity positions
        let current_entities = entity_server.map_entities::<(Option<ConnectionID>, PlayerPosition), _>(|_, entity| {
            (entity.owner(), entity.current_position())
        });

        // Apply actions
        let mut outgoing_actions = Vec::new();
        for (conn_id, &mut (ref slot, ref entity_slot, _, ref mut incoming_actions)) in &mut self.connections {

            // Apply Actions
            while let Some(action) = incoming_actions.pop_front() {
                println!("[Server] Received action from client: {:?}", action);
                match action {
                    Action::FiredLaserBeam(tick, client_r) => {
                        if let Some(entity) = entity_server.entity_get(entity_slot) {

                            let mut p = entity.position(tick);
                            p.merge_client_angle(client_r);

                            let (mut x, mut y, r, mut l) = (
                                // We move the origin of the beam into the player
                                // in order to avoid wall clipping
                                p.x + p.r.cos() * (PLAYER_RADIUS as f32 - 0.5),
                                p.y + p.r.sin() * (PLAYER_RADIUS as f32 - 0.5),
                                p.r,
                                LASER_BEAM_LENGTH
                            );

                            if let Some(intersection) = level.collide_beam(
                                x as f64,
                                y as f64,
                                r as f64,
                                l as f64
                            ) {
                                l = intersection[4] as f32;
                            }

                            // We now move the beam out of the player again and
                            // shorten it to fix any resulting wall clipping
                            x += r.cos() * 1.0;
                            y += r.sin() * 1.0;
                            l = (l - 1.0).max(0.0);

                            outgoing_actions.push(Action::CreateLaserBeam(
                                entity.color_name().to_u8(),
                                x, y, r, l
                            ));

                        }
                    },
                    _ => {}
                }
            }

            // Check to which other entities this player's entity is visible
            if let Some(player_entity) = entity_server.entity_get_mut(entity_slot) {

                let player_position = player_entity.current_position();

                // Check if player is in one of the level lights
                let player_in_light = level.circle_in_light(
                    player_position.x as f64,
                    player_position.y as f64,
                    PLAYER_RADIUS * 1.5
                );

                // Check and set visibility to other entities
                for &(entity_conn_id, ref position) in &current_entities {
                    if let Some(ref entity_conn_id) = entity_conn_id {

                        // Ignore self-visibility
                        if entity_conn_id != conn_id {
                            let visible = player_in_light || level.circle_visible_from(
                                player_position.x as f64,
                                player_position.y as f64,
                                PLAYER_RADIUS * 1.5,
                                position.x as f64,
                                position.y as f64
                            );
                            player_entity.set_visibility(*entity_conn_id, visible);
                        }

                    }

                }

            }

            // Send updates to clients
            for packet in entity_server.connection_send(slot, 512).unwrap() {
                server.send(conn_id, cobalt::MessageKind::Instant, packet).ok();
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

    fn connect(
        &mut self,
        entity_server: &mut hexahydrate::Server<Entity, ConnectionID>,
        conn: &mut cobalt::Connection
    ) {

        // TODO do not directly create a entity but rather add the connection and then wait for a
        // "JoinGame" Action and create the entity based on that
        if let Ok(slot) = entity_server.connection_add(conn.id()) {

            if let Some(color) = self.available_colors.pop() {

                // Create a new player entity for the connected client
                if let Ok(entity_slot) = entity_server.entity_create_with(|| {

                    Box::new(PlayerEntity::<ServerState<PlayerPosition, PlayerInput>>::new(Some(conn.id()), false, color, PlayerPosition {
                        x: 0.0,
                        y: 0.0,
                        r: 0.0,
                        visible: true
                    }))

                }) {
                    println!("[Server] New client connection.");
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

    fn disconnect(
        &mut self,
        entity_server: &mut hexahydrate::Server<Entity, ConnectionID>,
        id: &ConnectionID
    ) {
        if let Some((slot, entity_slot, color, _)) = self.connections.remove(id) {
            println!("[Server] Client disconnected.");
            entity_server.entity_destroy(entity_slot).ok();
            entity_server.connection_remove(slot).expect("Connection does not exist.");
            self.available_colors.push(color);
        }
    }

}

