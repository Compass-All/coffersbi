use sbi_spec::binary::SbiRet;
use spin::Once;
use crate::{enclave::{coffer_sm_init, coffer_sm_test}, enclave_id::EnclaveId, memory::{coffer_mem_alloc, coffer_memory_init, coffer_memory_test}};

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
            coffer_init(param)
        },
        CofferCallFunc::COFFER_MEM_ALLOC => {
            // TODO: change this to use the correct enclave id
            let tmp_eid = EnclaveId::Host;
            coffer_mem_alloc(tmp_eid, param[0])
        },
        CofferCallFunc::COFFER_TEST => {
            coffer_test(param)
        },
        _ => SbiRet::not_supported(),
    }
}

static INITIALIZED: Once<()> = Once::new();

fn coffer_init(param: [usize; 7]) -> SbiRet {
    if let Some(_) = INITIALIZED.get() {
        log::warn!("CofferSBI has already been initialized");
        return SbiRet::invalid_param();
    } else {
        INITIALIZED.call_once(|| ());
    }

    let ret = coffer_memory_init(param[0], param[1]);
    if ret.is_err() {
        return ret;
    }
    let ret = coffer_sm_init();
    if ret.is_err() {
        return ret;
    }
    return SbiRet::success(0);
}

fn coffer_test(param: [usize; 7]) -> SbiRet {
    let test_id = param[0];

    match test_id {
        0 => coffer_memory_test(param[1]),
        1 => coffer_sm_test(),
        _ => SbiRet::not_supported(),
    }
}
