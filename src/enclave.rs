use alloc::vec::Vec;
use sbi_spec::binary::SbiRet;
use spin::{Once, RwLock};
use crate::vcpu::VCpu;

// Remember to change this value together with RustSBI's prototyper/src/trap_stack.rs
const NUM_HART_MAX: usize = 8;

struct Enclave {
    vcpus: Vec<VCpu>,
}

impl Enclave {
    pub fn new(start_pc: usize) -> Self {
        let mut vcpus = Vec::with_capacity(NUM_HART_MAX);
        vcpus.push(VCpu::init_context(start_pc));
        Enclave {
            vcpus
        }
    }
}

type LockedEnclaveVec = RwLock<Vec<RwLock<Enclave>>>;

static ENCLAVE_VEC: Once<LockedEnclaveVec> = Once::new();

fn create_empty_enclave() {
    // randomly chosen. To be replaced with an allocated address
    let start_pc = 0x1_4000_0000_usize;

    let mut enclaves = ENCLAVE_VEC.get().unwrap().write();
    enclaves.push(RwLock::new(Enclave::new(start_pc)));
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
        log::debug!("vcpu 0:\n{:#?}", vcpus[0]);
    }

    // vcpu save and load context test
    {
        let enclaves = ENCLAVE_VEC.get().unwrap().read();
        let encl1 = &mut enclaves[0].write();  // write lock, or it will raise "cannot borrow as mutable"
        encl1.vcpus[0].save_context();
        encl1.vcpus[0].gpr.a[0] = 0x98765432;
        encl1.vcpus[0].load_context();
        let vcpus = &encl1.vcpus;
        log::debug!("vcpu 0:\n{:#?}", vcpus[0]);
    }


    log::debug!("CofferSBI Security Monitor test passed.");
    SbiRet::success(0)
}
