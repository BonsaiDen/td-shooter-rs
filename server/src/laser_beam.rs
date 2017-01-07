// STD Dependencies -----------------------------------------------------------
use std::cmp::Ordering;


// External Dependencies ------------------------------------------------------
use cobalt::ConnectionID;


// Internal Dependencies ------------------------------------------------------
use shared::util;
use shared::entity::PlayerData;
use shared::entity::PLAYER_RADIUS;
use shared::level::{Level, LevelCollision};
use shared::collision::line_segment_intersect_circle;


// Statics --------------------------------------------------------------------
const LASER_BEAM_LENGTH: f32 = 90.0;


// Laser Beam Helpers ---------------------------------------------------------
pub fn create(
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

pub fn get_player_hits(
    conn_id: &ConnectionID,
    beam_line: &[f32; 4],
    l: f32,
    entities: &[(Option<ConnectionID>, PlayerData, PlayerData)]

) -> Option<(ConnectionID, f32)> {

    // Hit detection against nearest entities
    let mut nearest_entities = Vec::new();
    for &(entity_conn_id, ref server_data, ref client_data) in entities {
        if let Some(ref entity_conn_id) = entity_conn_id {

            // Don't let players hit themselves or entities which are already dead on the server
            if entity_conn_id != conn_id && server_data.hp > 0 {

                // Ignore entities outside of beam range
                let distance = util::distance(
                    client_data.x, client_data.y, beam_line[0], beam_line[1]
                );

                if distance - PLAYER_RADIUS < l {
                    nearest_entities.push((
                        distance,
                        client_data.x, client_data.y,
                        *entity_conn_id
                    ));
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
        if let Some(intersection) = line_segment_intersect_circle(
            &beam_line,
            x, y,
            PLAYER_RADIUS
        ) {
            return Some((entity_conn_id, l - intersection[6]));
        }
    }

    None

}

