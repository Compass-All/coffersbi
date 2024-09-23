use sbi_spec::binary::SbiRet;
use crate::{enclave_id::EnclaveId, memory::{coffer_mem_alloc, coffer_memory_init, coffer_memory_test}};

pub struct CofferCallFunc;

impl CofferCallFunc {
    const COFFER_INIT: usize = 0x0_usize;
    const COFFER_MEM_ALLOC: usize = 0x1_usize;
    const COFFER_TEST: usize = 0x1000_usize;
}

pub(crate) fn handle_coffer_call(function: usize, param: [usize; 7]) -> SbiRet {
    log::debug!("function: 0x{:x}", function);
    log::debug!("param: {:?}", param);
    log::debug!("param hex: {:x?}", param);

    match function {
        CofferCallFunc::COFFER_INIT => {
            coffer_memory_init(param[0], param[1])
        },
        CofferCallFunc::COFFER_MEM_ALLOC => {
            // TODO: change this to use the correct enclave id
            let tmp_eid = EnclaveId::Host;
            coffer_mem_alloc(tmp_eid, param[0])
        },
        CofferCallFunc::COFFER_TEST => {
            coffer_memory_test(param[0])
        },
        _ => SbiRet::not_supported(),
    }
}
