//! Erlang Evaluator (erl_eval equivalent)
//!
//! Evaluates abstract syntax trees (AST) into Erlang values. This is the third
//! step in evaluating Erlang expressions, after scanning (erl_scan) and parsing (erl_parse).
//! Based on erl_eval.erl from lib/stdlib.

use super::erl_parse::{Expr, BinOp, UnOp};
use entities_process::{Eterm, Process, ProcessId};
use entities_data_handling::term_hashing::Term;
use std::collections::HashMap;
use std::sync::Arc;

/// Get or create the "true" atom
fn get_true_atom() -> u32 {
    use super::atom_table::get_global_atom_table;
    use entities_data_handling::AtomEncoding;
    let atom_table = get_global_atom_table();
    atom_table.put_index(b"true", AtomEncoding::SevenBitAscii, false)
        .unwrap_or(1) as u32
}

/// Get or create the "false" atom
fn get_false_atom() -> u32 {
    use super::atom_table::get_global_atom_table;
    use entities_data_handling::AtomEncoding;
    let atom_table = get_global_atom_table();
    atom_table.put_index(b"false", AtomEncoding::SevenBitAscii, false)
        .unwrap_or(0) as u32
}

/// Evaluation error
#[derive(Debug, Clone, PartialEq)]
pub enum EvalError {
    UnboundVariable(String),
    UndefinedFunction { module: Option<String>, function: String, arity: usize },
    DivisionByZero,
    InvalidOperation(String),
    TypeError(String),
    FunctionCallError(String),
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EvalError::UnboundVariable(v) => write!(f, "Unbound variable: {}", v),
            EvalError::UndefinedFunction { module, function, arity } => {
                if let Some(m) = module {
                    write!(f, "Undefined function {}/{}", m, function)
                } else {
                    write!(f, "Undefined function {}/{}", function, arity)
                }
            }
            EvalError::DivisionByZero => write!(f, "Division by zero"),
            EvalError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            EvalError::TypeError(msg) => write!(f, "Type error: {}", msg),
            EvalError::FunctionCallError(msg) => write!(f, "Function call error: {}", msg),
        }
    }
}

impl std::error::Error for EvalError {}

/// Variable bindings
pub type Bindings = HashMap<String, Term>;

/// Evaluate a list of expressions
///
/// This is the main entry point for evaluating Erlang expressions.
///
/// # Arguments
/// * `exprs` - List of expressions to evaluate
/// * `bindings` - Variable bindings (can be empty)
///
/// # Returns
/// * `Ok((Term, Bindings))` - Result value and updated bindings
/// * `Err(EvalError)` - Evaluation error
pub fn exprs(exprs: Vec<Expr>, bindings: Bindings) -> Result<(Term, Bindings), EvalError> {
    let mut current_bindings = bindings;
    let mut last_value = Term::Nil;
    
    for expr in exprs {
        let (value, new_bindings) = expr_eval(&expr, &current_bindings)?;
        current_bindings = new_bindings;
        last_value = value;
    }
    
    Ok((last_value, current_bindings))
}

/// Evaluate a single expression
///
/// # Arguments
/// * `expr` - Expression to evaluate
/// * `bindings` - Variable bindings
///
/// # Returns
/// * `Ok((Term, Bindings))` - Result value and updated bindings
/// * `Err(EvalError)` - Evaluation error
pub fn expr(expr: &Expr, bindings: &Bindings) -> Result<(Term, Bindings), EvalError> {
    expr_eval(expr, bindings)
}

