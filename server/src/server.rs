// STD Dependencies -----------------------------------------------------------
use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};


// External Dependencies ------------------------------------------------------
use hexahydrate;
use cobalt;
use cobalt::ConnectionID;
use netsync::ServerState;


// Internal Dependencies ------------------------------------------------------
use ::entity::Entity;
use shared::util;
use shared::entity::{
    PLAYER_RADIUS,
    PLAYER_MAX_HP,
    PLAYER_SERVER_VISIBILITY_SCALING,
    PLAYER_VISBILITY_CONE,
    PLAYER_VISBILITY_CONE_OFFSET
};
use shared::action::Action;
use shared::level::{
    Level, LevelCollision, LevelVisibility,
    LEVEL_MAX_VISIBILITY_DISTANCE,
    line_intersect_circle
};
use shared::color::ColorName;
use shared::entity::{PlayerInput, PlayerData, PlayerEntity};


// Statics --------------------------------------------------------------------
const LASER_BEAM_LENGTH: f32 = 100.0;


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
        Server {
            dt: 1.0 / updates_per_second as f32,
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
        let current_entities = entity_server.map_entities::<(Option<ConnectionID>, PlayerData), _>(|_, entity| {
            (entity.owner(), entity.current_data())
        });

        let mut outgoing_actions: Vec<(Option<ConnectionID>, Action)> = Vec::new();
        let mut beam_hits: Vec<(ConnectionID, ConnectionID)> = Vec::new();

        // Apply and generate actions
        for (conn_id, &mut (_, ref entity_slot, _, ref mut incoming_actions)) in &mut self.connections {

            while let Some(action) = incoming_actions.pop_front() {

                println!("[Server] Received action from client: {:?}", action);
                match action {
                    Action::FiredLaserBeam(tick, client_r) => {

                        // Correct firing angle to be somewhere between server
                        // and client side value
                        let entity = if let Some(entity) = entity_server.entity_get(entity_slot) {

                            let mut data = entity.data(tick);
                            data.merge_client_angle(client_r);

                            // Ignore action from dead entities
                            if data.hp > 0 {
                                Some((data, entity.color_name()))

                            } else {
                                None
                            }

                        } else {
                            None
                        };

                        if let Some((data, color_name)) = entity {

                            // Create initial laser beam
                            let (beam_line, mut l, r, _) = create_laser_beam(&level, &data);

                            // Get entity dat based on tick delay from client side action
                            let client_side_entities = entity_server.map_entities::<(Option<ConnectionID>, PlayerData), _>(|_, entity| {
                                (entity.owner(), entity.data(tick))
                            });

                            // TODO handle mirror walls and bounced off beams which hit the player
                            if let Some((hit_conn_id, hit_l)) = check_laser_beam_hits(
                                conn_id,
                                &beam_line,
                                l,
                                &client_side_entities
                            ) {
                                beam_hits.push((*conn_id, hit_conn_id));
                                l = hit_l;
                            }

                            // Send beam firing action to all players
                            outgoing_actions.push((
                                None,
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
        for (shooter_conn_id, hit_conn_id) in beam_hits {

            if let Some(entity) = entity_server.entity_get_mut(&self.connections.get(&hit_conn_id).unwrap().1) {

                println!("[Server] Beam Hit: {:?} -> {:?}", shooter_conn_id, hit_conn_id);

                // TODO reduce entity HP and if it reaches 0 setup respawn timer
                let data = entity.data(0);

                // Send hit notification to both involved players
                outgoing_actions.push((
                    Some(shooter_conn_id),
                    Action::LaserBeamHit(entity.color_name().to_u8(), data.x, data.y)
                ));

                outgoing_actions.push((
                    Some(hit_conn_id),
                    Action::LaserBeamHit(entity.color_name().to_u8(), data.x, data.y)
                ));

            }

        }

        // Update visibility and send entitiy data to clients
        for (conn_id, &mut (ref slot, ref entity_slot, _, _)) in &mut self.connections {

            // Check to which other entities this player's entity is visible
            if let Some(player_entity) = entity_server.entity_get_mut(entity_slot) {

                let player_data = player_entity.current_data();

                // Check if player is in one of the level lights
                let player_in_light = level.circle_in_light(
                    player_data.x,
                    player_data.y,
                    PLAYER_RADIUS * PLAYER_SERVER_VISIBILITY_SCALING
                );

                // Check and set visibility to other entities
                for &(entity_conn_id, ref data) in &current_entities {
                    if let Some(ref entity_conn_id) = entity_conn_id {

                        // Ignore self-visibility
                        if entity_conn_id != conn_id {

                            // TODO merge with client side visibility check
                            let distance = util::distance(
                                data.x, data.y,
                                player_data.x, player_data.y
                            );

                            // Entities standing in lights are always visible
                            let visible = if player_in_light {
                                true

                            // Dead entities can see no other entities
                            } else if player_data.hp == 0 {
                                false

                            // Entities outside the maximum visibility radius are never visible
                            } else if distance > LEVEL_MAX_VISIBILITY_DISTANCE - PLAYER_VISBILITY_CONE_OFFSET + PLAYER_RADIUS * 0.5 {
                                false

                            } else {

                                // Entities outside the visibility cone are never visible
                                if !util::angle_within_cone(
                                    data.x, data.y, data.r,
                                    player_data.x, player_data.y,
                                    PLAYER_VISBILITY_CONE_OFFSET,
                                    PLAYER_VISBILITY_CONE
                                ) {
                                    false

                                } else {
                                    // Entities within the visibility cone are only visible if sight is not blocked by a wall
                                    level.circle_visible_from(
                                        player_data.x,
                                        player_data.y,
                                        PLAYER_RADIUS * PLAYER_SERVER_VISIBILITY_SCALING,
                                        data.x,
                                        data.y
                                    )
                                }

                            };

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
            for &(filter_id, ref action) in &outgoing_actions {

                let send_to_connection = if filter_id.is_none() {
                    true

                } else {
                    filter_id.unwrap() == *id
                };

                if send_to_connection {
                    server.send(id, cobalt::MessageKind::Reliable, action.to_bytes()).ok();
                }

            }
        }

        // This sleeps to achieve the desired server tick rate
        server.flush().ok();

    }


    // Connection Handling ----------------------------------------------------
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

                    Box::new(PlayerEntity::<ServerState<PlayerData, PlayerInput>>::new(Some(conn.id()), false, color, PlayerData {
                        x: -60.0,
                        y: 20.0,
                        r: 0.0,
                        visible: true,
                        hp: PLAYER_MAX_HP
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


// Helpers --------------------------------------------------------------------
fn create_laser_beam(
    level: &Level,
    p: &PlayerData

) -> ([f32; 4], f32, f32, Option<usize>) {

    let (mut x, mut y, r, mut l) = (
        // We move the origin of the beam into the player
        // in order to avoid wall clipping
        p.x + p.r.cos() * (PLAYER_RADIUS - 0.5),
        p.y + p.r.sin() * (PLAYER_RADIUS - 0.5),
        p.r,
        LASER_BEAM_LENGTH
    );

    // Collide with level walls
    let mut wall: Option<usize> = None;
    if let Some(intersection) = level.collide_beam(
        x,
        y,
        r,
        l
    ) {
        // TODO check if the wall was a mirror
        // TODO get wall normal
        // TODO calculate reflection normal from beam and wall normal
        l = intersection.1[2];
        wall = Some(intersection.0);
    }

    // We now move the beam out of the player again and
    // shorten it to fix any resulting wall clipping
    x += r.cos() * 1.0;
    y += r.sin() * 1.0;
    l = (l - 1.0).max(0.0);

    (
        [
            x,
            y,
            x + r.cos() * l,
            y + r.sin() * l
        ],
        l,
        r,
        wall
    )

}

fn check_laser_beam_hits(
    conn_id: &ConnectionID,
    beam_line: &[f32; 4],
    l: f32,
    entities: &[(Option<ConnectionID>, PlayerData)]

) -> Option<(ConnectionID, f32)> {

    // Hit detection against nearest entities
    let mut nearest_entities = Vec::new();
    for &(entity_conn_id, ref data) in entities {
        if let Some(ref entity_conn_id) = entity_conn_id {

            // Don't let players hit themseves
            if entity_conn_id != conn_id && data.hp > 0 {

                // Ignore entities outside of beam range
                let distance = util::distance(data.x, data.y, beam_line[0], beam_line[1]);
                if distance < l {
                    nearest_entities.push((distance, data.x, data.y, *entity_conn_id));
                }

            }

        }
    }

    // Sort by nearest entity first
    nearest_entities.sort_by(|a, b| {
        if a.0 > b.0 {
            Ordering::Greater

        } else if a.0 < b.0 {
            Ordering::Less

        } else {
            Ordering::Equal
        }
    });

    // Find first entity which is hit by beam
    for &(l, x, y, entity_conn_id) in &nearest_entities {
        if let Some(intersection) = line_intersect_circle(&beam_line, x, y, PLAYER_RADIUS) {
            return Some((entity_conn_id, l - intersection[6]));
        }
    }

    None

}

