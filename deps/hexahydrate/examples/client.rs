// Crates ---------------------------------------------------------------------
extern crate cobalt;
extern crate hexahydrate;


// STD Dependencies -----------------------------------------------------------
use std::str;


// External Dependencies ------------------------------------------------------
use hexahydrate::Entity;


// Modules --------------------------------------------------------------------
mod shared;
use self::shared::PlayerEntity;


// Traits ---------------------------------------------------------------------
pub trait ClientEntity: hexahydrate::Entity<cobalt::ConnectionID> {
    fn update(&mut self) {}
    fn dropped(&mut self) {
        println!("Player Entity dropped");
    }
}


// Entities -------------------------------------------------------------------
impl ClientEntity for PlayerEntity {

}

impl Drop for PlayerEntity {
    fn drop(&mut self) {
        self.dropped();
    }
}


// Entity Registry ------------------------------------------------------------
#[derive(Debug)]
struct ClientRegistry {

}

impl hexahydrate::EntityRegistry<ClientEntity, cobalt::ConnectionID> for ClientRegistry {
    fn entity_from_bytes(&self, kind: u8, bytes: &[u8]) -> Option<Box<ClientEntity>> {
        match kind {
            1 => PlayerEntity::from_bytes(bytes).map(|e| Box::new(e) as Box<ClientEntity>),
            _ => None
        }
    }
}


// Client ---------------------------------------------------------------------
struct ClientHandler {
    client: hexahydrate::Client<ClientEntity, cobalt::ConnectionID, ClientRegistry>
}

impl ClientHandler {

    fn disconnect(&mut self, client: &mut cobalt::Client) {
        println!("Disconnected");
        client.close().unwrap();
        self.client.reset();
    }

}

impl cobalt::Handler<cobalt::Client> for ClientHandler {

    fn tick_connection(&mut self, _: &mut cobalt::Client, conn: &mut cobalt::Connection) {

        for msg in conn.received() {
            self.client.receive(msg).expect("Invalid packet received.");
        }

        self.client.update_with(|_, entity| {
            entity.update();
        });

        for packet in self.client.send(512) {
            conn.send(cobalt::MessageKind::Instant, packet);
        }

    }

    fn connection(&mut self, _: &mut cobalt::Client, _: &mut cobalt::Connection) {
        println!("Connected");
    }

    fn connection_failed(&mut self, client: &mut cobalt::Client, _: &mut cobalt::Connection) {
        self.disconnect(client);
    }

    fn connection_lost(&mut self, client: &mut cobalt::Client, _: &mut cobalt::Connection) {
        self.disconnect(client);
    }

    fn connection_closed(&mut self, client: &mut cobalt::Client, _: &mut cobalt::Connection, _: bool) {
        self.disconnect(client);
    }

}

fn main() {

    let config = cobalt::Config::default();
    let mut handler = ClientHandler {
        client: hexahydrate::Client::<ClientEntity, cobalt::ConnectionID, ClientRegistry>::new(ClientRegistry {

        }, 30)
    };
    let mut client = cobalt::Client::new(config);
    client.connect(&mut handler, "127.0.0.1:7156").unwrap();

}

