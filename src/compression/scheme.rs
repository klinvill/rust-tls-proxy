use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

#[repr(u8)]
#[derive(FromPrimitive, ToPrimitive, PartialEq, Debug, Copy, Clone)]
/// Supported compression schemes
/// Follows the format specified in IETF RFC 3749
pub enum Scheme {
    // TODO: should we support additional compression schemes?
    DEFLATE = 1,
}

impl Scheme {
    pub fn to_bytes(&self) -> Option<Vec<u8>> {
        return Some(self.to_u8()?.to_be_bytes().to_vec());
    }

    pub fn from_bytes(buffer: &[u8]) -> Option<Scheme> {
        return match buffer.len() {
            0 => None,
            _ => Scheme::from_u8(buffer[0]),
        };
    }
}


#[cfg(test)]
mod tests {
    use crate::compression::scheme::Scheme;

    #[test]
    fn scheme_to_bytes() {
        let expected_bytes: Vec<u8> = vec![0x01];
        assert_eq!(Scheme::DEFLATE.to_bytes(), Some(expected_bytes));
    }

    #[test]
    fn scheme_from_bytes() {
        let bytes: [u8; 1] = [0x01];
        let expected_scheme = Scheme::DEFLATE;
        assert_eq!(Scheme::from_bytes(&bytes), Some(expected_scheme));
    }
}
