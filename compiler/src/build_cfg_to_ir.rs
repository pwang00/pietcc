use crate::{cfg_gen::CFG, codegen::CodeGen};
use inkwell::{basic_block::BasicBlock, values::AnyValue};
use std::collections::HashMap;
use types::instruction::Instruction;

impl<'a, 'b> CodeGen<'a, 'b> {
    pub(crate) fn build_entry(&self, cfg: &CFG) {
        let i8_type = self.context.i8_type();
        let i64_type = self.context.i64_type();
        let void_type = self.context.void_type().fn_type(&[], false);
        let start_fn = self.module.add_function("start", void_type, None);
        let basic_block = self.context.append_basic_block(start_fn, "");

        let mut block_lookup_table = HashMap::<&str, BasicBlock>::new();

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
        // Generate all basic blocks
        for node in cfg.keys() {
            let block = self.context.append_basic_block(start_fn, &node.get_label());
            block_lookup_table.insert(node.get_label(), block);
        }

        let entry = block_lookup_table.get("Entry").unwrap().to_owned();
        let ret_block = self.context.append_basic_block(start_fn, "ret");

        // Init (jumps to entry block)
        self.builder.position_at_end(basic_block);
        let _ = self.builder.build_unconditional_branch(entry);

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
            let rotate_pointers = self.context.insert_basic_block_after(
                color_block_start,
                &("rotate_pointers_".to_owned() + &node.get_label()),
            );

            self.builder.position_at_end(color_block_start);

            let global_dp = self.builder.build_load(self.context.i64_type(), dp_addr, "load_dp").unwrap().into_int_value();
            let global_cc = self.builder.build_load(self.context.i64_type(), cc_addr, "load_cc").unwrap().into_int_value();

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

            if adj_blocks.len() > 0 {
                let _ = self.builder.build_unconditional_branch(adj_blocks[0]);
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
                let _ = self.builder.build_unconditional_branch(dirvec_blocks[0]);

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
                    ).unwrap();
                    let cc_cmp = self.builder.build_int_compare(
                        inkwell::IntPredicate::EQ,
                        global_cc,
                        cc_as_const,
                        "",
                    ).unwrap();
                    let and_dp_cc = self.builder.build_and(dp_cmp, cc_cmp, "").unwrap();

                    if j + 1 < dirvec.len() {
                        let _ = self.builder.build_conditional_branch(
                            and_dp_cc,
                            call_instr,
                            dirvec_blocks[j + 1],
                        );
                    } else {
                        if i + 1 < adj_blocks.len() {
                            let _ = self.builder.build_conditional_branch(
                                and_dp_cc,
                                call_instr,
                                adj_blocks[i + 1],
                            );
                        } else {
                            let _ = self.builder.build_conditional_branch(
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
                                .build_gep(instr_fn.as_global_value().as_pointer_value().get_type(), instr_str_addr, &[const_0, const_0], "")
                        };

                        if instr == Instruction::Push {
                            let _ = self.builder.build_call(instr_fn, &[block_size.into()], "");
                        } else {
                            let _ = self.builder.build_call(instr_fn, &[], "");
                        }
                    } else {
                        let new_dp_as_const = i8_type.const_int(new_dp as i8 as u64, false);
                        let new_cc_as_const = i8_type.const_int(new_cc as i8 as u64, false);
                        let _ = self.builder.build_store(dp_addr, new_dp_as_const);
                        let _ = self.builder.build_store(cc_addr, new_cc_as_const);
                    }

                    let const_0_i8 = self.builder.build_int_truncate(const_0, i8_type, "").unwrap();
                    let _ = self.builder.build_store(rctr_addr, const_0_i8);
                    let next_block = block_lookup_table
                        .get(adj.get_label() as &str)
                        .unwrap()
                        .to_owned();
                    let _jmp_to_next = self.builder.build_unconditional_branch(next_block);
                }
            }
            // Rotates dp / cc and jumps to the beginning
            if adjs.len() > 0 {
                rotate_pointers.move_after(*adj_blocks.last().unwrap()).ok();
                self.builder.position_at_end(rotate_pointers);
                let _call_retry = self.builder.build_call(retry_fn, &[], "call_retry");
                let _ = self.builder.build_unconditional_branch(color_block_start);
            } else {
                let _ = self.builder.build_unconditional_branch(ret_block);
                unsafe {
                    rotate_pointers.delete().ok();
                }
            }
        }
        // Ret
        ret_block
            .move_after(*block_lookup_table.values().last().unwrap())
            .ok();
        let _ = self.builder.position_at_end(ret_block);
        let _ = self.builder.build_return(None);
    }
}
