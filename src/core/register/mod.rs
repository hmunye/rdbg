//! Defines information on registers used by the debugger.

mod macros;
pub(crate) use macros::{
    debug_reg, debug_reg_offset, fp_reg, fp_reg_mm, fp_reg_offset, fp_reg_size, fp_reg_st,
    fp_reg_xmm, gp_reg_8_bit_h, gp_reg_8_bit_l, gp_reg_16_bit, gp_reg_32_bit, gp_reg_64_bit,
    gp_reg_offset,
};

mod register_info;
pub(crate) use register_info::{RegisterFormat, RegisterInfo, RegisterType};
