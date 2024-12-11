pub mod check_for_logic_loops;
pub mod check_for_unconnected_clock_reset;
pub mod check_for_undriven;
pub mod constant_buffer_elimination;
pub mod constant_propagation;
pub mod lower_any_with_single_argument;
pub mod lower_case_to_select;
pub mod lower_select_to_buffer;
pub mod lower_select_with_identical_args;
pub mod pass;
pub mod remove_and_with_constant;
pub mod remove_hardwired_selects;
pub mod remove_or_with_constant;
pub mod remove_orphan_constants;
pub mod remove_unused_buffers;
pub mod remove_useless_selects;
pub mod remove_zeros_from_any;
