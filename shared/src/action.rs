// External Dependencies ------------------------------------------------------
use hexahydrate::NETWORK_BYTE_OFFSET;
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode, DecodingError};


// Network Actions ------------------------------------------------------------
#[derive(Debug, RustcEncodable, RustcDecodable)]
pub enum Action {
    FiredLaserBeam(u8, f32),
    // TODO use smaller values
    // TODO create helper for f32 <> u16
    CreateLaserBeam(u8, f32, f32, f32, u8)
}

impl Action {

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![NETWORK_BYTE_OFFSET];
        bytes.append(&mut encode(self, SizeLimit::Infinite).unwrap());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Action, DecodingError> {
        if bytes[0] == NETWORK_BYTE_OFFSET {
            decode::<Action>(&bytes[1..])

        } else {
            Err(DecodingError::SizeLimit)
        }
    }

}

