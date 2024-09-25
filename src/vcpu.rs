use core::fmt::{self, Formatter, Debug};

use fast_trap::FlowContext;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

impl Clone for VCpu {
    fn clone(&self) -> Self {
        VCpu {
            gpr: FlowContext {
                ra: self.gpr.ra,
                t: self.gpr.t,
                a: self.gpr.a,
                s: self.gpr.s,
                gp: self.gpr.gp,
                tp: self.gpr.tp,
                sp: self.gpr.sp,
                pc: self.gpr.pc,
            },
            fpr: self.fpr.clone(),
            scsr: self.scsr.clone(),
            mcsr: self.mcsr.clone(),
        }
    }
}

impl Debug for VCpu {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let gpr = FlowContext {
            ra: self.gpr.ra,
            t: self.gpr.t,
            a: self.gpr.a,
            s: self.gpr.s,
            gp: self.gpr.gp,
            tp: self.gpr.tp,
            sp: self.gpr.sp,
            pc: self.gpr.pc,
        };
        f.debug_struct("General Purpose Registers")
            .field("ra", &gpr.ra)
            .field("t", &gpr.t)
            .field("a", &gpr.a)
            .field("s", &gpr.s)
            .field("gp", &gpr.gp)
            .field("tp", &gpr.tp)
            .field("sp", &gpr.sp)
            .field("pc", &gpr.pc)
            .finish()?;
        f.debug_struct("Floating Point Registers")
            .field("fpr", &self.fpr)
            .finish()?;
        f.debug_struct("Supervisor CSR")
            .field("scsr", &self.scsr)
            .finish()?;
        f.debug_struct("Machine CSR")
            .field("mcsr", &self.mcsr)
            .finish()
    }
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