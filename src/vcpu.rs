use fast_trap::FlowContext;

struct SCsr {
    sstatus: u64,
    sscratch: u64,
    sepc: u64,
    stvec: u64,
    satp: u64,
    scause: u64,
    stval: u64,
    sip: u64,
    sie: u64,
}

struct MCsr {
    mstatus: u64,
    mepc: u64,
    mip: u64,
    mie: u64,
    medeleg: u64,
    mideleg: u64,
}

pub(crate) struct VCpu {
    pub(crate) gpr: FlowContext,
    fpr: [f64; 32], // Currently only 64-bit floating point registers are supported
    scsr: SCsr,
    mcsr: MCsr,
}

impl VCpu {
    pub fn new() -> Self {
        VCpu {
            gpr: FlowContext::ZERO,
            fpr: [0.0; 32],
            scsr: SCsr {
                sstatus: 0,
                sscratch: 0,
                sepc: 0,
                stvec: 0,
                satp: 0,
                scause: 0,
                stval: 0,
                sip: 0,
                sie: 0,
            },
            mcsr: MCsr {
                mstatus: 0,
                mepc: 0,
                mip: 0,
                mie: 0,
                medeleg: 0,
                mideleg: 0,
            },
        }
    }
}