/// Internal expression evaluator
fn expr_eval(expr: &Expr, bindings: &Bindings) -> Result<(Term, Bindings), EvalError> {
    match expr {
        Expr::Integer(i) => Ok((Term::Small(*i), bindings.clone())),
        Expr::Float(f) => Ok((Term::Float(*f), bindings.clone())),
        Expr::Atom(s) => {
            // Convert atom string to atom index
            use super::atom_table::get_global_atom_table;
            use entities_data_handling::AtomEncoding;
            let atom_table = get_global_atom_table();
            let index = atom_table.put_index(s.as_bytes(), AtomEncoding::SevenBitAscii, false)
                .map_err(|_| EvalError::InvalidOperation(format!("Failed to create atom: {}", s)))?;
            Ok((Term::Atom(index as u32), bindings.clone()))
        }
        Expr::String(s) => {
            // Convert string to list of characters
            let chars: Vec<Term> = s.chars()
                .map(|c| Term::Small(c as i64))
                .collect();
            let list = chars.into_iter().rev().fold(Term::Nil, |acc, ch| {
                Term::List {
                    head: Box::new(ch),
                    tail: Box::new(acc),
                }
            });
            Ok((list, bindings.clone()))
        }
        Expr::Char(c) => Ok((Term::Small(*c as i64), bindings.clone())),
        Expr::Var(v) => {
            match bindings.get(v) {
                Some(term) => Ok((term.clone(), bindings.clone())),
                None => Err(EvalError::UnboundVariable(v.clone())),
            }
        }
        Expr::Nil => Ok((Term::Nil, bindings.clone())),
        Expr::Cons { head, tail } => {
            let (head_val, bindings1) = expr_eval(head, bindings)?;
            let (tail_val, bindings2) = expr_eval(tail, &bindings1)?;
            Ok((Term::List {
                head: Box::new(head_val),
                tail: Box::new(tail_val),
            }, bindings2))
        }
        Expr::List(elems) => {
            let mut list = Term::Nil;
            let mut current_bindings = bindings.clone();
            
            for elem in elems.iter().rev() {
                let (val, new_bindings) = expr_eval(elem, &current_bindings)?;
                list = Term::List {
                    head: Box::new(val),
                    tail: Box::new(list),
                };
                current_bindings = new_bindings;
            }
            
            Ok((list, current_bindings))
        }
        Expr::Tuple(elems) => {
            let mut tuple_elems = Vec::new();
            let mut current_bindings = bindings.clone();
            
            for elem in elems {
                let (val, new_bindings) = expr_eval(elem, &current_bindings)?;
                tuple_elems.push(val);
                current_bindings = new_bindings;
            }
            
            Ok((Term::Tuple(tuple_elems), current_bindings))
        }
        Expr::BinOp { op, left, right } => {
            let (left_val, bindings1) = expr_eval(left, bindings)?;
            let (right_val, bindings2) = expr_eval(right, &bindings1)?;
            let result = eval_binop(op, &left_val, &right_val)?;
            Ok((result, bindings2))
        }
        Expr::UnOp { op, expr } => {
            let (val, new_bindings) = expr_eval(expr, bindings)?;
            let result = eval_unop(op, &val)?;
            Ok((result, new_bindings))
        }
        Expr::Call { module, function, args } => {
            eval_function_call(module.as_ref(), function, args, bindings)
        }
        Expr::LocalCall { function, args } => {
            eval_function_call(None, function, args, bindings)
        }
        Expr::Paren(expr) => expr_eval(expr, bindings),
        Expr::Match { left, right } => {
            // Pattern matching: evaluate right side first, then match against left
            let (right_val, bindings1) = expr_eval(right, bindings)?;
            match_pattern(left, &right_val, &bindings1)
        }
    }
}

