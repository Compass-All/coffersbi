use sbi_spec::binary::SbiRet;
use crate::memory::{coffer_memory_init, coffer_memory_test};

pub struct CofferCallFunc;

impl CofferCallFunc {
    pub const COFFER_INIT: usize = 0x0_usize;
    pub const COFFER_TEST: usize = 0x1000_usize;
}

pub fn handle_coffer_call(function: usize, param: [usize; 7]) -> SbiRet {
    log::debug!("function: 0x{:x}", function);
    log::debug!("param: {:?}", param);
    log::debug!("param hex: {:x?}", param);

    match function {
        CofferCallFunc::COFFER_INIT => {
            coffer_memory_init(param[0], param[1])
        },
        CofferCallFunc::COFFER_TEST => {
            coffer_memory_test()
        },
        _ => SbiRet::not_supported(),
    }
}
