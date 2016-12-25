// Crates ---------------------------------------------------------------------
extern crate cobalt;
extern crate hexahydrate;


// STD Dependencies -----------------------------------------------------------
use std::collections::HashMap;


// Modules --------------------------------------------------------------------
mod shared;
use self::shared::PlayerEntity;


// Traits ---------------------------------------------------------------------
pub trait ServerEntity: hexahydrate::Entity<cobalt::ConnectionID> {
    fn update(&mut self) {}
    fn dropped(&mut self) {
        println!("Player Entity dropped");
    }
}


// Entities -------------------------------------------------------------------
impl ServerEntity for PlayerEntity {

}

impl Drop for PlayerEntity {
    fn drop(&mut self) {
        self.dropped();
    }
}


// Server ---------------------------------------------------------------------
struct ServerHandler {
    server: hexahydrate::Server<ServerEntity, cobalt::ConnectionID>,
    connections: HashMap<cobalt::ConnectionID, (hexahydrate::ConnectionSlot<cobalt::ConnectionID>, hexahydrate::ServerEntitySlot)>
}

impl ServerHandler {

    fn disconnect(&mut self, connection: &mut cobalt::Connection) {
        if let Some((slot, entity_slot)) = self.connections.remove(&connection.id()) {
            println!("Client disconnected");
            self.server.entity_destroy(entity_slot).ok();
            self.server.connection_remove(slot).expect("Connection does not exist.");
        }
    }

}

impl cobalt::Handler<cobalt::Server> for ServerHandler {

    fn tick_connections(
        &mut self, _: &mut cobalt::Server,
        connections: &mut HashMap<cobalt::ConnectionID, cobalt::Connection>
    ) {

        for (id, conn) in connections.iter_mut() {
            for msg in conn.received() {
                if let Some(&(ref slot, _)) = self.connections.get(id) {
                    self.server.connection_receive(slot, msg).expect("Invalid packet received.");
                }
            }
        }

        self.server.update_with(|_, entity| {
            entity.update();
        });

        for (id, conn) in connections.iter_mut() {
            if let Some(&(ref slot, _)) = self.connections.get(id) {
                for packet in self.server.connection_send(slot, 512).unwrap() {
                    conn.send(cobalt::MessageKind::Instant, packet);
                }
            }
        }
    }

    fn connection(&mut self, _: &mut cobalt::Server, conn: &mut cobalt::Connection) {
        if let Ok(slot) = self.server.connection_add(conn.id()) {
            if let Ok(entity_slot) = self.server.entity_create_with(|| Box::new(PlayerEntity::new(Some(conn.id()), false))) {
                println!("Client connected");
                self.connections.insert(conn.id(), (slot, entity_slot));

            } else {
                conn.close()
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


fn main() {

    let config = cobalt::Config::default();
    let mut handler = ServerHandler {
        server: hexahydrate::Server::<ServerEntity, cobalt::ConnectionID>::new(30),
        connections: HashMap::new()
    };

    let mut server = cobalt::Server::new(config);
    server.bind(&mut handler, "127.0.0.1:7156").unwrap();

}

