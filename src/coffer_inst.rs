use fast_trap::FastContext;

use crate::coffer_call::handle_coffer_call;

pub struct CofferInst;

impl CofferInst {
    pub const COFFER_CALL: usize = 0xC0FF_E0A7_usize;
    pub const ENCLAVE_CALL: usize = 0xC0FF_E1A7_usize;
    pub const HOST_CALL: usize = 0xC0FF_E2A7_usize;
    pub const MESSAGE_CALL: usize = 0xC0FF_E3A7_usize;
}

pub fn emulate_coffer_inst(inst: usize, ctx: &mut FastContext) {
    match inst {
        CofferInst::COFFER_CALL => {
            let [a0, a1, a2, a3, a4, a5, a6, a7] = ctx.regs().a;
            let ret = handle_coffer_call(a7, [a0, a1, a2, a3, a4, a5, a6]);
            ctx.regs().a = [ret.error, ret.value, a2, a3, a4, a5, a6, a7];
        },
        CofferInst::ENCLAVE_CALL => {
            todo!()
        },
        CofferInst::HOST_CALL => {
            todo!()
        },
        CofferInst::MESSAGE_CALL => {
            todo!()
        },
        _ => panic!("Invalid CofferSBI instruction!"),
    }
}

