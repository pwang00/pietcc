use crate::lowering_ctx::LoweringCtx;
use inkwell::values::AnyValue;
use piet_core::cfg::{BlockId, CFG};
use piet_core::instruction::Instruction;

/// Lowers the graph into the `start` function: one basic block per color block, each dispatching
/// on the global DP / CC to find the exit it should take and falling through to `retry` when no
/// exit matches.
///
/// `entry` is the block execution resumes from, which is `CFG::ENTRY` for a full compile but the
/// block static evaluation stopped at when compiling a partial result.
pub(crate) fn build_transitions<'a, 'b>(ctx: &LoweringCtx<'a, 'b>, cfg: &CFG, entry: BlockId) {
    let i8_type = ctx.llvm_context.i8_type();
    let i64_type = ctx.llvm_context.i64_type();
    let start_fn = ctx.module.get_function("start").unwrap();
    let start_adj_basic_block = ctx.llvm_context.append_basic_block(start_fn, "");

    ctx.builder.position_at_end(start_adj_basic_block);
    // Globals
    let dp_addr = ctx
        .module
        .get_global("dp")
        .unwrap()
        .as_any_value_enum()
        .into_pointer_value();
    let cc_addr = ctx
        .module
        .get_global("cc")
        .unwrap()
        .as_any_value_enum()
        .into_pointer_value();

    let rctr_addr = ctx.module.get_global("rctr").unwrap().as_pointer_value();

    // Constants
    let const_0 = i64_type.const_zero();
    // Functions
    let retry_fn = ctx.module.get_function("retry").unwrap();
    // Generate all basic blocks.  Ids are dense and walked in order, so the layout of the
    // emitted module is reproducible rather than dependent on hash iteration order.
    let basic_blocks = cfg
        .ids()
        .map(|id| {
            ctx.llvm_context
                .append_basic_block(start_fn, &cfg.block(id).label)
        })
        .collect::<Vec<_>>();

    let ret_block = ctx.llvm_context.append_basic_block(start_fn, "ret");

    // Init (jumps to entry block)
    ctx.builder.position_at_end(start_adj_basic_block);
    ctx.builder
        .build_unconditional_branch(basic_blocks[entry.index()])
        .unwrap();

    // For every node, we want to get its adjacencies and generate the correct instructions depending on DP / CC
    // We essentially want an if / elif chain of different dp / cc cases.  If the dp or cc fall through then we
    // increment the retries counter until we find one that matches.
    for node in cfg.ids() {
        let adjs = cfg.adjacencies(node);
        let label = &cfg.block(node).label;
        let block_size = i64_type.const_int(cfg.block(node).count as u64, false);

        let color_block_start = basic_blocks[node.index()];
        let rotate_pointers = ctx
            .llvm_context
            .insert_basic_block_after(color_block_start, &("rotate_pointers_".to_owned() + label));

        ctx.builder.position_at_end(color_block_start);

        let global_dp = ctx
            .builder
            .build_load(ctx.llvm_context.i8_type(), dp_addr, "load_dp")
            .unwrap()
            .into_int_value();
        let global_cc = ctx
            .builder
            .build_load(ctx.llvm_context.i8_type(), cc_addr, "load_cc")
            .unwrap()
            .into_int_value();

        let adj_blocks = adjs
            .iter()
            .enumerate()
            .map(|(i, _)| {
                ctx.llvm_context.insert_basic_block_after(
                    color_block_start,
                    &("adjacency_".to_owned()
                        + &color_block_start.get_name().to_string_lossy()
                        + "_"
                        + &i.to_string()),
                )
            })
            .collect::<Vec<_>>();

        if !adj_blocks.is_empty() {
            ctx.builder
                .build_unconditional_branch(adj_blocks[0])
                .unwrap();
        }

        for (i, (adj, dirvec)) in adjs.iter().enumerate() {
            let dirvec_blocks = (0..dirvec.len())
                .map(|_| {
                    ctx.llvm_context
                        .insert_basic_block_after(adj_blocks[i], "dirvec_adj")
                })
                .collect::<Vec<_>>();

            // Build link to dirvec adjacency
            ctx.builder.position_at_end(adj_blocks[i]);
            ctx.builder
                .build_unconditional_branch(dirvec_blocks[0])
                .unwrap();

            for (j, transition) in dirvec.iter().enumerate() {
                let call_instr = ctx.llvm_context.insert_basic_block_after(
                    dirvec_blocks[j],
                    &("call_instr_".to_owned() + label.as_str()),
                );

                ctx.builder.position_at_end(dirvec_blocks[j]);

                let dp_as_const = i8_type.const_int(transition.entry_state.dp as i8 as u64, false);
                let cc_as_const = i8_type.const_int(transition.entry_state.cc as i8 as u64, false);

                let dp_cmp = ctx
                    .builder
                    .build_int_compare(inkwell::IntPredicate::EQ, global_dp, dp_as_const, "")
                    .unwrap();
                let cc_cmp = ctx
                    .builder
                    .build_int_compare(inkwell::IntPredicate::EQ, global_cc, cc_as_const, "")
                    .unwrap();
                let and_dp_cc = ctx.builder.build_and(dp_cmp, cc_cmp, "").unwrap();

                if j + 1 < dirvec.len() {
                    ctx.builder
                        .build_conditional_branch(and_dp_cc, call_instr, dirvec_blocks[j + 1])
                        .unwrap();
                } else {
                    if i + 1 < adj_blocks.len() {
                        ctx.builder
                            .build_conditional_branch(and_dp_cc, call_instr, adj_blocks[i + 1])
                            .unwrap();
                    } else {
                        ctx.builder
                            .build_conditional_branch(and_dp_cc, call_instr, rotate_pointers)
                            .unwrap();
                    }
                }

                // Calls the correct instruction
                ctx.builder.position_at_end(call_instr);
                if let Some(instr) = transition.instruction {
                    // Rotate by n
                    let instr_fn = ctx.module.get_function(instr.to_llvm_name()).unwrap();
                    let instr_str_addr = ctx
                        .module
                        .get_global(&(instr.to_llvm_name().to_owned() + "_fmt"))
                        .unwrap()
                        .as_any_value_enum()
                        .into_pointer_value();

                    let _instr_str = unsafe {
                        ctx.builder.build_gep(
                            instr_fn.as_global_value().as_pointer_value().get_type(),
                            instr_str_addr,
                            &[const_0, const_0],
                            "",
                        )
                    };

                    if matches!(instr, Instruction::Push) {
                        ctx.builder
                            .build_call(instr_fn, &[block_size.into()], "")
                            .unwrap();
                    } else {
                        ctx.builder.build_call(instr_fn, &[], "").unwrap();
                    }
                } else {
                    let new_dp_as_const =
                        i8_type.const_int(transition.exit_state.dp as i8 as u64, false);
                    let new_cc_as_const =
                        i8_type.const_int(transition.exit_state.cc as i8 as u64, false);
                    ctx.builder.build_store(dp_addr, new_dp_as_const).unwrap();
                    ctx.builder.build_store(cc_addr, new_cc_as_const).unwrap();
                }

                let const_0_i8 = ctx
                    .builder
                    .build_int_truncate(const_0, i8_type, "")
                    .unwrap();
                ctx.builder.build_store(rctr_addr, const_0_i8).unwrap();
                let _jmp_to_next = ctx
                    .builder
                    .build_unconditional_branch(basic_blocks[adj.index()]);
            }
        }
        // Rotates dp / cc and jumps to the beginning
        if !adjs.is_empty() {
            rotate_pointers.move_after(*adj_blocks.last().unwrap()).ok();
            ctx.builder.position_at_end(rotate_pointers);
            let _call_retry = ctx.builder.build_call(retry_fn, &[], "call_retry");
            ctx.builder
                .build_unconditional_branch(color_block_start)
                .unwrap();
        } else {
            ctx.builder.build_unconditional_branch(ret_block).unwrap();
            unsafe {
                rotate_pointers.delete().unwrap();
            }
        }
    }
    // Ret
    if let Some(last) = basic_blocks.last() {
        ret_block.move_after(*last).ok();
    }
    ctx.builder.position_at_end(ret_block);
    ctx.builder.build_return(None).unwrap();
}
