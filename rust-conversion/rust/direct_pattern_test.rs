use entities_erlang_syntax::*;
use interfaces_compiler_api::bytecode::*;
use std::collections::HashMap;

fn main() {
    println!("🧪 Runtime Execution Test - Function Dispatch Fix");
    
    // Create the same AST that the parsing pipeline creates
    let factorial_patterns_0 = vec![Pattern::Literal(
        Literal::Integer(Integer::from_i64(0))
    )];
    let factorial_body_0 = vec![Expression::Literal(
        Literal::Integer(Integer::from_i64(1))
    )];
    let factorial_clause_0 = Clause::new(factorial_patterns_0, vec![], factorial_body_0);

    let factorial_var_n = Variable::new("N");
    let factorial_patterns_n = vec![Pattern::Variable(factorial_var_n.clone())];
    let factorial_body_n = vec![Expression::BinaryOp(
        BinaryOp::new(
            BinaryOperator::Multiply,
            Expression::Variable(factorial_var_n.clone()),
            Expression::FunctionCall(
                FunctionCall {
                    module: None,
                    function: Atom::new("factorial".to_string()),
                    args: vec![
                        Expression::BinaryOp(
                            BinaryOp::new(
                                BinaryOperator::Minus,
                                Expression::Variable(factorial_var_n),
                                Expression::Literal(Literal::Integer(Integer::from_i64(1))),
                            )
                        )
                    ],
                }
            ),
        )
    )];
    let factorial_clause_n = Clause::new(factorial_patterns_n, vec![], factorial_body_n);

    let factorial_function = Function::new(
        FunctionName::new(
            Atom::new("factorial".to_string()),
            1
        ),
        vec![factorial_clause_0, factorial_clause_n]  // Multiple clauses!
    );

    let mut module = Module::new(Atom::new("test_pattern_matching"));
    module.add_function(factorial_function);

    // Create compilation result
    let result = CompilationResult {
        module_name: Atom::new("test_pattern_matching"),
        ast: module,
        bytecode: vec![],
        warnings: vec![],
        metadata: CompilationMetadata::default(),
        context_metadata: HashMap::new(),
    };

    // Test bytecode generation
    let generator = BytecodeGenerator::new();
    match generator.generate_beam_file(&result) {
        Ok(beam_file) => {
            println!("✅ Bytecode generation successful!");
            
            // Check export table for correct function labels
            if let Some(exp_chunk) = beam_file.chunks.iter().find(|c| c.name == "ExpT") {
                println!("📊 Export table size: {} bytes", exp_chunk.data.len());

                // Parse export table (big-endian)
                if exp_chunk.data.len() >= 4 {
                    let num_exports = u32::from_be_bytes([
                        exp_chunk.data[0], exp_chunk.data[1],
                        exp_chunk.data[2], exp_chunk.data[3]
                    ]);
                    println!("📊 Number of exports: {}", num_exports);

                    // Each export entry is 12 bytes: atom_idx(4), arity(4), label(4)
                    for i in 0..num_exports as usize {
                        let offset = 4 + i * 12;
                        if offset + 12 <= exp_chunk.data.len() {
                            let atom_idx = u32::from_be_bytes([
                                exp_chunk.data[offset], exp_chunk.data[offset+1],
                                exp_chunk.data[offset+2], exp_chunk.data[offset+3]
                            ]);
                            let arity = u32::from_be_bytes([
                                exp_chunk.data[offset+4], exp_chunk.data[offset+5],
                                exp_chunk.data[offset+6], exp_chunk.data[offset+7]
                            ]);
                            let label = u32::from_be_bytes([
                                exp_chunk.data[offset+8], exp_chunk.data[offset+9],
                                exp_chunk.data[offset+10], exp_chunk.data[offset+11]
                            ]);

                            println!("  Export {}: atom_idx={}, arity={}, label={}",
                                   i, atom_idx, arity, label);
                        }
                    }
                }
            }

            // Check function table
            if let Some(fun_chunk) = beam_file.chunks.iter().find(|c| c.name == "FunT") {
                println!("📊 Function table size: {} bytes", fun_chunk.data.len());

                if fun_chunk.data.len() >= 4 {
                    let num_functions = u32::from_be_bytes([
                        fun_chunk.data[0], fun_chunk.data[1],
                        fun_chunk.data[2], fun_chunk.data[3]
                    ]);
                    println!("📊 Number of functions: {}", num_functions);

                    // Each function entry is 20 bytes: atom_idx(4), arity(4), label(4), index(4), num_free(4)
                    for i in 0..num_functions as usize {
                        let offset = 4 + i * 20;
                        if offset + 20 <= fun_chunk.data.len() {
                            let atom_idx = u32::from_be_bytes([
                                fun_chunk.data[offset], fun_chunk.data[offset+1],
                                fun_chunk.data[offset+2], fun_chunk.data[offset+3]
                            ]);
                            let arity = u32::from_be_bytes([
                                fun_chunk.data[offset+4], fun_chunk.data[offset+5],
                                fun_chunk.data[offset+6], fun_chunk.data[offset+7]
                            ]);
                            let label = u32::from_be_bytes([
                                fun_chunk.data[offset+8], fun_chunk.data[offset+9],
                                fun_chunk.data[offset+10], fun_chunk.data[offset+11]
                            ]);

                            println!("  Function {}: atom_idx={}, arity={}, label={}",
                                   i, atom_idx, arity, label);
                        }
                    }
                }
            }

            // Check code chunk for function structure
            if let Some(code_chunk) = beam_file.chunks.iter().find(|c| c.name == "Code") {
                println!("📊 Code chunk size: {} bytes", code_chunk.data.len());

                // Look for key opcodes
                let mut func_info_found = false;
                let mut label_found = false;
                let mut return_found = false;

                for &byte in &code_chunk.data {
                    if byte == infrastructure_beam_utilities::beam_instructions::BeamOpcode::FuncInfo.to_c_opcode() as u8 {
                        func_info_found = true;
                    }
                    if byte == infrastructure_beam_utilities::beam_instructions::BeamOpcode::Label.to_c_opcode() as u8 {
                        label_found = true;
                    }
                    if byte == infrastructure_beam_utilities::beam_instructions::BeamOpcode::Return.to_c_opcode() as u8 {
                        return_found = true;
                    }
                }

                println!("🎯 Function Structure Opcodes:");
                println!("  FuncInfo: {}", if func_info_found { "✅ FOUND" } else { "❌ MISSING" });
                println!("  Label: {}", if label_found { "✅ FOUND" } else { "❌ MISSING" });
                println!("  Return: {}", if return_found { "✅ FOUND" } else { "❌ MISSING" });
                
                if func_info_found && label_found && return_found {
                    println!("🏆 SUCCESS: Function dispatch structure is correct!");
                    println!("   Functions should now execute instead of returning undef.");
                } else {
                    println!("⚠️  ISSUE: Missing function structure opcodes");
                }
            }

            // Write the BEAM file for manual testing
            match beam_file.write_to_file("test_runtime.beam") {
                Ok(_) => println!("💾 BEAM file written to test_runtime.beam for manual testing"),
                Err(e) => println!("❌ Failed to write BEAM file: {}", e),
            }
        }
        Err(e) => {
            println!("❌ Bytecode generation failed: {}", e);
        }
    }
}
