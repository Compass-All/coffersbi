#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum EnclaveId {
    Coffer,
    Host,
    Encl(u32),
}
