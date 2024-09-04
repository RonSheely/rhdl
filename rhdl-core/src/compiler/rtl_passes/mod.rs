pub(crate) mod dead_code_elimination;
pub(crate) mod lower_empty_splice_to_copy;
pub(crate) mod lower_index_all_to_copy;
pub(crate) mod lower_multiply_to_shift;
pub(crate) mod lower_signal_casts;
pub(crate) mod lower_single_concat_to_copy;
pub(crate) mod pass;
pub(crate) mod remove_extra_registers;
pub(crate) mod remove_unused_operands;
pub(crate) mod strip_empty_args_from_concat;
pub(crate) mod symbol_table_is_complete;
