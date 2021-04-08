#[repr(u8)]
/// Supported compression schemes
/// Follows the format specified in IETF RFC 3749
pub enum Scheme {
    // TODO: should we support additional compression schemes?
    DEFLATE = 1,
}
