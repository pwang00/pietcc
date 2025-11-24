use crate::{builder, lowering_ctx::LoweringCtx};
use piet_core::{cfg::CFG, instruction::Instruction, state::ExecutionState};

pub(crate) fn build_partial<'a, 'b>(
    ctx: &LoweringCtx<'a, 'b>,
    cfg: &mut CFG,
    execution_state: &ExecutionState,
) {
    builder::build_globals(ctx, execution_state);
    builder::build_stdout_unbuffered(ctx);
    builder::build_print_stack(ctx);
    builder::build_terminate(ctx);
    builder::build_stack_size_check(ctx);
    builder::build_binops(ctx, Instruction::Add);
    builder::build_binops(ctx, Instruction::Sub);
    builder::build_binops(ctx, Instruction::Div);
    builder::build_binops(ctx, Instruction::Mul);
    builder::build_binops(ctx, Instruction::Mod);
    builder::build_binops(ctx, Instruction::Gt);
    builder::build_input(ctx, Instruction::CharIn);
    builder::build_input(ctx, Instruction::IntIn);
    builder::build_output(ctx, Instruction::CharOut);
    builder::build_output(ctx, Instruction::IntOut);
    builder::build_roll(ctx);
    builder::build_dup(ctx);
    builder::build_push(ctx);
    builder::build_pop(ctx);
    builder::build_not(ctx);
    builder::build_switch(ctx);
    builder::build_rotate(ctx);
    builder::build_retry(ctx);
    builder::build_transitions(ctx, &cfg, &execution_state.prev_cb_label);
    builder::build_main(ctx);
}
