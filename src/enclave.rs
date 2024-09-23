use alloc::vec::Vec;
use sbi_spec::binary::SbiRet;
use spin::{Mutex, Once, RwLock};
use crate::vcpu::VCpu;

struct Enclave {
    vcpus: Vec<VCpu>,
}

impl Enclave {
    pub fn new() -> Self {
        Enclave {
            vcpus: vec![VCpu::new()],
        }
    }
}

static ENCLAVE_VEC: Once<RwLock<Vec<Enclave>>> = Once::new();

pub(crate) fn coffer_sm_init() -> SbiRet {
    log::info!("Initializing CofferSBI Security Monitor");

    ENCLAVE_VEC.call_once(|| RwLock::new(Vec::new()));

    SbiRet::success(0)
}

pub(crate) fn coffer_sm_test() -> SbiRet {
    log::debug!("CofferSBI Security Monitor test");

    // test read
    {
        let enclaves = ENCLAVE_VEC.get().unwrap().read();
        log::debug!("Enclave count: {}", enclaves.len());
    }

    // test write
    {
        let mut enclaves = ENCLAVE_VEC.get().unwrap().write();
        enclaves.push(Enclave::new());
        log::debug!("Enclave count: {}", enclaves.len());
    }

    // test read
    {
        let enclaves = ENCLAVE_VEC.get().unwrap().read();
        log::debug!("Enclave count: {}", enclaves.len());
        
        // print a0..a2 of the first vcpu
        let vcpus = &enclaves[0].vcpus;
        log::debug!("a0: 0x{:x}", vcpus[0].gpr.a[0]);
        log::debug!("a1: 0x{:x}", vcpus[0].gpr.a[1]);
        log::debug!("a2: 0x{:x}", vcpus[0].gpr.a[2]);
    }

    SbiRet::success(0)
}
