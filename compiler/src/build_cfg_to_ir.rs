use crate::{cfg_gen::CFG, codegen::CodeGen};
use inkwell::{basic_block::BasicBlock, values::AnyValue};
use types::instruction::Instruction;

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

        let rctr_addr = self.module.get_global("rctr").unwrap().as_pointer_value();

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
            let block_size = i64_type.const_int(node.get_region_size(), false);

            let color_block_start = self.get_basic_block_by_name(node.get_label()).unwrap();
            let rotate_pointers = self.context.insert_basic_block_after(
                color_block_start,
                &("rotate_pointers_".to_owned() + &node.get_label()),
            );
            self.builder.position_at_end(color_block_start);

            let global_dp = self.builder.build_load(dp_addr, "load_dp").into_int_value();
            let global_cc = self.builder.build_load(cc_addr, "load_cc").into_int_value();
            unsafe {
                self.builder.build_global_string(
                    &("Curr: ".to_owned() + node.get_label() + "\n"),
                    &(node.get_label().to_owned() + "curr"),
                );
            }

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
                let dirvec = adjs.get(adj).unwrap();
                let dirvec_blocks = (0..dirvec.len())
                    .map(|_| {
                        self.context
                            .insert_basic_block_after(adj_blocks[i], "dirvec_adj")
                    })
                    .collect::<Vec<_>>();

                // Build link to dirvec adjacency
                self.builder.position_at_end(adj_blocks[i]);
                self.builder.build_unconditional_branch(dirvec_blocks[0]);

                for (j, &((dp, cc), (new_dp, new_cc), instr)) in dirvec.iter().enumerate() {
                    let call_instr = self.context.insert_basic_block_after(
                        dirvec_blocks[j],
                        &("call_instr_".to_owned() + node.get_label().as_str()),
                    );

                    self.builder.position_at_end(dirvec_blocks[j]);

                    let dp_as_const = i8_type.const_int(dp as i8 as u64, false);
                    let cc_as_const = i8_type.const_int(cc as i8 as u64, false);

                    let dp_cmp = self.builder.build_int_compare(
                        inkwell::IntPredicate::EQ,
                        global_dp,
                        dp_as_const,
                        "",
                    );
                    let cc_cmp = self.builder.build_int_compare(
                        inkwell::IntPredicate::EQ,
                        global_cc,
                        cc_as_const,
                        "",
                    );
                    let and_dp_cc = self.builder.build_and(dp_cmp, cc_cmp, "");

                    if j + 1 < dirvec.len() {
                        self.builder.build_conditional_branch(
                            and_dp_cc,
                            call_instr,
                            dirvec_blocks[j + 1],
                        );
                    } else {
                        if i + 1 < adj_blocks.len() {
                            self.builder.build_conditional_branch(
                                and_dp_cc,
                                call_instr,
                                adj_blocks[i + 1],
                            );
                        } else {
                            self.builder.build_conditional_branch(
                                and_dp_cc,
                                call_instr,
                                rotate_pointers,
                            );
                        }
                    }

                    // Calls the correct instruction
                    self.builder.position_at_end(call_instr);
                    if let Some(instr) = instr {
                        // Rotate by n
                        let instr_fn = self.module.get_function(instr.to_llvm_name()).unwrap();
                        let instr_str_addr = self
                            .module
                            .get_global(&(instr.to_llvm_name().to_owned() + "_fmt"))
                            .unwrap()
                            .as_any_value_enum()
                            .into_pointer_value();

                        let _instr_str = unsafe {
                            self.builder
                                .build_gep(instr_str_addr, &[const_0, const_0], "")
                        };

                        if instr == Instruction::Push {
                            self.builder.build_call(instr_fn, &[block_size.into()], "");
                        } else {
                            self.builder.build_call(instr_fn, &[], "");
                        }
                    }

                    if instr.is_none() {
                        let new_dp_as_const = i8_type.const_int(new_dp as i8 as u64, false);
                        let new_cc_as_const = i8_type.const_int(new_cc as i8 as u64, false);
                        self.builder.build_store(dp_addr, new_dp_as_const);
                        self.builder.build_store(cc_addr, new_cc_as_const);
                    }
                    let const_0_i8 = self.builder.build_int_truncate(const_0, i8_type, "");
                    self.builder.build_store(rctr_addr, const_0_i8);
                    let next_block = self.get_basic_block_by_name(adj.get_label()).unwrap();
                    let _jmp_to_next = self.builder.build_unconditional_branch(next_block);
                }
            }
            // Rotates dp / cc and jumps to the beginning
            if adjs.len() > 0 {
                rotate_pointers.move_after(*adj_blocks.last().unwrap()).ok();
                self.builder.position_at_end(rotate_pointers);
                let _call_retry = self.builder.build_call(retry_fn, &[], "call_retry");
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
