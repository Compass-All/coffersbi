use sbi_spec::binary::SbiRet;

pub struct MessageCallFunc;

impl MessageCallFunc {
    
}

pub fn handle_message_call(function: usize, param: [usize; 7]) -> SbiRet {
    log::debug!("function: 0x{:x}", function);
    log::debug!("param: {:?}", param);
    log::debug!("param hex: {:x?}", param);

    match function {
        _ => SbiRet::not_supported(),
    }
}
