// STD Dependencies -----------------------------------------------------------
use std::collections::HashMap;


// External Dependencies ------------------------------------------------------
use hexahydrate;
use cobalt;
use cobalt::ConnectionID;
use netsync::ServerState;


// Internal Dependencies ------------------------------------------------------
use ::entity::Entity;
use shared::action::Action;
use shared::level::Level;
use shared::color::ColorName;
use shared::entity::{PlayerInput, PlayerPosition, PlayerEntity};


// Server Implementation ------------------------------------------------------
pub struct Server {
    dt: f64,
    level: Option<Level>,
    server: hexahydrate::Server<Entity, ConnectionID>,
    connections: HashMap<ConnectionID, (hexahydrate::ConnectionSlot<ConnectionID>, hexahydrate::ServerEntitySlot, ColorName)>,
    available_colors: Vec<ColorName>
}

impl Server {

    pub fn new(updates_per_second: u64, level: Level) -> Server {
        Server {
            dt: 1.0 / updates_per_second as f64,
            level: Some(level),
            server: hexahydrate::Server::<Entity, ConnectionID>::new((updates_per_second * 2) as usize),
            connections: HashMap::new(),
            available_colors: ColorName::all_colored().into_iter().rev().collect()
        }
    }

    fn disconnect(&mut self, connection: &mut cobalt::Connection) {
        if let Some((slot, entity_slot, color)) = self.connections.remove(&connection.id()) {
            println!("Client disconnected");
            self.server.entity_destroy(entity_slot).ok();
            self.server.connection_remove(slot).expect("Connection does not exist.");
            self.available_colors.push(color);
        }
    }

}

impl cobalt::Handler<cobalt::Server> for Server {

    fn tick_connections(
        &mut self, _: &mut cobalt::Server,
        connections: &mut HashMap<ConnectionID, cobalt::Connection>
    ) {

        // Receive client inputs
        for (id, conn) in connections.iter_mut() {
            for packet in conn.received() {
                if let Some(&(ref slot, _, _)) = self.connections.get(id) {
                    match self.server.connection_receive(slot, packet) {
                        Err(hexahydrate::ServerError::InvalidPacketData(bytes)) => {
                            println!("[Server] Unknown packet data: {:?}", bytes);
                            println!("{:?}", Action::from_bytes(&bytes));
                        },
                        _ => {}
                    }
                }
            }
        }

        // TODO clean this up and get rif of the take() workaround
        // TODO synchronous / event driven cobalt server?
        let dt = self.dt;
        let level = self.level.take().unwrap();
        self.server.update_with(|_, entity| {
            // TODO need to get the connection handle so we can get the actions
            entity.update(dt, &level);
        });
        self.level = Some(level);

        // Send updates to clients
        for (id, conn) in connections.iter_mut() {
            if let Some(&(ref slot, _, _)) = self.connections.get(id) {
                for packet in self.server.connection_send(slot, 512).unwrap() {
                    conn.send(cobalt::MessageKind::Instant, packet);
                }
            }
        }

    }

    fn connection(&mut self, _: &mut cobalt::Server, conn: &mut cobalt::Connection) {

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
                    println!("Client connected");
                    self.connections.insert(conn.id(), (slot, entity_slot, color));

                } else {
                    conn.close()
                }

            } else {
                conn.close();
            }

        } else {
            conn.close();
        }

    }

    fn connection_lost(&mut self, _: &mut cobalt::Server, conn: &mut cobalt::Connection) {
        self.disconnect(conn);
    }

    fn connection_closed(&mut self, _: &mut cobalt::Server, conn: &mut cobalt::Connection, _: bool) {
        self.disconnect(conn);
    }

}

