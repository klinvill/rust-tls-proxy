use crate::compression::scheme::Scheme;

// TODO: compression is not supported in TLS 1.3, and rustls doesn't support compression at all.
//  Should we drop compression support from our project, or keep it in with warnings that it can
//  potentially leak information?
/// Custom compression header to indicate that the following content is compressed
pub struct Header {
    pub magic: u8,
    pub scheme: Scheme,
}

// TODO: what should the magic value be? How many bytes should it be?
/// Magic value that indicates this is the start of a compression Header record
const HEADER_MAGIC_VALUE: u8 = 0x12;

impl Header {
    pub fn new(scheme: Scheme) -> Header {
       Header {
           magic: HEADER_MAGIC_VALUE,
           scheme,
       }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        unimplemented!("Serialization not implemented yet");
    }

    // TODO: should create an error type rather than using String
    fn from_bytes(buf: &[u8]) -> Result<Header, String> {
        unimplemented!("Deserialization not implemented yet")
    }
}