/// Evaluate binary operation
fn eval_binop(op: &BinOp, left: &Term, right: &Term) -> Result<Term, EvalError> {
    match op {
        BinOp::Add => {
            match (left, right) {
                (Term::Small(a), Term::Small(b)) => {
                    Ok(Term::Small(a + b))
                }
                (Term::Float(a), Term::Float(b)) => {
                    Ok(Term::Float(a + b))
                }
                (Term::Small(a), Term::Float(b)) => {
                    Ok(Term::Float(*a as f64 + b))
                }
                (Term::Float(a), Term::Small(b)) => {
                    Ok(Term::Float(a + *b as f64))
                }
                _ => Err(EvalError::TypeError("Invalid operands for addition".to_string())),
            }
        }
        BinOp::Sub => {
            match (left, right) {
                (Term::Small(a), Term::Small(b)) => {
                    Ok(Term::Small(a - b))
                }
                (Term::Float(a), Term::Float(b)) => {
                    Ok(Term::Float(a - b))
                }
                (Term::Small(a), Term::Float(b)) => {
                    Ok(Term::Float(*a as f64 - b))
                }
                (Term::Float(a), Term::Small(b)) => {
                    Ok(Term::Float(a - *b as f64))
                }
                _ => Err(EvalError::TypeError("Invalid operands for subtraction".to_string())),
            }
        }
        BinOp::Mul => {
            match (left, right) {
                (Term::Small(a), Term::Small(b)) => {
                    Ok(Term::Small(a * b))
                }
                (Term::Float(a), Term::Float(b)) => {
                    Ok(Term::Float(a * b))
                }
                (Term::Small(a), Term::Float(b)) => {
                    Ok(Term::Float(*a as f64 * b))
                }
                (Term::Float(a), Term::Small(b)) => {
                    Ok(Term::Float(a * *b as f64))
                }
                _ => Err(EvalError::TypeError("Invalid operands for multiplication".to_string())),
            }
        }
        BinOp::Div => {
            match (left, right) {
                (Term::Small(_), Term::Small(0)) => Err(EvalError::DivisionByZero),
                (Term::Float(a), Term::Float(b)) if *b == 0.0 => Err(EvalError::DivisionByZero),
                (Term::Small(a), Term::Small(b)) => {
                    Ok(Term::Float(*a as f64 / *b as f64))
                }
                (Term::Float(a), Term::Float(b)) => {
                    Ok(Term::Float(a / b))
                }
                (Term::Small(a), Term::Float(b)) => {
                    Ok(Term::Float(*a as f64 / b))
                }
                (Term::Float(a), Term::Small(b)) => {
                    Ok(Term::Float(a / *b as f64))
                }
                _ => Err(EvalError::TypeError("Invalid operands for division".to_string())),
            }
        }
        BinOp::IntDiv => {
            match (left, right) {
                (Term::Small(_), Term::Small(0)) => Err(EvalError::DivisionByZero),
                (Term::Small(a), Term::Small(b)) => {
                    Ok(Term::Small(a / b))
                }
                _ => Err(EvalError::TypeError("Invalid operands for integer division".to_string())),
            }
        }
        BinOp::Rem => {
            match (left, right) {
                (Term::Small(_), Term::Small(0)) => Err(EvalError::DivisionByZero),
                (Term::Small(a), Term::Small(b)) => {
                    Ok(Term::Small(a % b))
                }
                _ => Err(EvalError::TypeError("Invalid operands for remainder".to_string())),
            }
        }
        BinOp::Equal => {
            let true_atom = get_true_atom();
            let false_atom = get_false_atom();
            Ok(Term::Atom(if left == right { true_atom } else { false_atom }))
        }
        BinOp::NotEqual => {
            let true_atom = get_true_atom();
            let false_atom = get_false_atom();
            Ok(Term::Atom(if left != right { true_atom } else { false_atom }))
        }
        BinOp::Less => {
            let true_atom = get_true_atom();
            let false_atom = get_false_atom();
            match (left, right) {
                (Term::Small(a), Term::Small(b)) => {
                    Ok(Term::Atom(if a < b { true_atom } else { false_atom }))
                }
                (Term::Float(a), Term::Float(b)) => {
                    Ok(Term::Atom(if a < b { true_atom } else { false_atom }))
                }
                _ => Err(EvalError::TypeError("Invalid operands for comparison".to_string())),
            }
        }
        BinOp::LessEqual => {
            let true_atom = get_true_atom();
            let false_atom = get_false_atom();
            match (left, right) {
                (Term::Small(a), Term::Small(b)) => {
                    Ok(Term::Atom(if a <= b { true_atom } else { false_atom }))
                }
                (Term::Float(a), Term::Float(b)) => {
                    Ok(Term::Atom(if a <= b { true_atom } else { false_atom }))
                }
                _ => Err(EvalError::TypeError("Invalid operands for comparison".to_string())),
            }
        }
        BinOp::Greater => {
            let true_atom = get_true_atom();
            let false_atom = get_false_atom();
            match (left, right) {
                (Term::Small(a), Term::Small(b)) => {
                    Ok(Term::Atom(if a > b { true_atom } else { false_atom }))
                }
                (Term::Float(a), Term::Float(b)) => {
                    Ok(Term::Atom(if a > b { true_atom } else { false_atom }))
                }
                _ => Err(EvalError::TypeError("Invalid operands for comparison".to_string())),
            }
        }
        BinOp::GreaterEqual => {
            let true_atom = get_true_atom();
            let false_atom = get_false_atom();
            match (left, right) {
                (Term::Small(a), Term::Small(b)) => {
                    Ok(Term::Atom(if a >= b { true_atom } else { false_atom }))
                }
                (Term::Float(a), Term::Float(b)) => {
                    Ok(Term::Atom(if a >= b { true_atom } else { false_atom }))
                }
                _ => Err(EvalError::TypeError("Invalid operands for comparison".to_string())),
            }
        }
        BinOp::And | BinOp::AndAlso => {
            let true_atom = get_true_atom();
            let false_atom = get_false_atom();
            // Check if both are true atoms
            match (left, right) {
                (Term::Atom(a), Term::Atom(b)) if *a == true_atom && *b == true_atom => {
                    Ok(Term::Atom(true_atom))
                }
                _ => Ok(Term::Atom(false_atom)),
            }
        }
        BinOp::Or | BinOp::OrElse => {
            let true_atom = get_true_atom();
            let false_atom = get_false_atom();
            // Check if both are false atoms
            match (left, right) {
                (Term::Atom(a), Term::Atom(b)) if *a == false_atom && *b == false_atom => {
                    Ok(Term::Atom(false_atom))
                }
                _ => Ok(Term::Atom(true_atom)),
            }
        }
        BinOp::Xor => {
            let true_atom = get_true_atom();
            let false_atom = get_false_atom();
            match (left, right) {
                (Term::Atom(a), Term::Atom(b)) if *a != *b => Ok(Term::Atom(true_atom)),
                _ => Ok(Term::Atom(false_atom)),
            }
        }
    }
}

