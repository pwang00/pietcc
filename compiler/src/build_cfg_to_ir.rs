use inkwell::{basic_block::BasicBlock, values::AnyValue, IntPredicate};
use parser::decode::DecodeInstruction;
use types::instruction::{self, Instruction};

use crate::{
    cfg_gen::{Node, CFG},
    codegen::CodeGen,
};

impl<'a, 'b> CodeGen<'a, 'b> {
    fn get_basic_block_by_name(&self, name: &str) -> Option<BasicBlock<'b>> {
        let func = self.module.get_function("start").unwrap();
        let mut curr_block = func.get_first_basic_block();

        while let Some(block) = curr_block {
            if block.get_name().to_str().unwrap() == name {
                return Some(block);
            }
            curr_block = block.get_next_basic_block();
        }
        return None;
    }

    pub(crate) fn build_entry(&self, cfg: &CFG) {
        let i8_type = self.context.i8_type();
        let i64_type = self.context.i64_type();
        let void_type = self.context.void_type().fn_type(&[], false);
        let start_fn = self.module.add_function("start", void_type, None);
        let basic_block = self.context.append_basic_block(start_fn, "");
        let printf_fn = self.module.get_function("printf").unwrap();
        let print_stack_fn = self.module.get_function("print_piet_stack").unwrap();
        let print_pointers_fn = self.module.get_function("print_pointers").unwrap();
        self.builder.position_at_end(basic_block);
        // Globals
        let dp_addr = self
            .module
            .get_global("dp")
            .unwrap()
            .as_any_value_enum()
            .into_pointer_value();
        let cc_addr = self
            .module
            .get_global("cc")
            .unwrap()
            .as_any_value_enum()
            .into_pointer_value();

        // Constants
        let const_0 = i64_type.const_zero();

        // Functions
        let retry_fn = self.module.get_function("retry").unwrap();

        let mut vector_of_blocks = Vec::new();
        // Generate all basic blocks
        for node in cfg.keys() {
            vector_of_blocks.push(self.context.append_basic_block(start_fn, &node.get_label()));
        }

        let entry = self.get_basic_block_by_name("Entry").unwrap();
        let ret_block = self.context.append_basic_block(start_fn, "ret");

        // Init (jumps to entry block)
        self.builder.position_at_end(basic_block);
        self.builder.build_unconditional_branch(entry);

        // For every node, we want to get its adjacencies and generate the correct instructions depending on DP / CC
        // We essentially want an if / elif chain of different dp / cc cases.  If the dp or cc fall through then we
        // increment the retries counter until we find one that matches.
        for node in cfg.keys() {
            let adjs = cfg.get(node).unwrap();
            let curr_lightness = node.get_lightness();
            let block_size = i64_type.const_int(node.get_region_size(), false);

            let color_block_start = self.get_basic_block_by_name(node.get_label()).unwrap();
            let rotate_pointers = self.context.insert_basic_block_after(
                color_block_start,
                &("rotate_pointers_".to_owned() + &node.get_label()),
            );
            self.builder.position_at_end(color_block_start);

            let global_dp = self.builder.build_load(dp_addr, "").into_int_value();
            let global_cc = self.builder.build_load(cc_addr, "").into_int_value();

            let adj_blocks = adjs
                .keys()
                .enumerate()
                .map(|(i, _)| {
                    self.context.insert_basic_block_after(
                        color_block_start,
                        &("adjacency_".to_owned()
                            + &color_block_start.get_name().to_string_lossy()
                            + "_"
                            + &i.to_string()),
                    )
                })
                .collect::<Vec<_>>();

            let first_adjacency_name =
                &("adjacency_".to_owned() + &color_block_start.get_name().to_string_lossy() + "_0");

            if adj_blocks.len() > 0 {
                let first_adjacency = self.get_basic_block_by_name(&first_adjacency_name);
                self.builder
                    .build_unconditional_branch(first_adjacency.unwrap());
            }

            for (i, adj) in adjs.keys().enumerate() {
                let adj_lightness = adj.get_lightness();
                let instr =
                    <Self as DecodeInstruction>::decode_instr(curr_lightness, adj_lightness);

                let dirvec = adjs.get(adj).unwrap();

                // Check directions
                let mut accumulated_cmp = None;

                self.builder.position_at_end(adj_blocks[i]);

                let call_instr = self.context.insert_basic_block_after(
                    adj_blocks[i],
                    &("call_instr_".to_owned() + node.get_label().as_str()),
                );

                for (&(dp, cc), &(next_dp, next_cc)) in dirvec.iter().zip(dirvec[1..].iter()) {
                    let curr_dp_as_llvm_const = i8_type.const_int(dp as i8 as u64, false);
                    let curr_cc_as_llvm_const = i8_type.const_int(cc as i8 as u64, false);
                    let next_dp_as_llvm_const = i8_type.const_int(next_dp as i8 as u64, false);
                    let next_cc_as_llvm_const = i8_type.const_int(next_cc as i8 as u64, false);
                    let curr_dp_cmp = self.builder.build_int_compare(
                        IntPredicate::EQ,
                        curr_dp_as_llvm_const,
                        global_dp,
                        "",
                    );
                    let curr_cc_cmp = self.builder.build_int_compare(
                        IntPredicate::EQ,
                        curr_cc_as_llvm_const,
                        global_cc,
                        "",
                    );
                    let next_dp_cmp = self.builder.build_int_compare(
                        IntPredicate::EQ,
                        next_dp_as_llvm_const,
                        global_dp,
                        "",
                    );
                    let next_cc_cmp = self.builder.build_int_compare(
                        IntPredicate::EQ,
                        next_cc_as_llvm_const,
                        global_cc,
                        "",
                    );
                    let and_curr_dp_cc = self.builder.build_and(curr_dp_cmp, curr_cc_cmp, "");
                    let and_next_dp_cc = self.builder.build_and(next_dp_cmp, next_cc_cmp, "");
                    let or_curr_next = self.builder.build_or(and_curr_dp_cc, and_next_dp_cc, "");

                    match accumulated_cmp {
                        Some(prev_cmp) => {
                            accumulated_cmp =
                                Some(self.builder.build_or(prev_cmp, or_curr_next, ""));
                        }
                        None => {
                            accumulated_cmp = Some(or_curr_next);
                        }
                    }
                }

                let dp_as_llvm_const =
                    i8_type.const_int(dirvec.last().unwrap().0 as i8 as u64, false);
                let cc_as_llvm_const =
                    i8_type.const_int(dirvec.last().unwrap().1 as i8 as u64, false);
                let dp_cmp = self.builder.build_int_compare(
                    IntPredicate::EQ,
                    dp_as_llvm_const,
                    global_dp,
                    "",
                );
                let cc_cmp = self.builder.build_int_compare(
                    IntPredicate::EQ,
                    cc_as_llvm_const,
                    global_cc,
                    "",
                );
                let and = self.builder.build_and(dp_cmp, cc_cmp, "");
                let mut final_cmp = and;
                if let Some(cmp) = accumulated_cmp {
                    final_cmp = self.builder.build_or(cmp, and, "");
                }

                // If the index is less than length - 1 then
                if i + 1 < adj_blocks.len() {
                    self.builder
                        .build_conditional_branch(final_cmp, call_instr, adj_blocks[i + 1]);
                } else {
                    self.builder
                        .build_conditional_branch(final_cmp, call_instr, rotate_pointers);
                }

                // Calls the correct instruction
                self.builder.position_at_end(call_instr);
                if let Some(instr) = instr {
                    let instr_fn = self.module.get_function(instr.to_llvm_name()).unwrap();
                    let instr_str_addr = self
                        .module
                        .get_global(&(instr.to_llvm_name().to_owned() + "_fmt"))
                        .unwrap()
                        .as_any_value_enum()
                        .into_pointer_value();

                    // let instr_str = unsafe { self.builder.build_gep(instr_str_addr, &[const_0, const_0], "") };
                    // self.builder.build_call(printf_fn, &[instr_str.into()], "");

                    if instr == Instruction::Push {
                        self.builder.build_call(instr_fn, &[block_size.into()], "");
                    } else {
                        self.builder.build_call(instr_fn, &[], "");
                    }
                    // self.builder.build_call(print_pointers_fn, &[], "");
                    // self.builder.build_call(print_stack_fn, &[], "");
                }
                let next_block = self.get_basic_block_by_name(adj.get_label()).unwrap();
                let jmp_to_next = self.builder.build_unconditional_branch(next_block);
            }
            // Rotates dp / cc and jumps to the beginning
            if adjs.len() > 0 {
                rotate_pointers.move_after(*adj_blocks.last().unwrap()).ok();
                self.builder.position_at_end(rotate_pointers);
                let call_retry = self.builder.build_call(retry_fn, &[], "call_retry");
                self.builder.build_unconditional_branch(color_block_start);
            } else {
                self.builder.build_unconditional_branch(ret_block);
                unsafe {
                    rotate_pointers.delete().ok();
                }
            }
        }
        // Ret
        ret_block.move_after(*vector_of_blocks.last().unwrap()).ok();
        self.builder.position_at_end(ret_block);
        self.builder.build_return(None);
    }
}
