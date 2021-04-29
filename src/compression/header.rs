use crate::compression::scheme::Scheme;

#[derive(PartialEq, Debug, Copy, Clone)]
// TODO: compression is not supported in TLS 1.3, and rustls doesn't support compression at all.
//  Should we drop compression support from our project, or keep it in with warnings that it can
//  potentially leak information?
/// Custom compression header to indicate that the following content is compressed
pub struct Header {
    pub magic: u16,
    pub scheme: Scheme,
}

// TODO: what should the magic value be? How many bytes should it be?
/// Magic value that indicates this is the start of a compression Header record
pub const HEADER_MAGIC_VALUE: u16 = 0xbeef;

impl Header {
    pub fn new(scheme: Scheme) -> Header {
        Header {
            magic: HEADER_MAGIC_VALUE,
            scheme,
        }
    }

    pub fn to_bytes(&self) -> Option<Vec<u8>> {
        return Some(
            [
                &self.magic.to_be_bytes(),
                self.scheme.to_bytes()?.as_slice(),
            ]
            .concat(),
        );
    }

    pub fn from_bytes(buf: &[u8]) -> Option<Header> {
        let magic_bytes = HEADER_MAGIC_VALUE.to_be_bytes();
        if buf[..magic_bytes.len()] != magic_bytes {
            return None;
        }

        let scheme = Scheme::from_bytes(&buf[magic_bytes.len()..])?;
        Some(Header::new(scheme))
    }

    /// Size in bytes when serialized
    pub fn serialized_size() -> usize {
        // TODO: connect size to size of serialized HEADER_MAGIC_VALUE. Shouldn't require converting
        //  it to bytes first.
        2 + Scheme::serialized_size()
    }
}

#[cfg(test)]
mod tests {
    use crate::compression::header::{Header, HEADER_MAGIC_VALUE};
    use crate::compression::scheme::Scheme;

    #[test]
    fn header_to_bytes() {
        let scheme = Scheme::Deflate;
        let scheme_bytes = match scheme.to_bytes() {
            Some(data) => data,
            None => panic!("Could not convert scheme to bytes"),
        };
        let magic_bytes = &HEADER_MAGIC_VALUE.to_be_bytes();
        let expected_result: Vec<u8> = [magic_bytes, scheme_bytes.as_slice()].concat();
        assert_eq!(Header::new(scheme).to_bytes(), Some(expected_result));
    }

    #[test]
    fn header_from_bytes() {
        let scheme = Scheme::Deflate;
        let scheme_bytes = match scheme.to_bytes() {
            Some(data) => data,
            None => panic!("Could not convert scheme to bytes"),
        };
        let magic_bytes = &HEADER_MAGIC_VALUE.to_be_bytes();
        let bytes: Vec<u8> = [magic_bytes, scheme_bytes.as_slice()].concat();
        let expected_header = Header::new(scheme);
        assert_eq!(Header::from_bytes(&bytes), Some(expected_header));
    }
}
