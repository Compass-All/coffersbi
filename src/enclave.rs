use alloc::vec::Vec;
use sbi_spec::binary::SbiRet;
use spin::{Once, RwLock};
use crate::vcpu::VCpu;

struct Enclave {
    vcpus: Vec<VCpu>,
}

impl Enclave {
    pub fn new() -> Self {
        let mut vcpus = Vec::with_capacity(8);
        vcpus.push(VCpu::new());
        Enclave {
            vcpus
        }
    }
}

type LockedEnclaveVec = RwLock<Vec<RwLock<Enclave>>>;

static ENCLAVE_VEC: Once<LockedEnclaveVec> = Once::new();

fn create_empty_enclave() {
    let mut enclaves = ENCLAVE_VEC.get().unwrap().write();
    enclaves.push(RwLock::new(Enclave::new()));
}


pub(crate) fn coffer_sm_init() -> SbiRet {
    log::info!("Initializing CofferSBI Security Monitor");

    ENCLAVE_VEC.call_once(|| RwLock::new(Vec::with_capacity(1024)));

    SbiRet::success(0)
}

pub(crate) fn coffer_sm_test() -> SbiRet {
    log::debug!("CofferSBI Security Monitor test");

    // Test read
    {
        log::debug!("Test 1: Read enclave count");
        let enclaves = ENCLAVE_VEC.get().unwrap().read();
        log::debug!("Enclave count: {}", enclaves.len());
    }

    // Test write (adding a new enclave)
    {
        log::debug!("Test 2: Create enclave");
        create_empty_enclave();
    }

    // Test write to an enclave
    {
        log::debug!("Test 3: Write to enclave 1");
        let enclaves = ENCLAVE_VEC.get().unwrap().read();
        let encl1 = &mut enclaves[0].write();
        encl1.vcpus[0].gpr.a[0] = 0x12345678;
    }

    // Test read
    {
        log::debug!("Test 4: Read enclave 1");
        let enclaves = ENCLAVE_VEC.get().unwrap().read();
        log::debug!("Enclave count: {}", enclaves.len());
        
        // Print a0..a2 of the first vCPU
        let encl1 = &enclaves[0].read();
        let vcpus = &encl1.vcpus;
        log::debug!("a0: 0x{:x}", vcpus[0].gpr.a[0]);
        log::debug!("a1: 0x{:x}", vcpus[0].gpr.a[1]);
        log::debug!("a2: 0x{:x}", vcpus[0].gpr.a[2]);
    }

    SbiRet::success(0)
}
