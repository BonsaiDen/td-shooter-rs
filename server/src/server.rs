// STD Dependencies -----------------------------------------------------------
use std::collections::{HashMap, VecDeque};


// External Dependencies ------------------------------------------------------
use rand;
use rand::Rng;
use clock_ticks;
use hexahydrate;
use cobalt;
use cobalt::ConnectionID;
use netsync::ServerState;


// Internal Dependencies ------------------------------------------------------
use ::Timer;
use ::laser_beam;
use ::entity::Entity;
use shared::color::ColorName;
use shared::level::{
    Level, LevelVisibility, LevelSpawn,
    LEVEL_MAX_BEAM_VISIBILITY_DISTANCE
};
use shared::collision::aabb_intersect_circle;
use shared::entity::{PLAYER_MAX_HP, PLAYER_RESPAWN_INTERVAL, ENTITY_STATE_DELAY};
use shared::action::{Action, ActionVisibility};
use shared::entity::{PlayerInput, PlayerData, PlayerEntity};


// Server Implementation ------------------------------------------------------
pub struct Server {
    dt: f32,
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
        let mut colors: Vec<ColorName> = ColorName::all_colored().into_iter().rev().collect();
        rand::thread_rng().shuffle(&mut colors);
        Server {
            dt: 1.0 / updates_per_second as f32,
            addr: addr,
            connections: HashMap::new(),
            available_colors: colors
        }
    }

    pub fn update(
        &mut self,
        timer: &mut Timer,
        entity_server: &mut hexahydrate::Server<Entity, ConnectionID>,
        server: &mut cobalt::ServerStream,
        level: &Level
    ) {

        self.receive(entity_server, server, level);
        self.update_entities_before(entity_server, level);

        let actions = self.apply_actions(timer, entity_server, level);
        self.update_entities_after(entity_server, level);
        self.send(entity_server, server, level, &actions);

        // This sleeps to achieve the desired server tick rate
        server.flush().ok();

    }

    fn receive(
        &mut self,
        entity_server: &mut hexahydrate::Server<Entity, ConnectionID>,
        server: &mut cobalt::ServerStream,
        level: &Level
    ) {

        while let Ok(event) = server.accept_receive() {

            match event {
                cobalt::ServerEvent::Bind => {
                    println!("[Server] Now accepting connections on {}", self.addr);
                },
                cobalt::ServerEvent::Connection(id) => {
                    if let Some(conn) = server.connection_mut(&id) {
                        self.connect(entity_server, level, conn);
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

    }

    fn update_entities_before(
        &mut self,
        entity_server: &mut hexahydrate::Server<Entity, ConnectionID>,
        level: &Level
    ) {
        entity_server.update_with(|_, entity| {
            entity.update(self.dt, &level);
        });
    }

    fn apply_actions(
        &mut self,
        timer: &mut Timer,
        entity_server: &mut hexahydrate::Server<Entity, ConnectionID>,
        level: &Level

    ) -> Vec<(ActionVisibility, Action)> {

        let t = clock_ticks::precise_time_ms();
        let mut outgoing_actions: Vec<(ActionVisibility, Action)> = Vec::new();
        let mut beam_hits: Vec<(ConnectionID, ColorName, ConnectionID)> = Vec::new();

        for (conn_id, &mut (_, ref entity_slot, _, ref mut incoming_actions)) in &mut self.connections {

            while let Some(action) = incoming_actions.pop_front() {

                match action {

                    // TODO should we perform a persistent check for the duration
                    // of the laser beam?
                    Action::FiredLaserBeam(tick, client_r) => {

                        // Correct firing angle to be somewhere between server
                        // and client side value
                        let entity = if let Some(entity) = entity_server.entity_get_mut(entity_slot) {

                            let mut data = entity.client_data(tick, 0);
                            data.merge_client_angle(client_r);

                            // Ignore action from dead client entities
                            if data.hp > 0 && entity.fire_beam(t) {
                                Some((data, entity.color_name()))

                            } else {
                                None
                            }

                        } else {
                            None
                        };

                        if let Some((data, color_name)) = entity {

                            // Create initial laser beam
                            let (beam_line, mut l, r, _) = laser_beam::create(&level, &data);

                            // Get entity data for both the current server state and as it was seen on the client when they fired
                            let client_side_entities = entity_server.map_entities::<(Option<ConnectionID>, PlayerData, PlayerData), _>(|_, entity| {
                                (entity.owner(), entity.current_data(), entity.client_data(tick, ENTITY_STATE_DELAY))
                            });

                            // TODO handle mirror walls and bounced off beams which hit the player
                            if let Some((hit_conn_id, hit_l)) = laser_beam::get_player_hits(
                                conn_id,
                                &beam_line,
                                l,
                                &client_side_entities
                            ) {
                                beam_hits.push((*conn_id, color_name, hit_conn_id));
                                l = hit_l;
                            }

                            // Send beam firing action to all players
                            outgoing_actions.push((
                                ActionVisibility::WithinRange {
                                    aabb: [
                                       beam_line[0].min(beam_line[2]),
                                       beam_line[1].min(beam_line[3]),
                                       beam_line[0].max(beam_line[2]),
                                       beam_line[1].max(beam_line[3])
                                    ],
                                    r: LEVEL_MAX_BEAM_VISIBILITY_DISTANCE
                                },
                                Action::CreateLaserBeam(
                                    color_name.to_u8(),
                                    beam_line[0],
                                    beam_line[1],
                                    r,
                                    l
                                )
                            ));

                        }

                    },
                    _ => {}
                }
            }

        }

        // Handle laser beam hits
        for (shooter_conn_id, shooter_color, hit_conn_id) in beam_hits {

            if let Some(entity) = entity_server.entity_get_mut(&self.connections.get(&hit_conn_id).unwrap().1) {

                // Apply laser damage to entity, this saturates at 0 hp
                entity.damage(64);

                // TODO we need a simple timer system
                // VecDeque, sort to be next first when inserting (insert
                // clock_ticks::precise_time_ms() + delay)
                // Need to put a move closure
                // execute before receiving any client input (or after server tick delay, has
                // the same effect just different position)
                // When executing pop while entry timer is <=
                // clock_ticks::precise_time_ms()

                if entity.is_alive() {

                    println!("[Server] Beam Hit: {:?} -> {:?}", shooter_conn_id, hit_conn_id);

                    let data = entity.current_data();
                    let action = Action::LaserBeamHit(entity.color_name().to_u8(), shooter_color.to_u8(), data.x, data.y);

                    // Send action to all other players, except for the shooter
                    outgoing_actions.push((ActionVisibility::Entity(data, Some(shooter_conn_id)), action.clone()));

                    // Send another action just for the shooter to ensure he always
                    // gets the hit marker
                    outgoing_actions.push((ActionVisibility::Connection(shooter_conn_id), action));

                } else {

                    println!("[Server] Beam Kill: {:?} -> {:?}", shooter_conn_id, hit_conn_id);

                    let data = entity.current_data();
                    let action = Action::LaserBeamKill(entity.color_name().to_u8(), shooter_color.to_u8(), data.x, data.y);

                    // Send action to all other players, except for the shooter
                    outgoing_actions.push((ActionVisibility::Entity(data.clone(), Some(shooter_conn_id)), action.clone()));

                    // Send another action just for the shooter to ensure he always
                    // gets the hit marker
                    outgoing_actions.push((ActionVisibility::Connection(shooter_conn_id), action.clone()));

                    // Also send a action to the killed player, he can't see himself since he is no
                    // longer alive and would otherwise not receive it
                    outgoing_actions.push((ActionVisibility::Connection(hit_conn_id), action));

                    // Re-spawn logic
                    timer.schedule(move |server, entity_server, _, level| {

                        let spawn = server.find_player_spawn(entity_server, level);

                        if let Some(conn) = server.connections.get(&hit_conn_id) {
                            if let Some(entity) = entity_server.entity_get_mut(&conn.1) {
                                println!("[Server] Respawning player...");
                                entity.respawn(spawn);
                            }
                        }

                    }, PLAYER_RESPAWN_INTERVAL);

                };

            }

        }

        outgoing_actions

    }

    fn update_entities_after(
        &mut self,
        entity_server: &mut hexahydrate::Server<Entity, ConnectionID>,
        level: &Level
    ) {

        let entity_data = entity_server.map_entities::<(Option<ConnectionID>, PlayerData), _>(|_, entity| {
            (entity.owner(), entity.current_data())
        });

        // Update visibility and send entitiy data to clients
        for (conn_id, &mut (_, ref entity_slot, _, _)) in &mut self.connections {

            // Check to which other entities this player's entity is visible
            if let Some(player_entity) = entity_server.entity_get_mut(entity_slot) {

                let player_data = player_entity.current_data();

                // Check to which other entities the player is visible
                for &(other_conn_id, ref entity_data) in &entity_data {
                    if let Some(ref other_conn_id) = other_conn_id {

                        // Ignores self-visibility
                        if other_conn_id != conn_id {
                            player_entity.set_visibility(
                                *other_conn_id,
                                level.player_within_visibility(
                                    entity_data, &player_data
                                )
                            );
                        }

                    }

                }

            }

        }

    }

    fn send(
        &mut self,
        entity_server: &mut hexahydrate::Server<Entity, ConnectionID>,
        server: &mut cobalt::ServerStream,
        level: &Level,
        actions: &[(ActionVisibility, Action)]
    ) {

        for (conn_id, &mut (ref slot, ref entity_slot, _, _)) in &mut self.connections {

            // Send out entity state updates
            for packet in entity_server.connection_send(slot, 512).unwrap() {
                server.send(conn_id, cobalt::MessageKind::Instant, packet).ok();
            }

            // Send out actions
            let entity = entity_server.entity_get(entity_slot);
            for &(ref visibility, ref action) in actions {

                let send_to_connection = match *visibility {
                    ActionVisibility::Any => true,
                    ActionVisibility::Connection(filter_conn_id) => *conn_id == filter_conn_id,
                    ActionVisibility::Entity(ref other_data, filter_conn_id) => {
                        if filter_conn_id.is_some() && filter_conn_id.unwrap() == *conn_id {
                            false

                        } else if let Some(entity) = entity {
                            level.player_within_visibility(
                                &entity.current_data(),
                                other_data
                            )

                        } else {
                            false
                        }
                    },
                    ActionVisibility::WithinRange { aabb, r } => {
                        if let Some(entity) = entity {
                            let data = entity.current_data();
                            aabb_intersect_circle(&aabb, data.x, data.y, r)

                        } else {
                            false
                        }
                    }
                };

                if send_to_connection {
                    server.send(
                        conn_id,
                        cobalt::MessageKind::Reliable,
                        action.to_bytes()

                    ).ok();
                }

            }
        }

    }


    // Spawn Handling ---------------------------------------------------------
    fn find_player_spawn(
        &mut self,
        _: &mut hexahydrate::Server<Entity, ConnectionID>,
        level: &Level

    ) -> LevelSpawn {
        let spawns = level.randomized_spawns();
        // TODO find the spawn with the lowest number of players nearby
        spawns.get(0).unwrap().clone()
    }


    // Connection Handling ----------------------------------------------------
    fn connect(
        &mut self,
        entity_server: &mut hexahydrate::Server<Entity, ConnectionID>,
        level: &Level,
        conn: &mut cobalt::Connection
    ) {

        // Find a potential spawn point
        let spawn = self.find_player_spawn(entity_server, level);

        // TODO do not directly create a entity but rather add the connection and then wait for a
        // "JoinGame" Action and create the entity based on that
        if let Ok(slot) = entity_server.connection_add(conn.id()) {

            if let Some(color) = self.available_colors.pop() {

                // Create a new player entity for the connected client
                if let Ok(entity_slot) = entity_server.entity_create_with(|| {

                    Box::new(PlayerEntity::<ServerState<PlayerData, PlayerInput>>::new(
                        Some(conn.id()),
                        false,
                        color,
                        PlayerData::new(spawn.x, spawn.y, 0.0, PLAYER_MAX_HP)
                    ))

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