/// Evaluate unary operation
fn eval_unop(op: &UnOp, val: &Term) -> Result<Term, EvalError> {
    match op {
        UnOp::Not => {
            let true_atom = get_true_atom();
            let false_atom = get_false_atom();
            match val {
                Term::Atom(a) if *a == false_atom => Ok(Term::Atom(true_atom)), // not false = true
                Term::Atom(a) if *a == true_atom => Ok(Term::Atom(false_atom)), // not true = false
                _ => Err(EvalError::TypeError("Invalid operand for not".to_string())),
            }
        }
        UnOp::Neg => {
            match val {
                Term::Small(i) => Ok(Term::Small(-i)),
                Term::Float(f) => Ok(Term::Float(-f)),
                _ => Err(EvalError::TypeError("Invalid operand for negation".to_string())),
            }
        }
        UnOp::Pos => Ok(val.clone()),
    }
}

/// Evaluate function call
///
/// This function handles both local and remote function calls. For BIFs (Built-In Functions),
/// it calls the BIF dispatcher. For other functions, it would need to load and execute BEAM code.
fn eval_function_call(
    module: Option<&String>,
    function: &str,
    args: &[Expr],
    bindings: &Bindings,
) -> Result<(Term, Bindings), EvalError> {
    // Evaluate arguments
    let mut arg_values = Vec::new();
    let mut current_bindings = bindings.clone();
    
    for arg in args {
        let (val, new_bindings) = expr_eval(arg, &current_bindings)?;
        arg_values.push(val);
        current_bindings = new_bindings;
    }
    
    // Determine module (default to "erlang" if None)
    let module_name = module.map(|s| s.as_str()).unwrap_or("erlang");
    
    // Try built-in handlers for common operations first
    // TODO: Integrate with BIF dispatcher registry (requires breaking circular dependency)
    // For now, we handle common BIFs directly
    if module_name == "erlang" {
        match (function, arg_values.len()) {
            ("+", 2) => {
                eval_binop(&BinOp::Add, &arg_values[0], &arg_values[1])
                    .map(|result| (result, current_bindings))
            }
            ("-", 2) => {
                eval_binop(&BinOp::Sub, &arg_values[0], &arg_values[1])
                    .map(|result| (result, current_bindings))
            }
            ("*", 2) => {
                eval_binop(&BinOp::Mul, &arg_values[0], &arg_values[1])
                    .map(|result| (result, current_bindings))
            }
            ("/", 2) => {
                eval_binop(&BinOp::Div, &arg_values[0], &arg_values[1])
                    .map(|result| (result, current_bindings))
            }
            ("length", 1) => {
                // Calculate list length
                let len = list_length(&arg_values[0])?;
                Ok((Term::Small(len), current_bindings))
            }
            _ => {
                Err(EvalError::UndefinedFunction {
                    module: Some(module_name.to_string()),
                    function: function.to_string(),
                    arity: arg_values.len(),
                })
            }
        }
    } else {
        // Remote function call - try to load module on-demand
        // This is how the C REPL works: when you call lists:map/2, it loads lists.beam if needed
        
        use super::atom_table::get_global_atom_table;
        use entities_data_handling::AtomEncoding;
        use entities_io_operations::export::get_global_export_table;
        
        let atom_table = get_global_atom_table();
        let module_atom_index = atom_table.put_index(module_name.as_bytes(), AtomEncoding::SevenBitAscii, false)
            .map_err(|_| EvalError::InvalidOperation(format!("Failed to create atom for module: {}", module_name)))? as u32;
        
        let function_atom_index = atom_table.put_index(function.as_bytes(), AtomEncoding::SevenBitAscii, false)
            .map_err(|_| EvalError::InvalidOperation(format!("Failed to create atom for function: {}", function)))? as u32;
        
        let arity = arg_values.len() as u32;
        
        // Check export table first
        let export_table = get_global_export_table();
        let export = export_table.get(module_atom_index, function_atom_index, arity);
        
        if export.is_none() {
            // Export not found - try to load the module
            if let Err(_e) = try_load_module(module_name) {
                return Err(EvalError::UndefinedFunction {
                    module: Some(module_name.to_string()),
                    function: function.to_string(),
                    arity: arg_values.len(),
                });
            }
            
            // Check export table again after loading
            let export = export_table.get(module_atom_index, function_atom_index, arity);
            if export.is_none() {
                return Err(EvalError::UndefinedFunction {
                    module: Some(module_name.to_string()),
                    function: function.to_string(),
                    arity: arg_values.len(),
                });
            }
        }
        
        // Export exists - try to execute the function
        let export = export_table.get(module_atom_index, function_atom_index, arity)
            .ok_or_else(|| EvalError::UndefinedFunction {
                module: Some(module_name.to_string()),
                function: function.to_string(),
                arity: arg_values.len(),
            })?;
        
        // Check if we have a code pointer
        if let Some(code_ptr) = export.get_code_ptr() {
            // Execute the function
            execute_beam_function(code_ptr, &arg_values, &current_bindings)
        } else if export.label.is_some() {
            // We have a label but no code pointer - need to resolve it
            // For now, return an error indicating we need code loading
            Err(EvalError::FunctionCallError(format!(
                "Function {}/{} has label but code pointer not resolved. \
                 Code loading and label resolution not yet fully implemented.",
                module_name, function
            )))
        } else {
            // No code pointer or label - function exists but can't be executed
            Err(EvalError::FunctionCallError(format!(
                "Function {}/{} exists but has no code pointer or label. \
                 Module may not be fully loaded.",
                module_name, function
            )))
        }
    }
}

