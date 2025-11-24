use crate::lowering_ctx::LoweringCtx;
use inkwell::{basic_block::BasicBlock, values::AnyValue};
use piet_core::cfg::CFG;
use piet_core::instruction::Instruction;
use std::collections::HashMap;

pub(crate) fn build_transitions<'a, 'b>(ctx: &LoweringCtx<'a, 'b>, cfg: &CFG, entry_label: &str) {
    let i8_type = ctx.llvm_context.i8_type();
    let i64_type = ctx.llvm_context.i64_type();
    let start_fn = ctx.module.get_function("start").unwrap_or_else(|| {
        ctx.module.add_function(
            "start",
            ctx.llvm_context.void_type().fn_type(&[], false),
            None,
        )
    });
    let basic_block = ctx.llvm_context.append_basic_block(start_fn, "");

    let mut block_lookup_table = HashMap::<&str, BasicBlock>::new();

    ctx.builder.position_at_end(basic_block);
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
    // Generate all basic blocks
    for node in cfg.keys() {
        let block = ctx
            .llvm_context
            .append_basic_block(start_fn, &node.get_label());
        block_lookup_table.insert(node.get_label(), block);
    }
    let entry = block_lookup_table.get(entry_label).unwrap().to_owned();
    let ret_block = ctx.llvm_context.append_basic_block(start_fn, "ret");

    // Init (jumps to entry block)
    ctx.builder.position_at_end(basic_block);
    let _ = ctx.builder.build_unconditional_branch(entry);

    // For every node, we want to get its adjacencies and generate the correct instructions depending on DP / CC
    // We essentially want an if / elif chain of different dp / cc cases.  If the dp or cc fall through then we
    // increment the retries counter until we find one that matches.
    for node in cfg.keys() {
        let adjs = cfg.get(node).unwrap();
        let block_size = i64_type.const_int(node.get_region_size(), false);

        let color_block_start = block_lookup_table
            .get(&node.get_label() as &str)
            .unwrap()
            .to_owned();
        let rotate_pointers = ctx.llvm_context.insert_basic_block_after(
            color_block_start,
            &("rotate_pointers_".to_owned() + &node.get_label()),
        );

        ctx.builder.position_at_end(color_block_start);

        let global_dp = ctx
            .builder
            .build_load(ctx.llvm_context.i64_type(), dp_addr, "load_dp")
            .unwrap()
            .into_int_value();
        let global_cc = ctx
            .builder
            .build_load(ctx.llvm_context.i64_type(), cc_addr, "load_cc")
            .unwrap()
            .into_int_value();

        let adj_blocks = adjs
            .keys()
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
            let _ = ctx.builder.build_unconditional_branch(adj_blocks[0]);
        }

        for (i, adj) in adjs.keys().enumerate() {
            let dirvec = adjs.get(adj).unwrap();
            let dirvec_blocks = (0..dirvec.len())
                .map(|_| {
                    ctx.llvm_context
                        .insert_basic_block_after(adj_blocks[i], "dirvec_adj")
                })
                .collect::<Vec<_>>();

            // Build link to dirvec adjacency
            ctx.builder.position_at_end(adj_blocks[i]);
            let _ = ctx.builder.build_unconditional_branch(dirvec_blocks[0]);

            for (j, transition) in dirvec.iter().enumerate() {
                let call_instr = ctx.llvm_context.insert_basic_block_after(
                    dirvec_blocks[j],
                    &("call_instr_".to_owned() + node.get_label().as_str()),
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
                    let _ = ctx.builder.build_conditional_branch(
                        and_dp_cc,
                        call_instr,
                        dirvec_blocks[j + 1],
                    );
                } else {
                    if i + 1 < adj_blocks.len() {
                        let _ = ctx.builder.build_conditional_branch(
                            and_dp_cc,
                            call_instr,
                            adj_blocks[i + 1],
                        );
                    } else {
                        let _ = ctx.builder.build_conditional_branch(
                            and_dp_cc,
                            call_instr,
                            rotate_pointers,
                        );
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

                    if instr == Instruction::Push {
                        let _ = ctx.builder.build_call(instr_fn, &[block_size.into()], "");
                    } else {
                        let _ = ctx.builder.build_call(instr_fn, &[], "");
                    }
                } else {
                    let new_dp_as_const =
                        i8_type.const_int(transition.exit_state.dp as i8 as u64, false);
                    let new_cc_as_const =
                        i8_type.const_int(transition.exit_state.cc as i8 as u64, false);
                    let _ = ctx.builder.build_store(dp_addr, new_dp_as_const);
                    let _ = ctx.builder.build_store(cc_addr, new_cc_as_const);
                }

                let const_0_i8 = ctx
                    .builder
                    .build_int_truncate(const_0, i8_type, "")
                    .unwrap();
                let _ = ctx.builder.build_store(rctr_addr, const_0_i8);
                let next_block = block_lookup_table
                    .get(adj.get_label() as &str)
                    .unwrap()
                    .to_owned();
                let _jmp_to_next = ctx.builder.build_unconditional_branch(next_block);
            }
        }
        // Rotates dp / cc and jumps to the beginning
        if !adjs.is_empty() {
            rotate_pointers.move_after(*adj_blocks.last().unwrap()).ok();
            ctx.builder.position_at_end(rotate_pointers);
            let _call_retry = ctx.builder.build_call(retry_fn, &[], "call_retry");
            let _ = ctx.builder.build_unconditional_branch(color_block_start);
        } else {
            let _ = ctx.builder.build_unconditional_branch(ret_block);
            unsafe {
                rotate_pointers.delete().ok();
            }
        }
    }
    // Ret
    ret_block
        .move_after(*block_lookup_table.values().last().unwrap())
        .ok();
    let _ = ctx.builder.position_at_end(ret_block);
    let _ = ctx.builder.build_return(None);
}
