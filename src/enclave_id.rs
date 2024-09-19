#[derive(Debug, Copy, Clone, PartialEq)]
pub enum EnclaveId {
    Coffer,
    Host,
    Encl(u32),
}