/// Execute a BEAM function call
///
/// Creates a temporary process, sets up registers with arguments,
/// executes the BEAM code, and extracts the result.
fn execute_beam_function(
    code_ptr: entities_process::ErtsCodePtr,
    arg_values: &[Term],
    bindings: &Bindings,
) -> Result<(Term, Bindings), EvalError> {
    use super::process_table::get_global_process_table;
    use std::sync::Arc;
    use entities_process::{Process, ProcessExecutor};
    
    // Convert Term arguments to Eterm
    let mut eterm_args = Vec::new();
    for arg in arg_values {
        eterm_args.push(term_to_eterm(arg)?);
    }
    
    // Create a temporary process for execution
    // We'll create it outside the process table for now, since we need mutable access
    // In a full implementation, we'd use interior mutability or a different approach
    let pid = 9999; // Temporary PID for REPL execution
    let mut temp_process = Process::new(pid);
    
    // Set instruction pointer to function code
    temp_process.set_i(code_ptr);
    
    // Set up argument registers in process heap
    // Allocate space for arguments
    let arg_start = temp_process.allocate_heap_words(eterm_args.len())
        .ok_or_else(|| EvalError::FunctionCallError("Failed to allocate heap for arguments".to_string()))?;
    
    // Copy arguments to process heap
    {
        let mut heap_slice = temp_process.heap_slice_mut();
        for (i, arg) in eterm_args.iter().enumerate() {
            heap_slice[arg_start + i] = *arg;
        }
    } // Drop the guard before moving temp_process
    
    // Create Arc from the configured process
    let process_arc = Arc::new(temp_process);
    
    // Set arity - need to check if Process has interior mutability for this
    // For now, we'll work around it by using the process's internal state
    // The process executor should handle arity from the heap
    
    // Execute the process using the global executor
    
    match entities_process::execute_process(process_arc.clone()) {
        Ok(result) => {
            match result {
                entities_process::ProcessExecutionResult::NormalExit => {
                    // Process finished - extract result from register x(0)
                    // In BEAM, the return value is typically in x(0) after a function returns
                    // For now, we'll try to read from the heap
                    let heap_slice = process_arc.heap_slice();
                    let heap_start = process_arc.heap_start_index();
                    
                    if heap_start < heap_slice.len() {
                        let result_eterm = heap_slice[heap_start];
                        let result_term = eterm_to_term(result_eterm)?;
                        Ok((result_term, bindings.clone()))
                    } else {
                        // No result available - return a default value
                        // In a full implementation, we'd get the result from the return instruction
                        Ok((Term::Atom(0), bindings.clone())) // Return 'ok' atom as default
                    }
                }
                entities_process::ProcessExecutionResult::Yield => {
                    Err(EvalError::FunctionCallError("Process yielded before completion".to_string()))
                }
                entities_process::ProcessExecutionResult::ErrorExit => {
                    Err(EvalError::FunctionCallError("Process exited with error".to_string()))
                }
            }
        }
        Err(e) => {
            Err(EvalError::FunctionCallError(format!("Process execution failed: {}", e)))
        }
    }
}

/// Try to load a module from code paths
///
/// Searches for the module's .beam file in the code paths and loads it.
/// This is called on-demand when a function from an unloaded module is called.
/// Also parses the BEAM file and registers exports in the export table.
fn try_load_module(module_name: &str) -> Result<(), String> {
    use code_management_code_loading::CodeLoader;
    use code_management_code_loading::code_loader::LoadError;
    use code_management_code_loading::BeamLoader;
    use std::path::Path;
    use std::fs;
    use entities_io_operations::export::get_global_export_table;
    use super::atom_table::get_global_atom_table;
    use entities_data_handling::AtomEncoding;
    
    // Get code paths from boot script module
    // We need to access the code paths that were set during boot script execution
    // For now, we'll use a simple approach: try common OTP library paths
    let code_paths = get_code_paths_for_module_loading();
    
    for code_path in &code_paths {
        let beam_path = Path::new(code_path).join(format!("{}.beam", module_name));
        
        // Try to read the BEAM file
        match fs::read(&beam_path) {
            Ok(beam_data) => {
                // Parse the BEAM file
                match BeamLoader::read_beam_file(&beam_data) {
                    Ok(beam_file) => {
                        // Verify module name matches
                        let atom_table = get_global_atom_table();
                        let module_atom_index = atom_table.put_index(module_name.as_bytes(), AtomEncoding::SevenBitAscii, false)
                            .map_err(|_| format!("Failed to create atom for module: {}", module_name))? as u32;
                        
                        // Register all exports in the export table with labels
                        // Note: The BEAM file's atom table indices may not match our global atom table
                        // In a full implementation, we'd need to map BEAM atom indices to global atom indices
                        // For now, we'll use the atom indices from the BEAM file directly
                        // This works if the atom table is consistent, but may need refinement
                        let export_table = get_global_export_table();
                        for (function_atom_idx, arity, label) in &beam_file.exports {
                            // Use put to create/get export, then update it with the label
                            export_table.put(module_atom_index, *function_atom_idx, *arity);
                            
                            // Update the export with the label
                            export_table.update_export_label(module_atom_index, *function_atom_idx, *arity, *label);
                        }
                        
                        eprintln!("      ✓ Loaded module {} on-demand (from {}), registered {} exports", 
                                 module_name, beam_path.display(), beam_file.exports.len());
                        return Ok(());
                    }
                    Err(e) => {
                        eprintln!("      ✗ Failed to parse BEAM file {}: {:?}", beam_path.display(), e);
                        continue; // Try next path
                    }
                }
            }
            Err(_) => {
                // File not found, try next path
                continue;
            }
        }
    }
    
    Err(format!("Module {} not found in code paths: {:?}", module_name, code_paths))
}

/// Get code paths for module loading
///
/// This function retrieves the code paths that were set during boot script execution.
/// In a full implementation, this would access the global code path storage.
/// For now, we use a simplified approach that tries common OTP library locations.
fn get_code_paths_for_module_loading() -> Vec<String> {
    // Try to get code paths from environment or use defaults
    // In a full implementation, this would access the global CODE_PATH from boot_script.rs
    // For now, we'll construct likely paths based on ROOTDIR
    
    let mut paths = Vec::new();
    
    // Add current directory
    paths.push(".".to_string());
    
    // Try to get ROOTDIR from environment
    if let Ok(rootdir) = std::env::var("ROOTDIR") {
        // Add standard library paths
        let lib_dir = format!("{}/lib", rootdir);
        if let Ok(entries) = std::fs::read_dir(&lib_dir) {
            for entry in entries.flatten() {
                if let Ok(entry_type) = entry.file_type() {
                    if entry_type.is_dir() {
                        let ebin_path = entry.path().join("ebin");
                        if ebin_path.exists() {
                            paths.push(ebin_path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }
    
    // Also try ERL_LIBS environment variable
    if let Ok(erlang_libs) = std::env::var("ERL_LIBS") {
        for lib_path in erlang_libs.split(':') {
            if !lib_path.is_empty() {
                let ebin_path = std::path::Path::new(lib_path).join("ebin");
                if ebin_path.exists() {
                    paths.push(ebin_path.to_string_lossy().to_string());
                }
            }
        }
    }
    
    paths
}

/// Calculate list length
fn list_length(term: &Term) -> Result<i64, EvalError> {
    let mut len = 0;
    let mut current = term;
    
    loop {
        match current {
            Term::Nil => break,
            Term::List { tail, .. } => {
                len += 1;
                current = tail;
            }
            _ => return Err(EvalError::TypeError("Expected list".to_string())),
        }
    }
    
    Ok(len)
}

/// Convert Term to Eterm
///
/// This is a simplified conversion. In a full implementation, Eterm would use
/// proper tagging for different types. For now, we use a simple encoding.
fn term_to_eterm(term: &Term) -> Result<Eterm, EvalError> {
    match term {
        Term::Small(i) => {
            // Simple encoding: small integers as-is (would need proper tagging in full impl)
            Ok(*i as u64)
        }
        Term::Float(f) => {
            // Float encoding (would need proper boxed representation in full impl)
            Ok(f.to_bits())
        }
        Term::Atom(index) => {
            // Atom encoding (would need proper tagging in full impl)
            Ok((*index as u64) << 32 | 0x0B) // Simplified: atom tag + index
        }
        Term::Nil => {
            Ok(0x3F) // Nil tag (simplified)
        }
        _ => {
            Err(EvalError::TypeError(format!("Cannot convert term to Eterm: {:?}", term)))
        }
    }
}

/// Convert Eterm to Term
///
/// This is a simplified conversion. In a full implementation, we would decode
/// proper Eterm tags. For now, we use a simple decoding.
fn eterm_to_term(eterm: Eterm) -> Result<Term, EvalError> {
    // Check for nil
    if eterm == 0x3F {
        return Ok(Term::Nil);
    }
    
    // Check for atom (simplified tag check)
    if (eterm & 0x3F) == 0x0B {
        let atom_index = ((eterm >> 32) & 0xFFFFFFFF) as u32;
        return Ok(Term::Atom(atom_index));
    }
    
    // Check if it's a small integer (simplified - would need proper tag check)
    // For now, assume values < 2^31 are small integers
    if eterm < (1u64 << 31) {
        return Ok(Term::Small(eterm as i64));
    }
    
    // Try to decode as float
    let f = f64::from_bits(eterm);
    if f.is_finite() {
        return Ok(Term::Float(f));
    }
    
    // Default: treat as small integer
    Ok(Term::Small(eterm as i64))
}

/// Match a pattern against a value
///
/// In Erlang, pattern matching binds variables. If the pattern is a variable
/// and it's unbound, it gets bound to the value. If it's already bound, the
/// values must match.
fn match_pattern(
    pattern: &Expr,
    value: &Term,
    bindings: &Bindings,
) -> Result<(Term, Bindings), EvalError> {
    match pattern {
        Expr::Var(var_name) => {
            // Variable pattern: bind or check match
            let mut new_bindings = bindings.clone();
            match new_bindings.get(var_name) {
                Some(existing_value) => {
                    // Variable already bound - check if values match
                    if existing_value == value {
                        Ok((value.clone(), new_bindings))
                    } else {
                        Err(EvalError::InvalidOperation(format!(
                            "Pattern match failed: {} already bound to {:?}, cannot bind to {:?}",
                            var_name, existing_value, value
                        )))
                    }
                }
                None => {
                    // Variable unbound - bind it to the value
                    new_bindings.insert(var_name.clone(), value.clone());
                    Ok((value.clone(), new_bindings))
                }
            }
        }
        Expr::Integer(i) => {
            // Integer literal pattern: must match exactly
            match value {
                Term::Small(j) if *i == *j => Ok((value.clone(), bindings.clone())),
                _ => Err(EvalError::InvalidOperation(format!(
                    "Pattern match failed: expected integer {}, got {:?}",
                    i, value
                ))),
            }
        }
        Expr::Float(f) => {
            // Float literal pattern: must match exactly
            match value {
                Term::Float(g) if (f - g).abs() < f64::EPSILON => Ok((value.clone(), bindings.clone())),
                _ => Err(EvalError::InvalidOperation(format!(
                    "Pattern match failed: expected float {}, got {:?}",
                    f, value
                ))),
            }
        }
        Expr::Atom(s) => {
            // Atom pattern: must match exactly
            use super::atom_table::get_global_atom_table;
            use entities_data_handling::AtomEncoding;
            let atom_table = get_global_atom_table();
            let pattern_index = atom_table.put_index(s.as_bytes(), AtomEncoding::SevenBitAscii, false)
                .map_err(|_| EvalError::InvalidOperation(format!("Failed to create atom: {}", s)))? as u32;
            
            match value {
                Term::Atom(value_index) if *value_index == pattern_index => {
                    Ok((value.clone(), bindings.clone()))
                }
                _ => Err(EvalError::InvalidOperation(format!(
                    "Pattern match failed: expected atom {}, got {:?}",
                    s, value
                ))),
            }
        }
        Expr::Paren(expr) => {
            // Parenthesized pattern
            match_pattern(expr, value, bindings)
        }
        Expr::Cons { head, tail } => {
            // List cons pattern: [Head | Tail]
            match value {
                Term::List { head: value_head, tail: value_tail } => {
                    // Match head pattern against value head
                    let (_, bindings1) = match_pattern(head, value_head, bindings)?;
                    // Match tail pattern against value tail (dereference Box)
                    match_pattern(tail, value_tail, &bindings1)
                }
                _ => Err(EvalError::InvalidOperation(format!(
                    "Pattern match failed: expected list, got {:?}",
                    value
                ))),
            }
        }
        Expr::List(pattern_elems) => {
            // List literal pattern: [E1, E2, ...]
            match value {
                Term::Nil => {
                    // Empty list pattern matches empty list
                    if pattern_elems.is_empty() {
                        Ok((Term::Nil, bindings.clone()))
                    } else {
                        Err(EvalError::InvalidOperation(format!(
                            "Pattern match failed: expected list with {} elements, got empty list",
                            pattern_elems.len()
                        )))
                    }
                }
                Term::List { head: value_head, tail: value_tail } => {
                    // Match each element
                    if pattern_elems.is_empty() {
                        // Pattern is empty list, value must be empty
                        if matches!(**value_tail, Term::Nil) {
                            Ok((Term::Nil, bindings.clone()))
                        } else {
                            Err(EvalError::InvalidOperation(
                                "Pattern match failed: expected empty list, got non-empty list".to_string()
                            ))
                        }
                    } else {
                        // Match first element
                        let (_, mut current_bindings) = match_pattern(&pattern_elems[0], value_head, bindings)?;
                        
                        // Match remaining elements
                        let mut current_value: &Term = value_tail.as_ref();
                        for pattern_elem in &pattern_elems[1..] {
                            match current_value {
                                Term::Nil => {
                                    return Err(EvalError::InvalidOperation(format!(
                                        "Pattern match failed: list too short, expected {} elements",
                                        pattern_elems.len()
                                    )));
                                }
                                Term::List { head: next_head, tail: next_tail } => {
                                    let (_, new_bindings) = match_pattern(pattern_elem, next_head, &current_bindings)?;
                                    current_bindings = new_bindings;
                                    current_value = next_tail.as_ref();
                                }
                                _ => {
                                    return Err(EvalError::InvalidOperation(
                                        "Pattern match failed: expected list".to_string()
                                    ));
                                }
                            }
                        }
                        
                        // Check if there are remaining elements in the value
                        if !matches!(current_value, Term::Nil) {
                            return Err(EvalError::InvalidOperation(format!(
                                "Pattern match failed: list too long, expected {} elements",
                                pattern_elems.len()
                            )));
                        }
                        
                        Ok((value.clone(), current_bindings))
                    }
                }
                _ => Err(EvalError::InvalidOperation(format!(
                    "Pattern match failed: expected list, got {:?}",
                    value
                ))),
            }
        }
        Expr::Tuple(pattern_elems) => {
            // Tuple pattern: {E1, E2, ...}
            match value {
                Term::Tuple(value_elems) => {
                    if pattern_elems.len() != value_elems.len() {
                        return Err(EvalError::InvalidOperation(format!(
                            "Pattern match failed: tuple arity mismatch, expected {} elements, got {}",
                            pattern_elems.len(),
                            value_elems.len()
                        )));
                    }
                    
                    // Match each element
                    let mut current_bindings = bindings.clone();
                    for (pattern_elem, value_elem) in pattern_elems.iter().zip(value_elems.iter()) {
                        let (_, new_bindings) = match_pattern(pattern_elem, value_elem, &current_bindings)?;
                        current_bindings = new_bindings;
                    }
                    
                    Ok((value.clone(), current_bindings))
                }
                _ => Err(EvalError::InvalidOperation(format!(
                    "Pattern match failed: expected tuple, got {:?}",
                    value
                ))),
            }
        }
        Expr::Nil => {
            // Nil pattern: must match empty list
            match value {
                Term::Nil => Ok((Term::Nil, bindings.clone())),
                _ => Err(EvalError::InvalidOperation(format!(
                    "Pattern match failed: expected empty list, got {:?}",
                    value
                ))),
            }
        }
        _ => {
            // Other patterns not yet supported
            Err(EvalError::InvalidOperation(format!(
                "Pattern matching not yet supported for: {:?}",
                pattern
            )))
        }
    }
}

/// Create new empty bindings
pub fn new_bindings() -> Bindings {
    HashMap::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_eval_integer() {
        let expr = Expr::Integer(42);
        let bindings = new_bindings();
        let (result, _) = expr(&expr, &bindings).unwrap();
        assert_eq!(result, Term::Small(42));
    }
    
    #[test]
    fn test_eval_add() {
        let expr = Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Integer(2)),
            right: Box::new(Expr::Integer(3)),
        };
        let bindings = new_bindings();
        let (result, _) = expr(&expr, &bindings).unwrap();
        assert_eq!(result, Term::Small(5));
    }
    
    #[test]
    fn test_eval_mul() {
        let expr = Expr::BinOp {
            op: BinOp::Mul,
            left: Box::new(Expr::Integer(2)),
            right: Box::new(Expr::Integer(3)),
        };
        let bindings = new_bindings();
        let (result, _) = expr(&expr, &bindings).unwrap();
        assert_eq!(result, Term::Small(6));
    }
}

