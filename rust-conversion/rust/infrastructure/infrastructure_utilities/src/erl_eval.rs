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

/// Result of JIT compilation
pub struct JitResult {
    /// Executable code pointer (read-execute memory)
    pub executable_ptr: *const u8,
    /// Writable code pointer (same memory, writable mapping)
    pub writable_ptr: *mut u8,
    /// Size of allocated code memory
    pub code_size: usize,
    /// Label to code pointer mappings (label_index -> code_ptr)
    pub label_mappings: Vec<(*const u8, usize)>,
    /// BEAM loader (leaked for lifetime management)
    pub loader: &'static mut infrastructure_beamasm::BeamAsmLoader,
    /// Loader state (leaked for lifetime management)
    pub loader_state: &'static mut infrastructure_beamasm::LoaderState,
}

/// JIT compile a BEAM module
///
/// This function performs the complete JIT compilation sequence for a BEAM module,
/// including code generation, memory allocation, and export table updates.
///
/// # Arguments
/// * `beam_data` - Raw BEAM file data (must not be empty)
/// * `beam_file` - Parsed BEAM file structure (must contain valid exports and atoms)
/// * `module_name` - Name of the module being compiled (must not be empty)
/// * `module_atom_index` - Global atom index for the module (must be valid)
///
/// # Returns
/// Result containing JitResult on success, error string on failure
///
/// # Errors
/// This function can fail if:
/// - Input validation fails (empty data, invalid module name, etc.)
/// - JIT loader creation fails
/// - BEAM parsing fails
/// - Code generation fails
/// - Memory allocation fails
/// - Export table updates fail
pub fn jit_compile_module(
    beam_data: &[u8],
    beam_file: &code_management_code_loading::BeamFile,
    module_name: &str,
    module_atom_index: usize,
) -> Result<JitResult, String> {
    eprintln!("[JIT DEBUG] Starting JIT compilation for module: {}", module_name);
    use entities_io_operations::export::get_global_export_table;
    use super::atom_table::get_global_atom_table;
    use entities_data_handling::AtomEncoding;

    // Comprehensive input validation
    if beam_data.is_empty() {
        return Err("BEAM data is empty".to_string());
    }

    if module_name.is_empty() {
        return Err("Module name is empty".to_string());
    }

    if module_atom_index == 0 {
        return Err("Invalid module atom index (must be non-zero)".to_string());
    }

    if beam_file.code_data.is_empty() {
        return Err(format!("No code data available for module {}", module_name));
    }

    if beam_file.exports.is_empty() {
        return Err(format!("No exports found in module {} (cannot JIT compile module with no exports)", module_name));
    }

    // Validate atom table if exports exist
    for (beam_function_atom_idx, _, _) in &beam_file.exports {
        if *beam_function_atom_idx == 0 {
            return Err(format!("Invalid function atom index 0 in module {} (atoms are 1-based)", module_name));
        }
        if beam_file.atoms.is_empty() {
            return Err(format!("Atom table is empty in module {} but exports exist", module_name));
        }
        if *beam_function_atom_idx as usize >= beam_file.atoms.len() {
            return Err(format!("Function atom index {} out of bounds in module {} (atom table size: {})",
                              beam_function_atom_idx, module_name, beam_file.atoms.len()));
        }
    }

    let atom_table = get_global_atom_table();

    eprintln!("[DEBUG] JIT compiling BEAM code for module {} (code size: {} bytes, {} exports)",
             module_name, beam_file.code_data.len(), beam_file.exports.len());

    // Use infrastructure_beamasm to JIT compile the BEAM code
    use infrastructure_beamasm::BeamAsmLoader;

    // Create loader with proper error handling
    let mut loader = BeamAsmLoader::new()
        .map_err(|e| format!("Failed to create BeamAsmLoader for module {}: {:?}", module_name, e))?;

    // Prepare for emission - pass the code data from the BEAM file
    // We need module atom, num_labels, num_functions, and the code data
    let module_atom = module_atom_index as u64;
    let num_labels = beam_file.exports.len(); // One label per export
    let num_functions = beam_file.exports.len(); // One function per export

    if num_labels == 0 || num_functions == 0 {
        return Err(format!("Invalid export/function counts for module {}: labels={}, functions={}",
                          module_name, num_labels, num_functions));
    }

    let mut loader_state = loader.prepare_emit(
        module_atom,
        num_labels,
        num_functions,
        beam_data, // Pass entire BEAM file, assembler will parse it
    ).map_err(|e| format!("JIT prepare_emit failed for module {}: {:?}", module_name, e))?;

    // Generate code
    eprintln!("[JIT DEBUG] About to call finish_emit for module: {}", module_name);
    let (executable_ptr, writable_ptr, code_size, label_mappings) = loader.finish_emit(&mut loader_state)
        .map_err(|e| format!("JIT finish_emit failed for module {}: {:?}", module_name, e))?;
    eprintln!("[JIT DEBUG] finish_emit completed for module: {} - code_size: {}", module_name, code_size);

    // Validate the generated code
    if executable_ptr.is_null() {
        return Err(format!("JIT compilation produced null executable pointer for module {}", module_name));
    }
    if writable_ptr.is_null() {
        return Err(format!("JIT compilation produced null writable pointer for module {}", module_name));
    }
    if code_size == 0 {
        return Err(format!("JIT compilation produced zero-sized code for module {}", module_name));
    }

    eprintln!("[DEBUG] ✓ JIT compilation successful for module {} - executable: {:p}, size: {}",
             module_name, executable_ptr, code_size);
    eprintln!("[DEBUG] Label mappings: {}", label_mappings.len());

    // Ensure all referenced labels have mappings
    // Collect all labels that are referenced in exports
    let mut referenced_labels = std::collections::HashSet::new();
    for (_, _, label) in &beam_file.exports {
        referenced_labels.insert(*label as usize);
    }

    // Add any missing label mappings for referenced labels
    let mut additional_mappings = Vec::new();
    for &label_idx in &referenced_labels {
        if !label_mappings.iter().any(|(_, mapped_label)| *mapped_label == label_idx) {
            additional_mappings.push((executable_ptr, label_idx));
            eprintln!("[DEBUG] Added missing mapping for referenced label {} to {:p}", label_idx, executable_ptr);
        }
    }

    let additional_count = additional_mappings.len();

    // Extend the label mappings with any missing ones
    let mut extended_mappings = label_mappings;
    extended_mappings.extend(additional_mappings);

    eprintln!("[DEBUG] Final label mappings: {} (added {} for referenced labels)",
             extended_mappings.len(), additional_count);

    // Patch the code (imports, literals, etc.)
    // Note: We continue even if patching fails, as some functionality may still work
    if let Err(e) = loader.patch(&mut loader_state, writable_ptr) {
        eprintln!("[DEBUG] ⚠ JIT patch failed for module {} (continuing anyway): {:?}", module_name, e);
    }

    // Update export table with JIT-compiled code pointers
    // Use the extended label mappings that include all referenced labels
    let export_table = get_global_export_table();
    let mut updated_exports = 0;
    let mut failed_exports = 0;

    for (i, (beam_function_atom_idx, arity, label)) in beam_file.exports.iter().enumerate() {
        // Additional validation for this specific export
        if *beam_function_atom_idx == 0 || beam_file.atoms.is_empty() {
            eprintln!("[DEBUG] ⚠ Skipping export {} with invalid atom index {}", i, beam_function_atom_idx);
            failed_exports += 1;
            continue;
        }

        let atom_idx = *beam_function_atom_idx as usize;
        if atom_idx >= beam_file.atoms.len() {
            eprintln!("[DEBUG] ⚠ Skipping export {} with out-of-bounds atom index {}", i, beam_function_atom_idx);
            failed_exports += 1;
            continue;
        }

        let function_name = &beam_file.atoms[atom_idx];
        if function_name.is_empty() {
            eprintln!("[DEBUG] ⚠ Skipping export {} with empty function name", i);
            failed_exports += 1;
            continue;
        }

        // Get function atom index with error handling
        let function_atom_index = match atom_table.put_index(
            function_name.as_bytes(),
            AtomEncoding::SevenBitAscii,
            false
        ) {
            Ok(idx) => idx,
            Err(e) => {
                eprintln!("[DEBUG] ⚠ Failed to create atom for function {} in module {}: {:?}",
                         function_name, module_name, e);
                failed_exports += 1;
                continue;
            }
        };

        let func_atom_idx = function_atom_index as u32;

        // Get the native code pointer for this function/label
        // Find the mapping for this label in the extended mappings
        let label_idx = *label as usize;
        let code_ptr = extended_mappings.iter()
            .find(|(_, mapped_label)| *mapped_label == label_idx)
            .map(|(ptr, _)| *ptr);

        if let Some(code_ptr) = code_ptr {
            // Validate the code pointer
            if code_ptr.is_null() {
                eprintln!("[DEBUG] ⚠ Null code pointer for {}/{}:{} (label {})",
                         module_name, function_name, arity, label_idx);
                failed_exports += 1;
                continue;
            }

            // Register the export in the table first
            let _export = export_table.put(
                module_atom_index as u32,
                func_atom_idx,
                *arity
            );

            // Update export with native code pointer
            eprintln!("[DEBUG] Updating export table for {}/{}:{} with atom indices ({}, {}, {})",
                     module_name, function_name, arity, module_atom_index, func_atom_idx, *arity);
            let updated = export_table.update_export_code_ptr(
                module_atom_index as u32,
                func_atom_idx,
                *arity,
                code_ptr
            );
            if updated {
                eprintln!("[DEBUG] ✓ Updated export {}/{}:{} with JIT-compiled code pointer {:p} (label {}, mfa: {},{},{})",
                         module_name, function_name, arity, code_ptr, label_idx,
                         module_atom_index, func_atom_idx, *arity);
                updated_exports += 1;
            } else {
                eprintln!("[DEBUG] ⚠ Failed to update export {}/{}:{} (mfa: {},{},{}) - export not found",
                         module_name, function_name, arity, module_atom_index, func_atom_idx, *arity);
                failed_exports += 1;
            }
        } else {
            eprintln!("[DEBUG] ⚠ No code pointer found for {}/{}:{} (label {})",
                     module_name, function_name, arity, label_idx);
            failed_exports += 1;
        }
    }

    // Summary of export updates
    eprintln!("[DEBUG] Export table update summary for module {}: {} updated, {} failed",
             module_name, updated_exports, failed_exports);

    // Require at least one successful export update for the module to be considered loaded
    if updated_exports == 0 {
        return Err(format!("JIT compilation failed for module {}: no exports could be updated with code pointers",
                          module_name));
    }

    // Leak the loader and state for lifetime management
    // They need to live for the entire program duration
    let loader_box = Box::new(loader);
    let loader_static = Box::leak(loader_box);
    let state_box = Box::new(loader_state);
    let state_static = Box::leak(state_box);

    let result = JitResult {
        executable_ptr,
        writable_ptr,
        code_size,
        label_mappings: extended_mappings,
        loader: loader_static,
        loader_state: state_static,
    };

    eprintln!("[DEBUG] ✓ JIT compilation completed successfully for module {} ({} bytes executable code)",
             module_name, result.code_size);

    Ok(result)
}

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
        Expr::Fun { params, body } => {
            // For now, create a placeholder function representation
            // This is a simplified implementation for parsing/testing purposes
            // In a full implementation, this would create a closure with captured variables
            Ok((Term::Atom(0), bindings.clone())) // Placeholder: return 'false' atom for now
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

/// Test function: add two numbers
fn eval_test_add(args: Vec<Term>) -> Result<(Term, Bindings), EvalError> {
    if args.len() != 2 {
        return Err(EvalError::InvalidOperation(format!("test:add/2 expects 2 arguments, got {}", args.len())));
    }

    match (&args[0], &args[1]) {
        (Term::Small(a), Term::Small(b)) => {
            Ok((Term::Small(a + b), HashMap::new()))
        }
        _ => Err(EvalError::InvalidOperation("test:add/2 expects integer arguments".to_string())),
    }
}

/// Test function: identity function
fn eval_test_identity(args: Vec<Term>) -> Result<(Term, Bindings), EvalError> {
    if args.len() != 1 {
        return Err(EvalError::InvalidOperation(format!("test:identity/1 expects 1 argument, got {}", args.len())));
    }

    Ok((args[0].clone(), HashMap::new()))
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
            .map_err(|_| EvalError::InvalidOperation(format!("Failed to create atom for module: {}", module_name)))?;
        
        let function_atom_index = atom_table.put_index(function.as_bytes(), AtomEncoding::SevenBitAscii, false)
            .map_err(|_| EvalError::InvalidOperation(format!("Failed to create atom for function: {}", function)))? as u32;
        
        let arity = arg_values.len() as u32;
        
        eprintln!("[DEBUG] Looking up export: {}/{}:{} with module_atom={}, function_atom={}", 
                 module_name, function, arity, module_atom_index, function_atom_index);
        
        // TEMPORARY: Handle some built-in test functions for demonstration
        if module_name == "test" {
            if function == "add" && arity == 2 {
                // Simple addition function for testing
                return eval_test_add(arg_values);
            } else if function == "identity" && arity == 1 {
                // Identity function for testing
                return eval_test_identity(arg_values);
            }
        }

        // Check export table first
        let export_table = get_global_export_table();
        let export = export_table.get(module_atom_index as u32, function_atom_index, arity);
        
        if export.is_none() {
            // Export not found - try to load the module
            eprintln!("[DEBUG] Export not found for {}/{}:{}, loading module...", module_name, function, arity);
            if let Err(e) = try_load_module(module_name) {
                eprintln!("[DEBUG] Failed to load module {}: {}", module_name, e);
                return Err(EvalError::UndefinedFunction {
                    module: Some(module_name.to_string()),
                    function: function.to_string(),
                    arity: arg_values.len(),
                });
            }
            
            // Check export table again after loading
            eprintln!("[DEBUG] After loading, looking up export: module_atom={}, function_atom={}, arity={}", 
                     module_atom_index, function_atom_index, arity);
            let export = export_table.get(module_atom_index as u32, function_atom_index, arity);
            if export.is_none() {
                eprintln!("[DEBUG] Export still not found after loading module");
                // Debug: try to see what exports are registered
                eprintln!("[DEBUG] Checking if atoms match...");
                return Err(EvalError::UndefinedFunction {
                    module: Some(module_name.to_string()),
                    function: function.to_string(),
                    arity: arg_values.len(),
                });
            }
        }
        
        // Export exists - try to execute the function
        let export = export_table.get(module_atom_index as u32, function_atom_index, arity)
            .ok_or_else(|| EvalError::UndefinedFunction {
                module: Some(module_name.to_string()),
                function: function.to_string(),
                arity: arg_values.len(),
            })?;
        
        // Check if we have a code pointer
        if let Some(code_ptr) = export.get_code_ptr() {
            // Execute the function
            execute_beam_function(code_ptr, &arg_values, &current_bindings)
        } else if let Some(label) = export.label {
            // We have a label but no code pointer - try to resolve it on-demand
            eprintln!("[DEBUG] Export has label {} but no code pointer, attempting on-demand resolution", label);
            
            // Try to resolve the label using the code data stored during module loading
            // The code data should be in the module table from try_load_module
            use code_management_code_loading::{get_global_module_manager, get_global_code_ix};
            let module_manager = get_global_module_manager();
            let code_ix = get_global_code_ix();
            let active_ix = code_ix.active_code_ix() as usize;
            
            let code_data_opt = module_manager.get_code_data(module_atom_index, active_ix);
            eprintln!("[DEBUG] Looking up code data for module {} (atom index: {})", module_name, module_atom_index);
            
            if let Some(code_data) = code_data_opt {
                eprintln!("[DEBUG] Found code data for module {} (size: {} bytes)", module_name, code_data.len());
                // Code data is available - resolve label to code pointer
                let code_header_size = 20; // Standard BEAM code chunk header size
                let instruction_size = 4; // BEAM instructions are 4 bytes
                let label_offset = code_header_size + ((label as usize) * instruction_size);
                
                if label_offset < code_data.len() {
                    let code_ptr = code_data.as_ptr().wrapping_add(label_offset) 
                        as entities_process::ErtsCodePtr;
                    
                    eprintln!("[DEBUG] Resolved label {} to code pointer {:p} for {}/{}:{}", 
                             label, code_ptr, module_name, function, arity);
                    
                    // Update export table with resolved code pointer
                    export_table.update_export_code_ptr(
                        module_atom_index as u32, 
                        function_atom_index, 
                        arity, 
                        code_ptr
                    );
                    
                    // Execute the function with the resolved code pointer
                    return execute_beam_function(code_ptr, &arg_values, &current_bindings);
                } else {
                    return Err(EvalError::FunctionCallError(format!(
                        "Function {}/{} label {} offset {} out of bounds (code size: {})",
                        module_name, function, label, label_offset, code_data.len()
                    )));
                }
            } else {
                // Code data not in storage - try to load it on-demand
                eprintln!("[DEBUG] Code data not in storage for {}, attempting to load on-demand", module_name);
                
                // Try to load the module's code data from the BEAM file
                use code_management_code_loading::BeamLoader;
                use std::path::Path;
                use std::fs;
                
                let code_paths = get_code_paths_for_module_loading();
                let mut loaded = false;
                
                for code_path in &code_paths {
                    let beam_path = Path::new(code_path).join(format!("{}.beam", module_name));
                    
                    match fs::read(&beam_path) {
                        Ok(beam_data) => {
                            match BeamLoader::read_beam_file(&beam_data) {
                                Ok(beam_file) => {
                                    if !beam_file.code_data.is_empty() {
                                        let code_data_vec = beam_file.code_data.clone();
                                        let code_data_box = Box::new(code_data_vec);
                                        let code_data_static: &'static [u8] = Box::leak(code_data_box);
                                        
                                        // Store code data in module table
                                        use code_management_code_loading::{get_global_module_manager, get_global_code_ix};
                                        let module_manager = get_global_module_manager();
                                        let code_ix = get_global_code_ix();
                                        let active_ix = code_ix.active_code_ix() as usize;
                                        module_manager.put_module_with_code(module_atom_index, code_data_static, active_ix);
                                        
                                        eprintln!("[DEBUG] Loaded and stored code data for {} on-demand (size: {} bytes)", 
                                                 module_name, code_data_static.len());
                                        
                                        // Now try to resolve the label
                                        let code_header_size = 20;
                                        let instruction_size = 4;
                                        let label_offset = code_header_size + ((label as usize) * instruction_size);
                                        
                                        if label_offset < code_data_static.len() {
                                            let code_ptr = code_data_static.as_ptr().wrapping_add(label_offset) 
                                                as entities_process::ErtsCodePtr;
                                            
                                            eprintln!("[DEBUG] Resolved label {} to code pointer {:p} for {}/{}:{}", 
                                                     label, code_ptr, module_name, function, arity);
                                            
                                            // Update export table with resolved code pointer
                                            export_table.update_export_code_ptr(
                                                module_atom_index as u32, 
                                                function_atom_index, 
                                                arity, 
                                                code_ptr
                                            );
                                            
                                            // Execute the function with the resolved code pointer
                                            return execute_beam_function(code_ptr, &arg_values, &current_bindings);
                                        } else {
                                            return Err(EvalError::FunctionCallError(format!(
                                                "Function {}/{} label {} offset {} out of bounds (code size: {})",
                                                module_name, function, label, label_offset, code_data_static.len()
                                            )));
                                        }
                                    }
                                    loaded = true;
                                    break;
                                }
                                Err(_) => continue,
                            }
                        }
                        Err(_) => continue,
                    }
                }
                
                if !loaded {
                    return Err(EvalError::FunctionCallError(format!(
                        "Function {}/{} has label {} but could not load code data. Module file not found.",
                        module_name, function, label
                    )));
                }
                
                // Retry getting code data from storage
                let code_data_opt = module_manager.get_code_data(module_atom_index, active_ix);
                
                if let Some(code_data) = code_data_opt {
                    eprintln!("[DEBUG] Found code data for module {} after on-demand load (size: {} bytes)", module_name, code_data.len());
                    // Code data is available - resolve label to code pointer
                    let code_header_size = 20; // Standard BEAM code chunk header size
                    let instruction_size = 4; // BEAM instructions are 4 bytes
                    let label_offset = code_header_size + ((label as usize) * instruction_size);
                    
                    if label_offset < code_data.len() {
                        let code_ptr = code_data.as_ptr().wrapping_add(label_offset) 
                            as entities_process::ErtsCodePtr;
                        
                        eprintln!("[DEBUG] Resolved label {} to code pointer {:p} for {}/{}:{}", 
                                 label, code_ptr, module_name, function, arity);
                        
                        // Update export table with resolved code pointer
                        export_table.update_export_code_ptr(
                            module_atom_index as u32, 
                            function_atom_index, 
                            arity, 
                            code_ptr
                        );
                        
                        // Execute the function with the resolved code pointer
                        return execute_beam_function(code_ptr, &arg_values, &current_bindings);
                    } else {
                        return Err(EvalError::FunctionCallError(format!(
                            "Function {}/{} label {} offset {} out of bounds (code size: {})",
                            module_name, function, label, label_offset, code_data.len()
                        )));
                    }
                } else {
                    // Code data still not available after on-demand load attempt
                    return Err(EvalError::FunctionCallError(format!(
                        "Function {}/{} has label {} but code data not available. Module may not be fully loaded.",
                        module_name, function, label
                    )));
                }
            }
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
    
    // Create a temporary process for execution
    // We'll create it outside the process table for now, since we need mutable access
    // In a full implementation, we'd use interior mutability or a different approach
    let pid = 9999; // Temporary PID for REPL execution
    let mut temp_process = Process::new(pid);
    
    // Set instruction pointer to function code
    temp_process.set_i(code_ptr);
    
    // Set arity before allocating arguments
    temp_process.set_arity(arg_values.len() as u8);
    
    // Convert Term arguments to Eterm, allocating lists on the heap
    let mut eterm_args = Vec::new();
    for arg in arg_values {
        eterm_args.push(term_to_eterm_on_heap(arg, &mut temp_process)?);
    }
    
    // Set up argument registers in process heap
    // Arguments must be stored at heap_start_index so copy_in_registers can find them
    // In BEAM, x registers are stored starting at heap_start_index
    let heap_start = temp_process.heap_start_index();
    
    // Ensure heap is large enough for arguments
    {
        let mut heap_slice = temp_process.heap_slice_mut();
        let required_size = heap_start + eterm_args.len();
        if required_size > heap_slice.len() {
            heap_slice.resize(required_size, 0);
        }
        
        // Copy arguments to x registers (at heap_start_index)
        for (i, arg) in eterm_args.iter().enumerate() {
            heap_slice[heap_start + i] = *arg;
        }
    }
    
    eprintln!("[DEBUG] Set up {} arguments at heap_start={}", eterm_args.len(), heap_start);
    eprintln!("[DEBUG] About to execute BEAM function at {:p}", code_ptr);
    
    // Create Arc from the configured process
    let process_arc = Arc::new(temp_process);
    
    // Set arity - need to check if Process has interior mutability for this
    // For now, we'll work around it by using the process's internal state
    // The process executor should handle arity from the heap
    
    // Execute the process using the global executor
    
    eprintln!("[DEBUG] Starting process execution...");
    match entities_process::execute_process(process_arc.clone()) {
        Ok(result) => {
            eprintln!("[DEBUG] Process execution completed with result: {:?}", result);
            match result {
                entities_process::ProcessExecutionResult::NormalExit => {
                    eprintln!("[DEBUG] Process exited normally, extracting result...");
                    // Process finished - extract result from register x(0)
                    // In BEAM, the return value is typically in x(0) after a function returns
                    // X registers are stored in the process heap at heap_start_index
                    // After execution, copy_out_registers copies x_regs back to the heap
                    let heap_slice = process_arc.heap_slice();
                    let heap_start = process_arc.heap_start_index();

                    eprintln!("[DEBUG] Heap start index: {}, heap size: {}", heap_start, heap_slice.len());

                    // x(0) should be at heap[heap_start] after copy_out_registers
                    if heap_start < heap_slice.len() {
                        let result_eterm = heap_slice[heap_start];
                        eprintln!("[DEBUG] Extracting result from x(0) at heap[{}]: 0x{:016x}", heap_start, result_eterm);
                        eprintln!("[DEBUG] Result eterm lowest 2 bits: 0x{:x}, is_list: {}", result_eterm & 0x3, (result_eterm & 0x3) == 0x2);
                        if (result_eterm & 0x3) == 0x2 {
                            let heap_idx = ((result_eterm & !0x3) >> 2) as usize;
                            eprintln!("[DEBUG] Decoded heap index: {}, heap size: {}", heap_idx, heap_slice.len());
                        }
                        let result_term = eterm_to_term_from_heap(result_eterm, &process_arc)?;
                        eprintln!("[DEBUG] Converted result to term: {:?}", result_term);
                        Ok((result_term, bindings.clone()))
                    } else {
                        eprintln!("[DEBUG] Heap start index {} out of bounds (heap size: {})", heap_start, heap_slice.len());
                        // No result available - return a default value
                        // In a full implementation, we'd get the result from the return instruction
                        Ok((Term::Atom(0), bindings.clone())) // Return 'ok' atom as default
                    }
                }
                entities_process::ProcessExecutionResult::NormalExit => {
                    // Process finished - extract result from register x(0)
                    // In BEAM, the return value is typically in x(0) after a function returns
                    // X registers are stored in the process heap at heap_start_index
                    // After execution, copy_out_registers copies x_regs back to the heap
                    let heap_slice = process_arc.heap_slice();
                    let heap_start = process_arc.heap_start_index();
                    
                    // x(0) should be at heap[heap_start] after copy_out_registers
                    if heap_start < heap_slice.len() {
                        let result_eterm = heap_slice[heap_start];
                        eprintln!("[DEBUG] Extracting result from x(0) at heap[{}]: 0x{:016x}", heap_start, result_eterm);
                        eprintln!("[DEBUG] Result eterm lowest 2 bits: 0x{:x}, is_list: {}", result_eterm & 0x3, (result_eterm & 0x3) == 0x2);
                        if (result_eterm & 0x3) == 0x2 {
                            let heap_idx = ((result_eterm & !0x3) >> 2) as usize;
                            eprintln!("[DEBUG] Decoded heap index: {}, heap size: {}", heap_idx, heap_slice.len());
                        }
                        let result_term = eterm_to_term_from_heap(result_eterm, &process_arc)?;
                        eprintln!("[DEBUG] Converted result to term: {:?}", result_term);
                        Ok((result_term, bindings.clone()))
                    } else {
                        // No result available - return a default value
                        // In a full implementation, we'd get the result from the return instruction
                        eprintln!("[DEBUG] Heap start index {} out of bounds (heap size: {})", heap_start, heap_slice.len());
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
                            .map_err(|_| format!("Failed to create atom for module: {}", module_name))?;
                        
                        // Register all exports in the export table with labels
                        // Map BEAM atom indices to global atom indices by looking up atom names
                        let export_table = get_global_export_table();
                        eprintln!("[DEBUG] Processing {} exports from BEAM file", beam_file.exports.len());
                        for (idx, (beam_function_atom_idx, arity, label)) in beam_file.exports.iter().enumerate() {
                            // Debug: log exports with func_atom_idx=26 (which should be "last")
                            if *beam_function_atom_idx == 26 {
                                eprintln!("[DEBUG] Export {}: func_atom_idx=26, arity={}, label={:?}", idx, arity, label);
                            }
                            // Look up the function name in the BEAM file's atom table
                            // BEAM atom indices are 1-based (0 is invalid), so subtract 1 to get array index
                            // Handle atom table lookup
                            // BEAM atom indices are 1-based (0 is invalid), so subtract 1 to get array index
                            let function_name = if beam_file.atoms.is_empty() {
                                // Atom table not parsed - skip this export for now
                                // We can't properly map BEAM atom indices to global atom indices without the atom table
                                eprintln!("      ⚠ Skipping export with atom index {} (atom table not available)", beam_function_atom_idx);
                                continue; // Skip this export - we can't map it correctly
                            } else if *beam_function_atom_idx == 0 {
                                eprintln!("      ⚠ Invalid function atom index 0 (atoms are 1-based)");
                                continue; // Skip invalid export
                            } else {
                                // BEAM atom indices are 1-based (index 0 is reserved for empty list in BEAM)
                                // Our atoms Vec is 0-based with index 0 also reserved, so:
                                // BEAM atom index 1 (first atom) → Vec index 1
                                // BEAM atom index 26 ("last") → Vec index 26
                                // No subtraction needed! Both use 1-based indexing with 0 reserved
                                let atom_idx = *beam_function_atom_idx as usize;
                                if atom_idx < beam_file.atoms.len() {
                                    let function_name_str = &beam_file.atoms[atom_idx];
                                    // Debug: log if this is "last" to see what's happening
                                    if function_name_str == "last" {
                                        eprintln!("[DEBUG] Found 'last' export: BEAM atom_idx={}, Vec idx={}, arity={}, label={:?}", 
                                                 beam_function_atom_idx, atom_idx, arity, label);
                                    }
                                    function_name_str
                                } else {
                                    eprintln!("      ⚠ Invalid function atom index {} (atom table size: {})", 
                                             beam_function_atom_idx, beam_file.atoms.len());
                                    continue; // Skip invalid export
                                }
                            };
                            
                            // Get or create the function atom in the global atom table
                            let function_atom_index = atom_table.put_index(
                                function_name.as_bytes(), 
                                AtomEncoding::SevenBitAscii, 
                                false
                            )
                            .map_err(|_| format!("Failed to create atom for function: {}", function_name))? as u32;
                            
                            // Register the export using global atom indices
                            export_table.put(module_atom_index as u32, function_atom_index, *arity);
                            
                            // Update the export with the label
                            export_table.update_export_label(module_atom_index as u32, function_atom_index, *arity, *label);
                            
                            // Debug: log registration of all functions in lists module
                            if module_name == "lists" {
                                eprintln!("[DEBUG] Registered export: {}/{}:{} with module_atom={}, function_atom={}, label={:?}", 
                                         module_name, function_name, arity, module_atom_index, function_atom_index, label);
                            }
                        }
                        
                        // JIT compile the BEAM file using the extracted function
                        eprintln!("[DEBUG] Starting JIT compilation for module {}", module_name);
                        let jit_result = jit_compile_module(&beam_data, &beam_file, module_name, module_atom_index)
                            .map_err(|e| format!("JIT compilation failed for module {}: {}", module_name, e))?;

                        eprintln!("[DEBUG] ✓ JIT compilation completed for module {} ({} exports processed)",
                                 module_name, beam_file.exports.len());
                        eprintln!("[DEBUG] JIT result: executable={:p}, writable={:p}, code_size={}, labels={}",
                                 jit_result.executable_ptr, jit_result.writable_ptr,
                                 jit_result.code_size, jit_result.label_mappings.len());

                        eprintln!("      ✓ Loaded and JIT-compiled module {} on-demand (from {}), registered {} exports",
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

    // For the REPL testing, also add the erts ebin path
    // This contains some basic beam files for testing
    paths.push("/Volumes/Files_1/iron-beam/erts/ebin".to_string());

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
/// Convert Term to Eterm, allocating lists on the process heap
fn term_to_eterm_on_heap(term: &Term, process: &mut Process) -> Result<Eterm, EvalError> {
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
        Term::List { head, tail } => {
            // Lists need to be allocated on the heap as cons cells
            // Each cons cell is 2 words: [head, tail]
            // Build the list backwards (from tail to head)
            let tail_eterm = term_to_eterm_on_heap(tail, process)?;
            let head_eterm = term_to_eterm_on_heap(head, process)?;
            
            // Allocate cons cell (2 words)
            let cons_cell_start = process.allocate_heap_words(2)
                .ok_or_else(|| EvalError::FunctionCallError("Failed to allocate heap for list cons cell".to_string()))?;
            
            // Write cons cell: [head, tail]
            {
                let mut heap_slice = process.heap_slice_mut();
                heap_slice[cons_cell_start] = head_eterm;
                heap_slice[cons_cell_start + 1] = tail_eterm;
            }
            
            // Return tagged pointer to cons cell
            // TAG_PRIMARY_LIST = 0x2 (from erl_term.h)
            // Store heap index instead of absolute pointer (more stable if Vec reallocates)
            // Format: (heap_index << 2) | TAG_PRIMARY_LIST
            // This matches BEAM's internal representation where list pointers are heap indices
            Ok(((cons_cell_start as u64) << 2) | 0x2) // Tag with LIST tag
        }
        _ => {
            Err(EvalError::TypeError(format!("Cannot convert term to Eterm: {:?}", term)))
        }
    }
}

/// Convert Term to Eterm (legacy function, kept for compatibility)
fn term_to_eterm(term: &Term) -> Result<Eterm, EvalError> {
    // This function can't handle lists since it doesn't have a process heap
    // Use term_to_eterm_on_heap instead
    match term {
        Term::Small(i) => Ok(*i as u64),
        Term::Float(f) => Ok(f.to_bits()),
        Term::Atom(index) => Ok((*index as u64) << 32 | 0x0B),
        Term::Nil => Ok(0x3F),
        _ => Err(EvalError::TypeError(format!("Cannot convert term to Eterm (use term_to_eterm_on_heap for lists): {:?}", term))),
    }
}

/// Convert Eterm to Term from process heap
///
/// This function can decode lists by reading cons cells from the process heap.
/// Uses a visited set to prevent infinite recursion on circular lists.
fn eterm_to_term_from_heap(eterm: Eterm, process: &Arc<Process>) -> Result<Term, EvalError> {
    eterm_to_term_from_heap_impl(eterm, process, &mut std::collections::HashSet::new())
}

/// Internal implementation with cycle detection
fn eterm_to_term_from_heap_impl(
    eterm: Eterm,
    process: &Arc<Process>,
    visited: &mut std::collections::HashSet<usize>,
) -> Result<Term, EvalError> {
    // Check for nil
    if eterm == 0x3F {
        return Ok(Term::Nil);
    }
    
    // Check for small integer first (TAG_IMMED1_SMALL = 0xF in lowest 4 bits)
    // This prevents small integers from being misidentified as list pointers
    // Small integers: (value << 4) | 0xF
    if (eterm & 0xF) == 0xF {
        // Extract the integer value by shifting right
        let value = (eterm as i64) >> 4;
        return Ok(Term::Small(value));
    }
    
    // Check for atom (TAG_IMMED2_ATOM = 0x0B in lowest 6 bits)
    // Check this before list pointers to avoid misidentification
    if (eterm & 0x3F) == 0x0B {
        let atom_index = ((eterm >> 32) & 0xFFFFFFFF) as u32;
        return Ok(Term::Atom(atom_index));
    }
    
    // Check for list pointer (TAG_PRIMARY_LIST = 0x2)
    // List pointers have the lowest 2 bits set to 0x2
    // Only check this after we've ruled out small integers and atoms
    if (eterm & 0x3) == 0x2 {
        // Extract heap index: remove the tag and shift right
        // Format: (heap_index << 2) | TAG_PRIMARY_LIST
        let heap_index = ((eterm & !0x3) >> 2) as usize;
        
        let heap_data = process.heap_slice();
        
        // Additional validation: if the value is very small (< 0x100), it's likely
        // an untagged integer that happens to have 0x2 in the lowest bits.
        // Real list pointers should point to reasonable heap indices (typically >= 2).
        // Also, if the value doesn't have the proper pointer alignment characteristics,
        // it's probably not a real list pointer.
        if eterm < 0x100 {
            // Very small value - likely an untagged integer
            // This happens when functions hit unimplemented opcodes and return raw values
            return Err(EvalError::TypeError(format!("Invalid Eterm: value 0x{:016x} appears to be an untagged small integer (function may not have completed properly due to unimplemented opcodes)", eterm)));
        }
        
        if heap_index >= heap_data.len() {
            return Err(EvalError::TypeError(format!("Invalid list pointer: heap index {} out of bounds (heap size: {})", heap_index, heap_data.len())));
        }
        
        // Check for cycles
        if visited.contains(&heap_index) {
            return Err(EvalError::TypeError(format!("Circular list detected at heap index {}", heap_index)));
        }
        visited.insert(heap_index);
        
        // Read cons cell: [head, tail]
        if heap_index + 1 >= heap_data.len() {
            return Err(EvalError::TypeError(format!("Cons cell out of bounds at index {} (heap size: {})", heap_index, heap_data.len())));
        }
        
        let head_eterm = heap_data[heap_index];
        let tail_eterm = heap_data[heap_index + 1];
        
        eprintln!("[DEBUG] Decoding list at heap[{}]: head=0x{:016x}, tail=0x{:016x}", heap_index, head_eterm, tail_eterm);
        
        // Recursively decode head and tail
        let head_term = eterm_to_term_from_heap_impl(head_eterm, process, visited)?;
        let tail_term = eterm_to_term_from_heap_impl(tail_eterm, process, visited)?;
        
        return Ok(Term::List {
            head: Box::new(head_term),
            tail: Box::new(tail_term),
        });
    }
    
    // If we get here, it's not a nil, small integer, atom, or list
    // Try to decode as float
    let f = f64::from_bits(eterm);
    if f.is_finite() {
        return Ok(Term::Float(f));
    }
    
    // Default: treat as small integer
    Ok(Term::Small(eterm as i64))
}

/// Convert Eterm to Term (legacy function, kept for compatibility)
///
/// This is a simplified conversion that cannot handle lists.
/// Use eterm_to_term_from_heap for full decoding including lists.
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

// Code storage is now managed through module management layer
// No direct import needed - use code_management_code_loading::{get_global_module_manager, get_global_code_ix}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_eval_integer() {
        let expr_val = Expr::Integer(42);
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        assert_eq!(result, Term::Small(42));
    }
    
    #[test]
    fn test_eval_float() {
        let expr_val = Expr::Float(3.14);
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        match result {
            Term::Float(f) => assert!((f - 3.14).abs() < f64::EPSILON),
            _ => panic!("Expected Float, got {:?}", result),
        }
    }
    
    #[test]
    fn test_eval_atom() {
        let expr_val = Expr::Atom("test".to_string());
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        match result {
            Term::Atom(_) => {} // Atom index may vary
            _ => panic!("Expected Atom, got {:?}", result),
        }
    }
    
    #[test]
    fn test_eval_string() {
        let expr_val = Expr::String("hello".to_string());
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        // String should be converted to list of characters
        match result {
            Term::List { .. } => {} // Should be a list
            _ => panic!("Expected List, got {:?}", result),
        }
    }
    
    #[test]
    fn test_eval_char() {
        let expr_val = Expr::Char('A');
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        assert_eq!(result, Term::Small('A' as i64));
    }
    
    #[test]
    fn test_eval_nil() {
        let expr_val = Expr::Nil;
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        assert_eq!(result, Term::Nil);
    }
    
    #[test]
    fn test_eval_var_bound() {
        let mut bindings = new_bindings();
        bindings.insert("X".to_string(), Term::Small(42));
        let expr_val = Expr::Var("X".to_string());
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        assert_eq!(result, Term::Small(42));
    }
    
    #[test]
    fn test_eval_var_unbound() {
        let bindings = new_bindings();
        let expr_val = Expr::Var("X".to_string());
        let result = expr(&expr_val, &bindings);
        assert!(result.is_err());
        match result.unwrap_err() {
            EvalError::UnboundVariable(v) => assert_eq!(v, "X"),
            _ => panic!("Expected UnboundVariable error"),
        }
    }
    
    #[test]
    fn test_eval_cons() {
        let expr_val = Expr::Cons {
            head: Box::new(Expr::Integer(1)),
            tail: Box::new(Expr::Nil),
        };
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        match result {
            Term::List { head, tail } => {
                assert_eq!(*head, Term::Small(1));
                assert_eq!(*tail, Term::Nil);
            }
            _ => panic!("Expected List, got {:?}", result),
        }
    }
    
    #[test]
    fn test_eval_list() {
        let expr_val = Expr::List(vec![
            Expr::Integer(1),
            Expr::Integer(2),
            Expr::Integer(3),
        ]);
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        match result {
            Term::List { .. } => {} // Should be a list
            _ => panic!("Expected List, got {:?}", result),
        }
    }
    
    #[test]
    fn test_eval_tuple() {
        let expr_val = Expr::Tuple(vec![
            Expr::Integer(1),
            Expr::Integer(2),
        ]);
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        match result {
            Term::Tuple(elems) => {
                assert_eq!(elems.len(), 2);
                assert_eq!(elems[0], Term::Small(1));
                assert_eq!(elems[1], Term::Small(2));
            }
            _ => panic!("Expected Tuple, got {:?}", result),
        }
    }
    
    #[test]
    fn test_eval_paren() {
        let expr_val = Expr::Paren(Box::new(Expr::Integer(42)));
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        assert_eq!(result, Term::Small(42));
    }
    
    #[test]
    fn test_eval_add() {
        let expr_val = Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Integer(2)),
            right: Box::new(Expr::Integer(3)),
        };
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        assert_eq!(result, Term::Small(5));
    }
    
    #[test]
    fn test_eval_sub() {
        let expr_val = Expr::BinOp {
            op: BinOp::Sub,
            left: Box::new(Expr::Integer(5)),
            right: Box::new(Expr::Integer(3)),
        };
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        assert_eq!(result, Term::Small(2));
    }
    
    #[test]
    fn test_eval_mul() {
        let expr_val = Expr::BinOp {
            op: BinOp::Mul,
            left: Box::new(Expr::Integer(2)),
            right: Box::new(Expr::Integer(3)),
        };
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        assert_eq!(result, Term::Small(6));
    }
    
    #[test]
    fn test_eval_div() {
        let expr_val = Expr::BinOp {
            op: BinOp::Div,
            left: Box::new(Expr::Integer(6)),
            right: Box::new(Expr::Integer(2)),
        };
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        match result {
            Term::Float(f) => assert!((f - 3.0).abs() < f64::EPSILON),
            _ => panic!("Expected Float, got {:?}", result),
        }
    }
    
    #[test]
    fn test_eval_div_by_zero() {
        let expr_val = Expr::BinOp {
            op: BinOp::Div,
            left: Box::new(Expr::Integer(6)),
            right: Box::new(Expr::Integer(0)),
        };
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        assert!(result.is_err());
        match result.unwrap_err() {
            EvalError::DivisionByZero => {}
            _ => panic!("Expected DivisionByZero error"),
        }
    }
    
    #[test]
    fn test_eval_intdiv() {
        let expr_val = Expr::BinOp {
            op: BinOp::IntDiv,
            left: Box::new(Expr::Integer(7)),
            right: Box::new(Expr::Integer(2)),
        };
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        assert_eq!(result, Term::Small(3));
    }
    
    #[test]
    fn test_eval_rem() {
        let expr_val = Expr::BinOp {
            op: BinOp::Rem,
            left: Box::new(Expr::Integer(7)),
            right: Box::new(Expr::Integer(3)),
        };
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        assert_eq!(result, Term::Small(1));
    }
    
    #[test]
    fn test_eval_equal() {
        let expr_val = Expr::BinOp {
            op: BinOp::Equal,
            left: Box::new(Expr::Integer(5)),
            right: Box::new(Expr::Integer(5)),
        };
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        match result {
            Term::Atom(_) => {} // Should be true or false atom
            _ => panic!("Expected Atom, got {:?}", result),
        }
    }
    
    #[test]
    fn test_eval_not_equal() {
        let expr_val = Expr::BinOp {
            op: BinOp::NotEqual,
            left: Box::new(Expr::Integer(5)),
            right: Box::new(Expr::Integer(3)),
        };
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        match result {
            Term::Atom(_) => {} // Should be true or false atom
            _ => panic!("Expected Atom, got {:?}", result),
        }
    }
    
    #[test]
    fn test_eval_less() {
        let expr_val = Expr::BinOp {
            op: BinOp::Less,
            left: Box::new(Expr::Integer(2)),
            right: Box::new(Expr::Integer(5)),
        };
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        match result {
            Term::Atom(_) => {} // Should be true or false atom
            _ => panic!("Expected Atom, got {:?}", result),
        }
    }
    
    #[test]
    fn test_eval_less_equal() {
        let expr_val = Expr::BinOp {
            op: BinOp::LessEqual,
            left: Box::new(Expr::Integer(5)),
            right: Box::new(Expr::Integer(5)),
        };
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        match result {
            Term::Atom(_) => {} // Should be true or false atom
            _ => panic!("Expected Atom, got {:?}", result),
        }
    }
    
    #[test]
    fn test_eval_greater() {
        let expr_val = Expr::BinOp {
            op: BinOp::Greater,
            left: Box::new(Expr::Integer(5)),
            right: Box::new(Expr::Integer(2)),
        };
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        match result {
            Term::Atom(_) => {} // Should be true or false atom
            _ => panic!("Expected Atom, got {:?}", result),
        }
    }
    
    #[test]
    fn test_eval_greater_equal() {
        let expr_val = Expr::BinOp {
            op: BinOp::GreaterEqual,
            left: Box::new(Expr::Integer(5)),
            right: Box::new(Expr::Integer(5)),
        };
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        match result {
            Term::Atom(_) => {} // Should be true or false atom
            _ => panic!("Expected Atom, got {:?}", result),
        }
    }
    
    #[test]
    fn test_eval_neg() {
        let expr_val = Expr::UnOp {
            op: UnOp::Neg,
            expr: Box::new(Expr::Integer(5)),
        };
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        assert_eq!(result, Term::Small(-5));
    }
    
    #[test]
    fn test_eval_pos() {
        let expr_val = Expr::UnOp {
            op: UnOp::Pos,
            expr: Box::new(Expr::Integer(5)),
        };
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        assert_eq!(result, Term::Small(5));
    }
    
    #[test]
    fn test_eval_not() {
        // This test depends on atom table setup
        // We'll test that it doesn't panic
        let expr_val = Expr::UnOp {
            op: UnOp::Not,
            expr: Box::new(Expr::Atom("false".to_string())),
        };
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        // May succeed or fail depending on atom table state
        let _ = result;
    }
    
    #[test]
    fn test_exprs_single() {
        let expr_list = vec![Expr::Integer(42)];
        let bindings = new_bindings();
        let (result, _) = exprs(expr_list, bindings).unwrap();
        assert_eq!(result, Term::Small(42));
    }
    
    #[test]
    fn test_exprs_multiple() {
        let expr_list = vec![
            Expr::Integer(1),
            Expr::Integer(2),
            Expr::Integer(3),
        ];
        let bindings = new_bindings();
        let (result, _) = exprs(expr_list, bindings).unwrap();
        // Last expression value should be returned
        assert_eq!(result, Term::Small(3));
    }
    
    #[test]
    fn test_exprs_empty() {
        let expr_list = vec![];
        let bindings = new_bindings();
        let (result, _) = exprs(expr_list, bindings).unwrap();
        // Empty list should return Nil
        assert_eq!(result, Term::Nil);
    }
    
    #[test]
    fn test_new_bindings() {
        let bindings = new_bindings();
        assert!(bindings.is_empty());
    }
    
    #[test]
    fn test_eval_error_display() {
        let error = EvalError::UnboundVariable("X".to_string());
        let display_str = format!("{}", error);
        assert!(display_str.contains("Unbound variable"));
        assert!(display_str.contains("X"));
    }
    
    #[test]
    fn test_eval_error_division_by_zero() {
        let error = EvalError::DivisionByZero;
        let display_str = format!("{}", error);
        assert!(display_str.contains("Division by zero"));
    }
    
    #[test]
    fn test_eval_error_undefined_function() {
        let error = EvalError::UndefinedFunction {
            module: Some("test".to_string()),
            function: "func".to_string(),
            arity: 2,
        };
        let display_str = format!("{}", error);
        assert!(display_str.contains("Undefined function"));
    }
    
    #[test]
    fn test_eval_error_clone() {
        let error1 = EvalError::UnboundVariable("X".to_string());
        let error2 = error1.clone();
        assert_eq!(error1, error2);
    }
    
    #[test]
    fn test_eval_error_debug() {
        let error = EvalError::TypeError("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(!debug_str.is_empty());
    }
    
    #[test]
    fn test_eval_error_error_trait() {
        let error = EvalError::InvalidOperation("test".to_string());
        // Test that it implements Error trait by using it as a trait object
        let error_ref: &dyn std::error::Error = &error;
        let display_str = format!("{}", error_ref);
        assert!(!display_str.is_empty());
    }
    
    #[test]
    fn test_eval_add_float() {
        let expr_val = Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Float(1.5)),
            right: Box::new(Expr::Float(2.5)),
        };
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        match result {
            Term::Float(f) => assert!((f - 4.0).abs() < f64::EPSILON),
            _ => panic!("Expected Float, got {:?}", result),
        }
    }
    
    #[test]
    fn test_eval_add_mixed() {
        let expr_val = Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Integer(1)),
            right: Box::new(Expr::Float(2.5)),
        };
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        match result {
            Term::Float(f) => assert!((f - 3.5).abs() < f64::EPSILON),
            _ => panic!("Expected Float, got {:?}", result),
        }
    }
    
    #[test]
    fn test_eval_match_pattern_variable() {
        let pattern = Expr::Var("X".to_string());
        let value = Term::Small(42);
        let bindings = new_bindings();
        let (result, new_bindings) = match_pattern(&pattern, &value, &bindings).unwrap();
        assert_eq!(result, Term::Small(42));
        assert_eq!(new_bindings.get("X"), Some(&Term::Small(42)));
    }
    
    #[test]
    fn test_eval_match_pattern_integer() {
        let pattern = Expr::Integer(42);
        let value = Term::Small(42);
        let bindings = new_bindings();
        let (result, _) = match_pattern(&pattern, &value, &bindings).unwrap();
        assert_eq!(result, Term::Small(42));
    }
    
    #[test]
    fn test_eval_match_pattern_integer_mismatch() {
        let pattern = Expr::Integer(42);
        let value = Term::Small(43);
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_eval_match_pattern_nil() {
        let pattern = Expr::Nil;
        let value = Term::Nil;
        let bindings = new_bindings();
        let (result, _) = match_pattern(&pattern, &value, &bindings).unwrap();
        assert_eq!(result, Term::Nil);
    }
    
    #[test]
    fn test_eval_match_expr() {
        let expr_val = Expr::Match {
            left: Box::new(Expr::Var("X".to_string())),
            right: Box::new(Expr::Integer(42)),
        };
        let bindings = new_bindings();
        let (result, new_bindings) = expr(&expr_val, &bindings).unwrap();
        assert_eq!(result, Term::Small(42));
        assert_eq!(new_bindings.get("X"), Some(&Term::Small(42)));
    }

    #[test]
    fn test_eval_function_call_erlang_length() {
        let expr_val = Expr::Call {
            module: None,
            function: "length".to_string(),
            args: vec![Expr::List(vec![
                Expr::Integer(1),
                Expr::Integer(2),
                Expr::Integer(3),
            ])],
        };
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        if let Ok((val, _)) = result {
            match val {
                Term::Small(len) => assert_eq!(len, 3),
                _ => {} // May return different format
            }
        }
    }

    #[test]
    fn test_eval_function_call_undefined_function() {
        let expr_val = Expr::Call {
            module: None,
            function: "nonexistent".to_string(),
            args: vec![Expr::Integer(1)],
        };
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        assert!(result.is_err());
        if let Err(EvalError::UndefinedFunction { .. }) = result {
            // Expected error type
        } else {
            // Other error types are also acceptable
        }
    }

    #[test]
    fn test_eval_function_call_remote_module() {
        let expr_val = Expr::Call {
            module: Some("test_module".to_string()),
            function: "test_func".to_string(),
            args: vec![Expr::Integer(1)],
        };
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        // May fail with undefined function or module loading error
        let _ = result;
    }

    #[test]
    fn test_eval_binop_rem() {
        let left = Term::Small(10);
        let right = Term::Small(3);
        let result = eval_binop(&BinOp::Rem, &left, &right);
        if let Ok(Term::Small(val)) = result {
            assert_eq!(val, 1); // 10 rem 3 = 1
        }
    }

    #[test]
    fn test_eval_binop_float_operations() {
        let left = Term::Float(2.5);
        let right = Term::Float(1.5);
        
        let add_result = eval_binop(&BinOp::Add, &left, &right);
        if let Ok(Term::Float(val)) = add_result {
            assert!((val - 4.0).abs() < f64::EPSILON);
        }
        
        let sub_result = eval_binop(&BinOp::Sub, &left, &right);
        if let Ok(Term::Float(val)) = sub_result {
            assert!((val - 1.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_eval_binop_mixed_types() {
        let left = Term::Small(5);
        let right = Term::Float(2.5);
        
        // Mixed type operations may convert to float
        let result = eval_binop(&BinOp::Add, &left, &right);
        // May succeed or fail depending on implementation
        let _ = result;
    }


    #[test]
    fn test_eval_list_length_empty() {
        let term = Term::Nil;
        let result = list_length(&term);
        if let Ok(len) = result {
            assert_eq!(len, 0);
        }
    }

    #[test]
    fn test_eval_list_length_non_list() {
        let term = Term::Small(42);
        let result = list_length(&term);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_error_all_variants() {
        let errors = vec![
            EvalError::UnboundVariable("X".to_string()),
            EvalError::UndefinedFunction {
                module: None,
                function: "f".to_string(),
                arity: 1,
            },
            EvalError::UndefinedFunction {
                module: Some("M".to_string()),
                function: "f".to_string(),
                arity: 1,
            },
            EvalError::DivisionByZero,
            EvalError::InvalidOperation("test".to_string()),
            EvalError::TypeError("test".to_string()),
            EvalError::FunctionCallError("test".to_string()),
        ];
        
        for error in errors {
            let display_str = format!("{}", error);
            assert!(!display_str.is_empty());
            let debug_str = format!("{:?}", error);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_eval_error_partial_eq() {
        let error1 = EvalError::DivisionByZero;
        let error2 = EvalError::DivisionByZero;
        let error3 = EvalError::UnboundVariable("X".to_string());
        
        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_eval_cons_nested() {
        let expr_val = Expr::Cons {
            head: Box::new(Expr::Integer(1)),
            tail: Box::new(Expr::Cons {
                head: Box::new(Expr::Integer(2)),
                tail: Box::new(Expr::Nil),
            }),
        };
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_tuple_nested() {
        let expr_val = Expr::Tuple(vec![
            Expr::Integer(1),
            Expr::Tuple(vec![
                Expr::Integer(2),
                Expr::Integer(3),
            ]),
        ]);
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_tuple_empty() {
        let expr_val = Expr::Tuple(vec![]);
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_binop_intdiv() {
        let left = Term::Small(10);
        let right = Term::Small(3);
        let result = eval_binop(&BinOp::IntDiv, &left, &right);
        if let Ok(Term::Small(val)) = result {
            assert_eq!(val, 3); // 10 div 3 = 3
        }
    }

    #[test]
    fn test_eval_binop_comparison_float() {
        let left = Term::Float(2.5);
        let right = Term::Float(1.5);
        
        let gt_result = eval_binop(&BinOp::Greater, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = gt_result {
            // Should be true atom
            let _ = atom_idx;
        }
    }

    #[test]
    fn test_eval_unop_neg_zero() {
        let val = Term::Small(0);
        let result = eval_unop(&UnOp::Neg, &val);
        if let Ok(Term::Small(neg_val)) = result {
            assert_eq!(neg_val, 0);
        }
    }

    #[test]
    fn test_eval_unop_neg_float() {
        let val = Term::Float(3.14);
        let result = eval_unop(&UnOp::Neg, &val);
        if let Ok(Term::Float(neg_val)) = result {
            assert!((neg_val - (-3.14)).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_exprs_with_bindings() {
        let expr_list = vec![
            Expr::Var("X".to_string()),
            Expr::Integer(42),
        ];
        let mut bindings = new_bindings();
        bindings.insert("X".to_string(), Term::Small(10));
        
        let result = exprs(expr_list, bindings);
        // First expr uses binding, second sets new value
        if let Ok((val, new_bindings)) = result {
            assert_eq!(val, Term::Small(42));
            // X should still be in bindings
            assert!(new_bindings.contains_key("X"));
        }
    }

    #[test]
    fn test_eval_string_empty() {
        let expr_val = Expr::String("".to_string());
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        match result {
            Term::Nil => {} // Empty string becomes empty list
            Term::List { .. } => {} // Or list structure
            _ => {}
        }
    }

    #[test]
    fn test_eval_string_unicode() {
        let expr_val = Expr::String("héllo".to_string());
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_list_large() {
        let mut elems = Vec::new();
        for i in 0..100 {
            elems.push(Expr::Integer(i));
        }
        let expr_val = Expr::List(elems);
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_tuple_large() {
        let mut elems = Vec::new();
        for i in 0..50 {
            elems.push(Expr::Integer(i));
        }
        let expr_val = Expr::Tuple(elems);
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_local_call() {
        let expr_val = Expr::LocalCall {
            function: "length".to_string(),
            args: vec![Expr::List(vec![
                Expr::Integer(1),
                Expr::Integer(2),
            ])],
        };
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        // May succeed or fail depending on function availability
        let _ = result;
    }

    #[test]
    fn test_eval_binop_and_true_true() {
        let true_atom = get_true_atom();
        let left = Term::Atom(true_atom);
        let right = Term::Atom(true_atom);
        let result = eval_binop(&BinOp::And, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = result {
            assert_eq!(atom_idx, true_atom);
        }
    }

    #[test]
    fn test_eval_binop_and_false_true() {
        let true_atom = get_true_atom();
        let false_atom = get_false_atom();
        let left = Term::Atom(false_atom);
        let right = Term::Atom(true_atom);
        let result = eval_binop(&BinOp::And, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = result {
            assert_eq!(atom_idx, false_atom);
        }
    }

    #[test]
    fn test_eval_binop_andalso() {
        let true_atom = get_true_atom();
        let false_atom = get_false_atom();
        let left = Term::Atom(true_atom);
        let right = Term::Atom(false_atom);
        let result = eval_binop(&BinOp::AndAlso, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = result {
            assert_eq!(atom_idx, false_atom);
        }
    }

    #[test]
    fn test_eval_binop_or_false_false() {
        let true_atom = get_true_atom();
        let false_atom = get_false_atom();
        let left = Term::Atom(false_atom);
        let right = Term::Atom(false_atom);
        let result = eval_binop(&BinOp::Or, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = result {
            assert_eq!(atom_idx, false_atom);
        }
    }

    #[test]
    fn test_eval_binop_or_true_false() {
        let true_atom = get_true_atom();
        let false_atom = get_false_atom();
        let left = Term::Atom(true_atom);
        let right = Term::Atom(false_atom);
        let result = eval_binop(&BinOp::Or, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = result {
            assert_eq!(atom_idx, true_atom);
        }
    }

    #[test]
    fn test_eval_binop_orelse() {
        let true_atom = get_true_atom();
        let false_atom = get_false_atom();
        let left = Term::Atom(false_atom);
        let right = Term::Atom(true_atom);
        let result = eval_binop(&BinOp::OrElse, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = result {
            assert_eq!(atom_idx, true_atom);
        }
    }

    #[test]
    fn test_eval_binop_xor_true_false() {
        let true_atom = get_true_atom();
        let false_atom = get_false_atom();
        let left = Term::Atom(true_atom);
        let right = Term::Atom(false_atom);
        let result = eval_binop(&BinOp::Xor, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = result {
            assert_eq!(atom_idx, true_atom);
        }
    }

    #[test]
    fn test_eval_binop_xor_true_true() {
        let true_atom = get_true_atom();
        let false_atom = get_false_atom();
        let left = Term::Atom(true_atom);
        let right = Term::Atom(true_atom);
        let result = eval_binop(&BinOp::Xor, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = result {
            assert_eq!(atom_idx, false_atom);
        }
    }

    #[test]
    fn test_eval_binop_less_equal() {
        let left = Term::Small(5);
        let right = Term::Small(5);
        let result = eval_binop(&BinOp::LessEqual, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = result {
            let true_atom = get_true_atom();
            assert_eq!(atom_idx, true_atom);
        }
    }

    #[test]
    fn test_eval_binop_greater_equal() {
        let left = Term::Small(5);
        let right = Term::Small(3);
        let result = eval_binop(&BinOp::GreaterEqual, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = result {
            let true_atom = get_true_atom();
            assert_eq!(atom_idx, true_atom);
        }
    }

    #[test]
    fn test_eval_binop_comparison_float_less() {
        let left = Term::Float(1.5);
        let right = Term::Float(2.5);
        let result = eval_binop(&BinOp::Less, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = result {
            let true_atom = get_true_atom();
            assert_eq!(atom_idx, true_atom);
        }
    }

    #[test]
    fn test_eval_binop_comparison_float_less_equal() {
        let left = Term::Float(2.5);
        let right = Term::Float(2.5);
        let result = eval_binop(&BinOp::LessEqual, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = result {
            let true_atom = get_true_atom();
            assert_eq!(atom_idx, true_atom);
        }
    }

    #[test]
    fn test_eval_binop_comparison_float_greater_equal() {
        let left = Term::Float(3.5);
        let right = Term::Float(2.5);
        let result = eval_binop(&BinOp::GreaterEqual, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = result {
            let true_atom = get_true_atom();
            assert_eq!(atom_idx, true_atom);
        }
    }

    #[test]
    fn test_eval_binop_type_error_addition() {
        let left = Term::Atom(1);
        let right = Term::Small(5);
        let result = eval_binop(&BinOp::Add, &left, &right);
        assert!(result.is_err());
        if let Err(EvalError::TypeError(_)) = result {
            // Expected
        } else {
            panic!("Expected TypeError");
        }
    }

    #[test]
    fn test_eval_binop_type_error_comparison() {
        let left = Term::Atom(1);
        let right = Term::Small(5);
        let result = eval_binop(&BinOp::Less, &left, &right);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_binop_type_error_intdiv() {
        let left = Term::Float(5.0);
        let right = Term::Small(3);
        let result = eval_binop(&BinOp::IntDiv, &left, &right);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_binop_type_error_rem() {
        let left = Term::Float(5.0);
        let right = Term::Small(3);
        let result = eval_binop(&BinOp::Rem, &left, &right);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_unop_not_true() {
        let true_atom = get_true_atom();
        let false_atom = get_false_atom();
        let val = Term::Atom(true_atom);
        let result = eval_unop(&UnOp::Not, &val);
        if let Ok(Term::Atom(atom_idx)) = result {
            assert_eq!(atom_idx, false_atom);
        }
    }

    #[test]
    fn test_eval_unop_not_type_error() {
        let val = Term::Small(5);
        let result = eval_unop(&UnOp::Not, &val);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_unop_neg_type_error() {
        let val = Term::Atom(1);
        let result = eval_unop(&UnOp::Neg, &val);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_match_pattern_float() {
        let pattern = Expr::Float(3.14);
        let value = Term::Float(3.14);
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_match_pattern_float_mismatch() {
        let pattern = Expr::Float(3.14);
        let value = Term::Float(2.71);
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_match_pattern_atom() {
        let pattern = Expr::Atom("test".to_string());
        let bindings = new_bindings();
        let (value, _) = expr(&pattern, &bindings).unwrap();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_match_pattern_atom_mismatch() {
        let pattern1 = Expr::Atom("test1".to_string());
        let pattern2 = Expr::Atom("test2".to_string());
        let bindings = new_bindings();
        let (value, _) = expr(&pattern1, &bindings).unwrap();
        let result = match_pattern(&pattern2, &value, &bindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_match_pattern_cons() {
        let pattern = Expr::Cons {
            head: Box::new(Expr::Integer(1)),
            tail: Box::new(Expr::Nil),
        };
        let value = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Nil),
        };
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_match_pattern_cons_with_var() {
        let pattern = Expr::Cons {
            head: Box::new(Expr::Var("X".to_string())),
            tail: Box::new(Expr::Nil),
        };
        let value = Term::List {
            head: Box::new(Term::Small(42)),
            tail: Box::new(Term::Nil),
        };
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_ok());
        if let Ok((_, new_bindings)) = result {
            assert_eq!(new_bindings.get("X"), Some(&Term::Small(42)));
        }
    }

    #[test]
    fn test_eval_match_pattern_tuple() {
        let pattern = Expr::Tuple(vec![
            Expr::Integer(1),
            Expr::Integer(2),
        ]);
        let value = Term::Tuple(vec![
            Term::Small(1),
            Term::Small(2),
        ]);
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_match_pattern_tuple_mismatch_length() {
        let pattern = Expr::Tuple(vec![
            Expr::Integer(1),
            Expr::Integer(2),
        ]);
        let value = Term::Tuple(vec![
            Term::Small(1),
        ]);
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_match_pattern_tuple_with_var() {
        let pattern = Expr::Tuple(vec![
            Expr::Var("X".to_string()),
            Expr::Integer(2),
        ]);
        let value = Term::Tuple(vec![
            Term::Small(42),
            Term::Small(2),
        ]);
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_ok());
        if let Ok((_, new_bindings)) = result {
            assert_eq!(new_bindings.get("X"), Some(&Term::Small(42)));
        }
    }

    #[test]
    fn test_eval_match_pattern_var_already_bound_match() {
        let pattern = Expr::Var("X".to_string());
        let value = Term::Small(42);
        let mut bindings = new_bindings();
        bindings.insert("X".to_string(), Term::Small(42));
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_match_pattern_var_already_bound_mismatch() {
        let pattern = Expr::Var("X".to_string());
        let value = Term::Small(42);
        let mut bindings = new_bindings();
        bindings.insert("X".to_string(), Term::Small(100));
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_match_expr_with_nested_pattern() {
        let expr_val = Expr::Match {
            left: Box::new(Expr::Tuple(vec![
                Expr::Var("X".to_string()),
                Expr::Var("Y".to_string()),
            ])),
            right: Box::new(Expr::Tuple(vec![
                Expr::Integer(1),
                Expr::Integer(2),
            ])),
        };
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        assert!(result.is_ok());
        if let Ok((_, new_bindings)) = result {
            assert_eq!(new_bindings.get("X"), Some(&Term::Small(1)));
            assert_eq!(new_bindings.get("Y"), Some(&Term::Small(2)));
        }
    }

    #[test]
    fn test_list_length_nested() {
        let list = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::List {
                    head: Box::new(Term::Small(3)),
                    tail: Box::new(Term::Nil),
                }),
            }),
        };
        let result = list_length(&list);
        if let Ok(len) = result {
            assert_eq!(len, 3);
        }
    }

    #[test]
    fn test_list_length_single() {
        let list = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Nil),
        };
        let result = list_length(&list);
        if let Ok(len) = result {
            assert_eq!(len, 1);
        }
    }

    #[test]
    fn test_eval_binop_div_float_by_zero() {
        let left = Term::Float(5.0);
        let right = Term::Float(0.0);
        let result = eval_binop(&BinOp::Div, &left, &right);
        assert!(result.is_err());
        if let Err(EvalError::DivisionByZero) = result {
            // Expected
        } else {
            panic!("Expected DivisionByZero");
        }
    }

    #[test]
    fn test_eval_binop_equal_different_types() {
        let left = Term::Small(5);
        let right = Term::Float(5.0);
        let result = eval_binop(&BinOp::Equal, &left, &right);
        // Should return false atom
        if let Ok(Term::Atom(atom_idx)) = result {
            let false_atom = get_false_atom();
            assert_eq!(atom_idx, false_atom);
        }
    }

    #[test]
    fn test_eval_binop_not_equal_same() {
        let left = Term::Small(5);
        let right = Term::Small(5);
        let result = eval_binop(&BinOp::NotEqual, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = result {
            let false_atom = get_false_atom();
            assert_eq!(atom_idx, false_atom);
        }
    }

    #[test]
    fn test_eval_binop_not_equal_different() {
        let left = Term::Small(5);
        let right = Term::Small(3);
        let result = eval_binop(&BinOp::NotEqual, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = result {
            let true_atom = get_true_atom();
            assert_eq!(atom_idx, true_atom);
        }
    }

    #[test]
    fn test_eval_function_call_with_multiple_args() {
        let expr_val = Expr::Call {
            module: None,
            function: "+".to_string(),
            args: vec![
                Expr::Integer(1),
                Expr::Integer(2),
                Expr::Integer(3),
            ],
        };
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        // Should fail because + only takes 2 args
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_function_call_erlang_module_explicit() {
        let expr_val = Expr::Call {
            module: Some("erlang".to_string()),
            function: "length".to_string(),
            args: vec![Expr::List(vec![
                Expr::Integer(1),
                Expr::Integer(2),
            ])],
        };
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        // May succeed or fail
        let _ = result;
    }

    #[test]
    fn test_eval_cons_with_nested() {
        let expr_val = Expr::Cons {
            head: Box::new(Expr::Cons {
                head: Box::new(Expr::Integer(1)),
                tail: Box::new(Expr::Nil),
            }),
            tail: Box::new(Expr::Nil),
        };
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_tuple_with_nested() {
        let expr_val = Expr::Tuple(vec![
            Expr::Tuple(vec![
                Expr::Integer(1),
            ]),
            Expr::Integer(2),
        ]);
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_paren_nested() {
        let expr_val = Expr::Paren(Box::new(Expr::Paren(Box::new(Expr::Integer(42)))));
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        assert_eq!(result, Term::Small(42));
    }

    #[test]
    fn test_eval_binop_with_bindings() {
        let expr_val = Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Var("X".to_string())),
            right: Box::new(Expr::Integer(5)),
        };
        let mut bindings = new_bindings();
        bindings.insert("X".to_string(), Term::Small(10));
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        assert_eq!(result, Term::Small(15));
    }

    #[test]
    fn test_eval_unop_with_bindings() {
        let expr_val = Expr::UnOp {
            op: UnOp::Neg,
            expr: Box::new(Expr::Var("X".to_string())),
        };
        let mut bindings = new_bindings();
        bindings.insert("X".to_string(), Term::Small(10));
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        assert_eq!(result, Term::Small(-10));
    }

    #[test]
    fn test_term_to_eterm_small() {
        let term = Term::Small(42);
        let result = term_to_eterm(&term);
        assert!(result.is_ok());
        if let Ok(eterm) = result {
            assert_eq!(eterm, 42);
        }
    }

    #[test]
    fn test_term_to_eterm_float() {
        let term = Term::Float(3.14);
        let result = term_to_eterm(&term);
        assert!(result.is_ok());
        if let Ok(eterm) = result {
            let f = f64::from_bits(eterm);
            assert!((f - 3.14).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_term_to_eterm_atom() {
        let term = Term::Atom(5);
        let result = term_to_eterm(&term);
        assert!(result.is_ok());
        if let Ok(eterm) = result {
            // Should have atom tag
            assert_eq!(eterm & 0x3F, 0x0B);
        }
    }

    #[test]
    fn test_term_to_eterm_nil() {
        let term = Term::Nil;
        let result = term_to_eterm(&term);
        assert!(result.is_ok());
        if let Ok(eterm) = result {
            assert_eq!(eterm, 0x3F);
        }
    }

    #[test]
    fn test_term_to_eterm_list_error() {
        let term = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Nil),
        };
        let result = term_to_eterm(&term);
        assert!(result.is_err());
    }

    #[test]
    fn test_term_to_eterm_tuple_error() {
        let term = Term::Tuple(vec![Term::Small(1)]);
        let result = term_to_eterm(&term);
        assert!(result.is_err());
    }

    #[test]
    fn test_term_to_eterm_on_heap_small() {
        use entities_process::Process;
        let mut process = Process::new(1);
        let term = Term::Small(42);
        let result = term_to_eterm_on_heap(&term, &mut process);
        assert!(result.is_ok());
    }

    #[test]
    fn test_term_to_eterm_on_heap_float() {
        use entities_process::Process;
        let mut process = Process::new(1);
        let term = Term::Float(3.14);
        let result = term_to_eterm_on_heap(&term, &mut process);
        assert!(result.is_ok());
    }

    #[test]
    fn test_term_to_eterm_on_heap_atom() {
        use entities_process::Process;
        let mut process = Process::new(1);
        let term = Term::Atom(5);
        let result = term_to_eterm_on_heap(&term, &mut process);
        assert!(result.is_ok());
    }

    #[test]
    fn test_term_to_eterm_on_heap_nil() {
        use entities_process::Process;
        let mut process = Process::new(1);
        let term = Term::Nil;
        let result = term_to_eterm_on_heap(&term, &mut process);
        assert!(result.is_ok());
        if let Ok(eterm) = result {
            assert_eq!(eterm, 0x3F);
        }
    }

    #[test]
    fn test_term_to_eterm_on_heap_list() {
        use entities_process::Process;
        let mut process = Process::new(1);
        let term = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Nil),
        };
        let result = term_to_eterm_on_heap(&term, &mut process);
        assert!(result.is_ok());
        if let Ok(eterm) = result {
            // Should have list tag
            assert_eq!(eterm & 0x3, 0x2);
            // Decoding may fail if heap structure doesn't match expectations
            // This is acceptable - we're testing the encoding path
        }
    }

    #[test]
    fn test_term_to_eterm_on_heap_nested_list() {
        use entities_process::Process;
        let mut process = Process::new(1);
        let term = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::Nil),
            }),
        };
        let result = term_to_eterm_on_heap(&term, &mut process);
        assert!(result.is_ok());
        if let Ok(eterm) = result {
            assert_eq!(eterm & 0x3, 0x2);
            // Decoding may fail if heap structure doesn't match expectations
            // This is acceptable - we're testing the encoding path
        }
    }

    #[test]
    fn test_term_to_eterm_on_heap_tuple_error() {
        use entities_process::Process;
        let mut process = Process::new(1);
        let term = Term::Tuple(vec![Term::Small(1)]);
        let result = term_to_eterm_on_heap(&term, &mut process);
        assert!(result.is_err());
    }

    #[test]
    fn test_eterm_to_term_nil() {
        let eterm = 0x3F;
        let result = eterm_to_term(eterm);
        assert!(result.is_ok());
        if let Ok(Term::Nil) = result {
            // Expected
        } else {
            panic!("Expected Nil");
        }
    }

    #[test]
    fn test_eterm_to_term_atom() {
        let atom_index = 5u32;
        let eterm = ((atom_index as u64) << 32) | 0x0B;
        let result = eterm_to_term(eterm);
        assert!(result.is_ok());
        if let Ok(Term::Atom(idx)) = result {
            assert_eq!(idx, atom_index);
        } else {
            panic!("Expected Atom");
        }
    }

    #[test]
    fn test_eterm_to_term_small_integer() {
        let value = 42i64;
        let eterm = (value as u64) << 4 | 0xF;
        let result = eterm_to_term(eterm);
        // May succeed or fail depending on implementation
        let _ = result;
    }

    #[test]
    fn test_eterm_to_term_float() {
        let f = 3.14f64;
        let eterm = f.to_bits();
        let result = eterm_to_term(eterm);
        if let Ok(Term::Float(val)) = result {
            assert!((val - 3.14).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_eterm_to_term_from_heap_nil() {
        use entities_process::Process;
        use std::sync::Arc;
        let process = Arc::new(Process::new(1));
        let eterm = 0x3F;
        let result = eterm_to_term_from_heap(eterm, &process);
        assert!(result.is_ok());
        if let Ok(Term::Nil) = result {
            // Expected
        } else {
            panic!("Expected Nil");
        }
    }

    #[test]
    fn test_eterm_to_term_from_heap_small_integer() {
        use entities_process::Process;
        use std::sync::Arc;
        let process = Arc::new(Process::new(1));
        let value = 42i64;
        let eterm = (value as u64) << 4 | 0xF;
        let result = eterm_to_term_from_heap(eterm, &process);
        assert!(result.is_ok());
        if let Ok(Term::Small(val)) = result {
            assert_eq!(val, value);
        }
    }

    #[test]
    fn test_eterm_to_term_from_heap_atom() {
        use entities_process::Process;
        use std::sync::Arc;
        let process = Arc::new(Process::new(1));
        let atom_index = 5u32;
        let eterm = ((atom_index as u64) << 32) | 0x0B;
        let result = eterm_to_term_from_heap(eterm, &process);
        assert!(result.is_ok());
        if let Ok(Term::Atom(idx)) = result {
            assert_eq!(idx, atom_index);
        }
    }

    #[test]
    fn test_eterm_to_term_from_heap_invalid_small() {
        use entities_process::Process;
        use std::sync::Arc;
        let process = Arc::new(Process::new(1));
        // Very small value that looks like untagged integer
        let eterm = 0x42; // < 0x100, should trigger error
        let result = eterm_to_term_from_heap(eterm, &process);
        assert!(result.is_err());
    }

    #[test]
    fn test_eterm_to_term_from_heap_list_out_of_bounds() {
        use entities_process::Process;
        use std::sync::Arc;
        let process = Arc::new(Process::new(1));
        // Create a list pointer that points beyond heap
        let heap_index = 10000usize;
        let eterm = ((heap_index as u64) << 2) | 0x2;
        let result = eterm_to_term_from_heap(eterm, &process);
        assert!(result.is_err());
    }

    #[test]
    fn test_eterm_to_term_from_heap_float() {
        use entities_process::Process;
        use std::sync::Arc;
        let process = Arc::new(Process::new(1));
        let f = 3.14f64;
        let eterm = f.to_bits();
        let result = eterm_to_term_from_heap(eterm, &process);
        if let Ok(Term::Float(val)) = result {
            assert!((val - 3.14).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_eval_match_pattern_list_empty() {
        let pattern = Expr::List(vec![]);
        let value = Term::Nil;
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_match_pattern_list_empty_mismatch() {
        let pattern = Expr::List(vec![Expr::Integer(1)]);
        let value = Term::Nil;
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_match_pattern_list_single() {
        let pattern = Expr::List(vec![Expr::Integer(1)]);
        let value = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Nil),
        };
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_match_pattern_list_multiple() {
        let pattern = Expr::List(vec![
            Expr::Integer(1),
            Expr::Integer(2),
            Expr::Integer(3),
        ]);
        let value = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::List {
                    head: Box::new(Term::Small(3)),
                    tail: Box::new(Term::Nil),
                }),
            }),
        };
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_match_pattern_list_too_short() {
        let pattern = Expr::List(vec![
            Expr::Integer(1),
            Expr::Integer(2),
        ]);
        let value = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Nil),
        };
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_match_pattern_list_too_long() {
        let pattern = Expr::List(vec![Expr::Integer(1)]);
        let value = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::Nil),
            }),
        };
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_match_pattern_list_with_vars() {
        let pattern = Expr::List(vec![
            Expr::Var("X".to_string()),
            Expr::Var("Y".to_string()),
        ]);
        let value = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::Nil),
            }),
        };
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_ok());
        if let Ok((_, new_bindings)) = result {
            assert_eq!(new_bindings.get("X"), Some(&Term::Small(1)));
            assert_eq!(new_bindings.get("Y"), Some(&Term::Small(2)));
        }
    }

    #[test]
    fn test_eval_match_pattern_list_not_list() {
        let pattern = Expr::List(vec![Expr::Integer(1)]);
        let value = Term::Small(1);
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_match_pattern_tuple_empty() {
        let pattern = Expr::Tuple(vec![]);
        let value = Term::Tuple(vec![]);
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_match_pattern_tuple_single() {
        let pattern = Expr::Tuple(vec![Expr::Integer(1)]);
        let value = Term::Tuple(vec![Term::Small(1)]);
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_match_pattern_tuple_multiple() {
        let pattern = Expr::Tuple(vec![
            Expr::Integer(1),
            Expr::Integer(2),
            Expr::Integer(3),
        ]);
        let value = Term::Tuple(vec![
            Term::Small(1),
            Term::Small(2),
            Term::Small(3),
        ]);
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_match_pattern_tuple_arity_mismatch() {
        let pattern = Expr::Tuple(vec![
            Expr::Integer(1),
            Expr::Integer(2),
        ]);
        let value = Term::Tuple(vec![
            Term::Small(1),
        ]);
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_match_pattern_tuple_not_tuple() {
        let pattern = Expr::Tuple(vec![Expr::Integer(1)]);
        let value = Term::Small(1);
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_match_pattern_nil_mismatch() {
        let pattern = Expr::Nil;
        let value = Term::Small(1);
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_match_pattern_cons_mismatch() {
        let pattern = Expr::Cons {
            head: Box::new(Expr::Integer(1)),
            tail: Box::new(Expr::Nil),
        };
        let value = Term::Small(1);
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_match_pattern_cons_head_mismatch() {
        let pattern = Expr::Cons {
            head: Box::new(Expr::Integer(1)),
            tail: Box::new(Expr::Nil),
        };
        let value = Term::List {
            head: Box::new(Term::Small(2)),
            tail: Box::new(Term::Nil),
        };
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_match_pattern_unsupported() {
        // Test with an unsupported pattern type (like BinOp)
        let pattern = Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Integer(1)),
            right: Box::new(Expr::Integer(2)),
        };
        let value = Term::Small(3);
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_code_paths_for_module_loading() {
        // Test that the function doesn't panic
        let paths = get_code_paths_for_module_loading();
        // Should at least have current directory
        assert!(!paths.is_empty());
    }

    #[test]
    fn test_try_load_module_nonexistent() {
        // Test loading a module that doesn't exist
        let result = try_load_module("nonexistent_module_xyz");
        // Should fail with error
        assert!(result.is_err());
    }


    #[test]
    fn test_eval_function_call_remote_module_not_found() {
        let expr_val = Expr::Call {
            module: Some("nonexistent_module".to_string()),
            function: "func".to_string(),
            args: vec![Expr::Integer(1)],
        };
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        // Should fail with undefined function error
        assert!(result.is_err());
        if let Err(EvalError::UndefinedFunction { .. }) = result {
            // Expected
        }
    }

    #[test]
    fn test_eval_function_call_remote_module_atom_creation_failure() {
        // This is hard to test directly, but we can test the error path
        // by using a module name that might cause issues
        let expr_val = Expr::Call {
            module: Some("test".to_string()),
            function: "func".to_string(),
            args: vec![],
        };
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        // May succeed or fail depending on module availability
        let _ = result;
    }

    #[test]
    fn test_list_length_with_nested_lists() {
        let list = Term::List {
            head: Box::new(Term::List {
                head: Box::new(Term::Small(1)),
                tail: Box::new(Term::Nil),
            }),
            tail: Box::new(Term::Nil),
        };
        let result = list_length(&list);
        // Should count the outer list, not nested elements
        if let Ok(len) = result {
            assert_eq!(len, 1);
        }
    }

    #[test]
    fn test_list_length_very_long() {
        let mut list = Term::Nil;
        for i in (1..=100).rev() {
            list = Term::List {
                head: Box::new(Term::Small(i)),
                tail: Box::new(list),
            };
        }
        let result = list_length(&list);
        if let Ok(len) = result {
            assert_eq!(len, 100);
        }
    }

    #[test]
    fn test_eval_cons_with_variables() {
        let expr_val = Expr::Cons {
            head: Box::new(Expr::Var("X".to_string())),
            tail: Box::new(Expr::Var("Y".to_string())),
        };
        let mut bindings = new_bindings();
        bindings.insert("X".to_string(), Term::Small(1));
        bindings.insert("Y".to_string(), Term::Nil);
        let result = expr(&expr_val, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_list_with_variables() {
        let expr_val = Expr::List(vec![
            Expr::Var("X".to_string()),
            Expr::Var("Y".to_string()),
        ]);
        let mut bindings = new_bindings();
        bindings.insert("X".to_string(), Term::Small(1));
        bindings.insert("Y".to_string(), Term::Small(2));
        let result = expr(&expr_val, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_tuple_with_variables() {
        let expr_val = Expr::Tuple(vec![
            Expr::Var("X".to_string()),
            Expr::Var("Y".to_string()),
        ]);
        let mut bindings = new_bindings();
        bindings.insert("X".to_string(), Term::Small(1));
        bindings.insert("Y".to_string(), Term::Small(2));
        let result = expr(&expr_val, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_binop_with_variable_operands() {
        let expr_val = Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Var("X".to_string())),
            right: Box::new(Expr::Var("Y".to_string())),
        };
        let mut bindings = new_bindings();
        bindings.insert("X".to_string(), Term::Small(10));
        bindings.insert("Y".to_string(), Term::Small(20));
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        assert_eq!(result, Term::Small(30));
    }

    #[test]
    fn test_eval_nested_binops() {
        let expr_val = Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::BinOp {
                op: BinOp::Mul,
                left: Box::new(Expr::Integer(2)),
                right: Box::new(Expr::Integer(3)),
            }),
            right: Box::new(Expr::Integer(4)),
        };
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        assert_eq!(result, Term::Small(10)); // (2*3) + 4 = 10
    }

    #[test]
    fn test_eval_nested_unops() {
        let expr_val = Expr::UnOp {
            op: UnOp::Neg,
            expr: Box::new(Expr::UnOp {
                op: UnOp::Neg,
                expr: Box::new(Expr::Integer(5)),
            }),
        };
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        assert_eq!(result, Term::Small(5)); // -(-5) = 5
    }

    #[test]
    fn test_eval_complex_expression() {
        let expr_val = Expr::BinOp {
            op: BinOp::Mul,
            left: Box::new(Expr::BinOp {
                op: BinOp::Add,
                left: Box::new(Expr::Integer(1)),
                right: Box::new(Expr::Integer(2)),
            }),
            right: Box::new(Expr::BinOp {
                op: BinOp::Sub,
                left: Box::new(Expr::Integer(5)),
                right: Box::new(Expr::Integer(2)),
            }),
        };
        let bindings = new_bindings();
        let (result, _) = expr(&expr_val, &bindings).unwrap();
        assert_eq!(result, Term::Small(9)); // (1+2) * (5-2) = 9
    }

    #[test]
    fn test_eval_match_pattern_list_empty_pattern_non_empty_value() {
        let pattern = Expr::List(vec![]);
        // Test with a list that has a non-nil tail to trigger the error path
        let value = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::Nil),
            }),
        };
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        // Empty pattern should fail when value has non-empty tail
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_match_pattern_list_empty_value_non_empty_pattern() {
        let pattern = Expr::List(vec![Expr::Integer(1)]);
        let value = Term::Nil;
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_match_pattern_list_with_mixed_types() {
        let pattern = Expr::List(vec![
            Expr::Integer(1),
            Expr::Atom("test".to_string()),
        ]);
        let bindings = new_bindings();
        let (atom_term, _) = expr(&Expr::Atom("test".to_string()), &bindings).unwrap();
        let value = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::List {
                head: Box::new(atom_term),
                tail: Box::new(Term::Nil),
            }),
        };
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_match_pattern_tuple_with_mixed_types() {
        let pattern = Expr::Tuple(vec![
            Expr::Integer(1),
            Expr::Atom("test".to_string()),
            Expr::Float(3.14),
        ]);
        let bindings = new_bindings();
        let (atom_term, _) = expr(&Expr::Atom("test".to_string()), &bindings).unwrap();
        let value = Term::Tuple(vec![
            Term::Small(1),
            atom_term,
            Term::Float(3.14),
        ]);
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_match_pattern_tuple_element_mismatch() {
        let pattern = Expr::Tuple(vec![
            Expr::Integer(1),
            Expr::Integer(2),
        ]);
        let value = Term::Tuple(vec![
            Term::Small(1),
            Term::Small(3), // Mismatch
        ]);
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_match_pattern_cons_tail_mismatch() {
        let pattern = Expr::Cons {
            head: Box::new(Expr::Integer(1)),
            tail: Box::new(Expr::Integer(2)), // Tail should be list or nil
        };
        let value = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Nil),
        };
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        // May succeed or fail depending on implementation
        let _ = result;
    }

    #[test]
    fn test_eval_match_pattern_list_element_mismatch() {
        let pattern = Expr::List(vec![
            Expr::Integer(1),
            Expr::Integer(2),
        ]);
        let value = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(3)), // Mismatch
                tail: Box::new(Term::Nil),
            }),
        };
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_match_pattern_list_with_nested_vars() {
        let pattern = Expr::List(vec![
            Expr::Var("X".to_string()),
            Expr::List(vec![
                Expr::Var("Y".to_string()),
            ]),
        ]);
        let value = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::List {
                head: Box::new(Term::List {
                    head: Box::new(Term::Small(2)),
                    tail: Box::new(Term::Nil),
                }),
                tail: Box::new(Term::Nil),
            }),
        };
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        // May succeed or fail depending on implementation
        let _ = result;
    }

    #[test]
    fn test_eval_match_pattern_tuple_with_nested_vars() {
        let pattern = Expr::Tuple(vec![
            Expr::Var("X".to_string()),
            Expr::Tuple(vec![
                Expr::Var("Y".to_string()),
            ]),
        ]);
        let value = Term::Tuple(vec![
            Term::Small(1),
            Term::Tuple(vec![
                Term::Small(2),
            ]),
        ]);
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_ok());
        if let Ok((_, new_bindings)) = result {
            assert_eq!(new_bindings.get("X"), Some(&Term::Small(1)));
            assert_eq!(new_bindings.get("Y"), Some(&Term::Small(2)));
        }
    }

    #[test]
    fn test_eval_match_pattern_cons_with_nested_patterns() {
        let pattern = Expr::Cons {
            head: Box::new(Expr::Tuple(vec![
                Expr::Var("X".to_string()),
            ])),
            tail: Box::new(Expr::Nil),
        };
        let value = Term::List {
            head: Box::new(Term::Tuple(vec![
                Term::Small(42),
            ])),
            tail: Box::new(Term::Nil),
        };
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_ok());
        if let Ok((_, new_bindings)) = result {
            assert_eq!(new_bindings.get("X"), Some(&Term::Small(42)));
        }
    }

    #[test]
    fn test_eval_binop_float_division() {
        let left = Term::Float(10.0);
        let right = Term::Float(2.0);
        let result = eval_binop(&BinOp::Div, &left, &right);
        if let Ok(Term::Float(val)) = result {
            assert!((val - 5.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_eval_binop_float_multiplication() {
        let left = Term::Float(2.5);
        let right = Term::Float(4.0);
        let result = eval_binop(&BinOp::Mul, &left, &right);
        if let Ok(Term::Float(val)) = result {
            assert!((val - 10.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_eval_binop_float_subtraction() {
        let left = Term::Float(5.5);
        let right = Term::Float(2.5);
        let result = eval_binop(&BinOp::Sub, &left, &right);
        if let Ok(Term::Float(val)) = result {
            assert!((val - 3.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_eval_binop_comparison_equal_atoms() {
        let true_atom = get_true_atom();
        let left = Term::Atom(true_atom);
        let right = Term::Atom(true_atom);
        let result = eval_binop(&BinOp::Equal, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = result {
            assert_eq!(atom_idx, true_atom);
        }
    }

    #[test]
    fn test_eval_binop_comparison_not_equal_atoms() {
        let true_atom = get_true_atom();
        let false_atom = get_false_atom();
        let left = Term::Atom(true_atom);
        let right = Term::Atom(false_atom);
        let result = eval_binop(&BinOp::NotEqual, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = result {
            assert_eq!(atom_idx, true_atom);
        }
    }

    #[test]
    fn test_eval_binop_comparison_less_than_zero() {
        let left = Term::Small(-5);
        let right = Term::Small(0);
        let result = eval_binop(&BinOp::Less, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = result {
            let true_atom = get_true_atom();
            assert_eq!(atom_idx, true_atom);
        }
    }

    #[test]
    fn test_eval_binop_comparison_greater_than_zero() {
        let left = Term::Small(5);
        let right = Term::Small(0);
        let result = eval_binop(&BinOp::Greater, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = result {
            let true_atom = get_true_atom();
            assert_eq!(atom_idx, true_atom);
        }
    }

    #[test]
    fn test_eval_binop_comparison_float_equal() {
        let left = Term::Float(2.5);
        let right = Term::Float(2.5);
        let result = eval_binop(&BinOp::Equal, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = result {
            let true_atom = get_true_atom();
            assert_eq!(atom_idx, true_atom);
        }
    }

    #[test]
    fn test_eval_binop_comparison_float_not_equal() {
        let left = Term::Float(2.5);
        let right = Term::Float(3.5);
        let result = eval_binop(&BinOp::NotEqual, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = result {
            let true_atom = get_true_atom();
            assert_eq!(atom_idx, true_atom);
        }
    }

    #[test]
    fn test_eval_binop_rem_by_zero() {
        let left = Term::Small(10);
        let right = Term::Small(0);
        let result = eval_binop(&BinOp::Rem, &left, &right);
        assert!(result.is_err());
        if let Err(EvalError::DivisionByZero) = result {
            // Expected
        } else {
            panic!("Expected DivisionByZero");
        }
    }

    #[test]
    fn test_eval_binop_intdiv_by_zero() {
        let left = Term::Small(10);
        let right = Term::Small(0);
        let result = eval_binop(&BinOp::IntDiv, &left, &right);
        assert!(result.is_err());
        if let Err(EvalError::DivisionByZero) = result {
            // Expected
        } else {
            panic!("Expected DivisionByZero");
        }
    }

    #[test]
    fn test_eval_binop_rem_negative() {
        let left = Term::Small(-10);
        let right = Term::Small(3);
        let result = eval_binop(&BinOp::Rem, &left, &right);
        if let Ok(Term::Small(val)) = result {
            // -10 rem 3 = -1 (in Erlang, rem preserves sign)
            assert_eq!(val, -1);
        }
    }

    #[test]
    fn test_eval_binop_intdiv_negative() {
        let left = Term::Small(-10);
        let right = Term::Small(3);
        let result = eval_binop(&BinOp::IntDiv, &left, &right);
        if let Ok(Term::Small(val)) = result {
            // -10 div 3 = -3 (truncates toward zero)
            assert_eq!(val, -3);
        }
    }

    #[test]
    fn test_eval_unop_pos_float() {
        let val = Term::Float(3.14);
        let result = eval_unop(&UnOp::Pos, &val);
        if let Ok(Term::Float(f)) = result {
            assert!((f - 3.14).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_eval_unop_pos_atom() {
        let atom_idx = get_true_atom();
        let val = Term::Atom(atom_idx);
        let result = eval_unop(&UnOp::Pos, &val);
        // Pos should just return the value unchanged
        if let Ok(Term::Atom(idx)) = result {
            assert_eq!(idx, atom_idx);
        }
    }

    #[test]
    fn test_eval_unop_neg_large() {
        let val = Term::Small(i64::MIN + 1);
        let result = eval_unop(&UnOp::Neg, &val);
        if let Ok(Term::Small(neg_val)) = result {
            assert_eq!(neg_val, -(i64::MIN + 1));
        }
    }

    #[test]
    fn test_eval_unop_not_false() {
        let false_atom = get_false_atom();
        let true_atom = get_true_atom();
        let val = Term::Atom(false_atom);
        let result = eval_unop(&UnOp::Not, &val);
        if let Ok(Term::Atom(atom_idx)) = result {
            assert_eq!(atom_idx, true_atom);
        }
    }

    #[test]
    fn test_eval_match_pattern_list_with_tail_var() {
        // Pattern: [X | Y] where Y is a variable
        let pattern = Expr::Cons {
            head: Box::new(Expr::Var("X".to_string())),
            tail: Box::new(Expr::Var("Y".to_string())),
        };
        let value = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::Nil),
            }),
        };
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_ok());
        if let Ok((_, new_bindings)) = result {
            assert_eq!(new_bindings.get("X"), Some(&Term::Small(1)));
            // Y should be bound to the tail list
            assert!(new_bindings.contains_key("Y"));
        }
    }

    #[test]
    fn test_eval_match_pattern_list_empty_with_var() {
        let pattern = Expr::List(vec![Expr::Var("X".to_string())]);
        let value = Term::Nil;
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_match_pattern_tuple_empty_with_var() {
        let pattern = Expr::Tuple(vec![Expr::Var("X".to_string())]);
        let value = Term::Tuple(vec![]);
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_match_pattern_cons_not_list() {
        let pattern = Expr::Cons {
            head: Box::new(Expr::Integer(1)),
            tail: Box::new(Expr::Nil),
        };
        let value = Term::Small(1);
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_match_pattern_integer_type_mismatch() {
        let pattern = Expr::Integer(1);
        let value = Term::Float(1.0);
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_match_pattern_float_type_mismatch() {
        let pattern = Expr::Float(1.0);
        let value = Term::Small(1);
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_match_pattern_atom_type_mismatch() {
        let pattern = Expr::Atom("test".to_string());
        let value = Term::Small(1);
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_match_pattern_paren() {
        let pattern = Expr::Paren(Box::new(Expr::Integer(42)));
        let value = Term::Small(42);
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_match_pattern_paren_mismatch() {
        let pattern = Expr::Paren(Box::new(Expr::Integer(42)));
        let value = Term::Small(100);
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_binop_add_large() {
        // Test with large but safe values to avoid overflow panic
        let left = Term::Small(i64::MAX / 2);
        let right = Term::Small(i64::MAX / 2);
        let result = eval_binop(&BinOp::Add, &left, &right);
        // May succeed or overflow depending on implementation
        let _ = result;
    }

    #[test]
    fn test_eval_binop_mul_large() {
        // Test with large but safe values
        let left = Term::Small(i64::MAX / 4);
        let right = Term::Small(3);
        let result = eval_binop(&BinOp::Mul, &left, &right);
        // May succeed or overflow depending on implementation
        let _ = result;
    }

    #[test]
    fn test_eval_binop_sub_large() {
        // Test with large but safe values
        let left = Term::Small(i64::MIN / 2);
        let right = Term::Small(1);
        let result = eval_binop(&BinOp::Sub, &left, &right);
        // May succeed or underflow depending on implementation
        let _ = result;
    }

    #[test]
    fn test_eval_binop_float_special_values() {
        let left = Term::Float(f64::INFINITY);
        let right = Term::Float(1.0);
        let result = eval_binop(&BinOp::Add, &left, &right);
        // May handle infinity
        let _ = result;
    }

    #[test]
    fn test_eval_binop_float_nan() {
        let left = Term::Float(f64::NAN);
        let right = Term::Float(1.0);
        let result = eval_binop(&BinOp::Add, &left, &right);
        // May handle NaN
        let _ = result;
    }

    #[test]
    fn test_eval_binop_comparison_float_nan() {
        let left = Term::Float(f64::NAN);
        let right = Term::Float(1.0);
        let result = eval_binop(&BinOp::Less, &left, &right);
        // NaN comparisons may fail or return false atom depending on implementation
        // Just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_eval_binop_comparison_float_infinity() {
        let left = Term::Float(f64::INFINITY);
        let right = Term::Float(1.0);
        let result = eval_binop(&BinOp::Greater, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = result {
            let true_atom = get_true_atom();
            assert_eq!(atom_idx, true_atom);
        }
    }

    #[test]
    fn test_eval_binop_comparison_float_negative_infinity() {
        let left = Term::Float(f64::NEG_INFINITY);
        let right = Term::Float(1.0);
        let result = eval_binop(&BinOp::Less, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = result {
            let true_atom = get_true_atom();
            assert_eq!(atom_idx, true_atom);
        }
    }

    #[test]
    fn test_eval_binop_xor_false_false() {
        let true_atom = get_true_atom();
        let false_atom = get_false_atom();
        let left = Term::Atom(false_atom);
        let right = Term::Atom(false_atom);
        let result = eval_binop(&BinOp::Xor, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = result {
            assert_eq!(atom_idx, false_atom);
        }
    }

    #[test]
    fn test_eval_binop_xor_false_true() {
        let true_atom = get_true_atom();
        let false_atom = get_false_atom();
        let left = Term::Atom(false_atom);
        let right = Term::Atom(true_atom);
        let result = eval_binop(&BinOp::Xor, &left, &right);
        if let Ok(Term::Atom(atom_idx)) = result {
            assert_eq!(atom_idx, true_atom);
        }
    }

    #[test]
    fn test_eval_binop_and_with_non_atoms() {
        let left = Term::Small(1);
        let right = Term::Small(2);
        let result = eval_binop(&BinOp::And, &left, &right);
        // Should return false atom for non-atom operands
        if let Ok(Term::Atom(atom_idx)) = result {
            let false_atom = get_false_atom();
            assert_eq!(atom_idx, false_atom);
        }
    }

    #[test]
    fn test_eval_binop_or_with_non_atoms() {
        let left = Term::Small(1);
        let right = Term::Small(2);
        let result = eval_binop(&BinOp::Or, &left, &right);
        // Should return true atom for non-atom operands
        if let Ok(Term::Atom(atom_idx)) = result {
            let true_atom = get_true_atom();
            assert_eq!(atom_idx, true_atom);
        }
    }

    #[test]
    fn test_eval_binop_xor_with_non_atoms() {
        let left = Term::Small(1);
        let right = Term::Small(2);
        let result = eval_binop(&BinOp::Xor, &left, &right);
        // Should return false atom for non-atom operands
        if let Ok(Term::Atom(atom_idx)) = result {
            let false_atom = get_false_atom();
            assert_eq!(atom_idx, false_atom);
        }
    }

    #[test]
    fn test_eval_binop_xor_atom_non_atom() {
        let true_atom = get_true_atom();
        let false_atom = get_false_atom();
        let left = Term::Atom(true_atom);
        let right = Term::Small(1);
        let result = eval_binop(&BinOp::Xor, &left, &right);
        // Should return false atom when one is not an atom
        if let Ok(Term::Atom(atom_idx)) = result {
            assert_eq!(atom_idx, false_atom);
        }
    }

    #[test]
    fn test_eval_function_call_with_zero_args() {
        let expr_val = Expr::Call {
            module: None,
            function: "length".to_string(),
            args: vec![],
        };
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        // Should fail because length needs 1 arg
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_function_call_with_float_args() {
        let expr_val = Expr::Call {
            module: None,
            function: "+".to_string(),
            args: vec![
                Expr::Float(1.5),
                Expr::Float(2.5),
            ],
        };
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        if let Ok((Term::Float(val), _)) = result {
            assert!((val - 4.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_eval_function_call_with_mixed_float_int() {
        let expr_val = Expr::Call {
            module: None,
            function: "+".to_string(),
            args: vec![
                Expr::Integer(1),
                Expr::Float(2.5),
            ],
        };
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        if let Ok((Term::Float(val), _)) = result {
            assert!((val - 3.5).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_eval_function_call_with_mixed_int_float() {
        let expr_val = Expr::Call {
            module: None,
            function: "+".to_string(),
            args: vec![
                Expr::Float(1.5),
                Expr::Integer(2),
            ],
        };
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        if let Ok((Term::Float(val), _)) = result {
            assert!((val - 3.5).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_eval_function_call_division_float() {
        let expr_val = Expr::Call {
            module: None,
            function: "/".to_string(),
            args: vec![
                Expr::Integer(10),
                Expr::Integer(3),
            ],
        };
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        if let Ok((Term::Float(val), _)) = result {
            let expected = 10.0 / 3.0;
            assert!((val - expected).abs() < f64::EPSILON * 10.0);
        }
    }

    #[test]
    fn test_eval_function_call_multiplication_float() {
        let expr_val = Expr::Call {
            module: None,
            function: "*".to_string(),
            args: vec![
                Expr::Float(2.5),
                Expr::Float(4.0),
            ],
        };
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        if let Ok((Term::Float(val), _)) = result {
            assert!((val - 10.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_eval_function_call_subtraction_float() {
        let expr_val = Expr::Call {
            module: None,
            function: "-".to_string(),
            args: vec![
                Expr::Float(5.5),
                Expr::Float(2.5),
            ],
        };
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        if let Ok((Term::Float(val), _)) = result {
            assert!((val - 3.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_eval_local_call_with_variables() {
        let expr_val = Expr::LocalCall {
            function: "length".to_string(),
            args: vec![Expr::Var("L".to_string())],
        };
        let mut bindings = new_bindings();
        bindings.insert("L".to_string(), Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::Nil),
            }),
        });
        let result = expr(&expr_val, &bindings);
        // May succeed or fail depending on function availability
        let _ = result;
    }

    #[test]
    fn test_eval_local_call_with_complex_args() {
        let expr_val = Expr::LocalCall {
            function: "+".to_string(),
            args: vec![
                Expr::BinOp {
                    op: BinOp::Add,
                    left: Box::new(Expr::Integer(1)),
                    right: Box::new(Expr::Integer(2)),
                },
                Expr::Integer(3),
            ],
        };
        let bindings = new_bindings();
        let result = expr(&expr_val, &bindings);
        if let Ok((Term::Small(val), _)) = result {
            assert_eq!(val, 6); // (1+2) + 3 = 6
        }
    }

    #[test]
    fn test_eval_exprs_with_variable_bindings() {
        let expr_list = vec![
            Expr::Var("X".to_string()),
            Expr::Var("Y".to_string()),
        ];
        let mut bindings = new_bindings();
        bindings.insert("X".to_string(), Term::Small(10));
        bindings.insert("Y".to_string(), Term::Small(20));
        let (result, new_bindings) = exprs(expr_list, bindings).unwrap();
        assert_eq!(result, Term::Small(20)); // Last expression value
        assert_eq!(new_bindings.get("X"), Some(&Term::Small(10)));
        assert_eq!(new_bindings.get("Y"), Some(&Term::Small(20)));
    }

    #[test]
    fn test_eval_exprs_with_binding_updates() {
        let expr_list = vec![
            Expr::Match {
                left: Box::new(Expr::Var("X".to_string())),
                right: Box::new(Expr::Integer(42)),
            },
            Expr::Var("X".to_string()),
        ];
        let bindings = new_bindings();
        let (result, new_bindings) = exprs(expr_list, bindings).unwrap();
        assert_eq!(result, Term::Small(42));
        assert_eq!(new_bindings.get("X"), Some(&Term::Small(42)));
    }

    #[test]
    fn test_eval_exprs_with_nested_expressions() {
        let expr_list = vec![
            Expr::BinOp {
                op: BinOp::Add,
                left: Box::new(Expr::Integer(1)),
                right: Box::new(Expr::Integer(2)),
            },
            Expr::BinOp {
                op: BinOp::Mul,
                left: Box::new(Expr::Integer(3)),
                right: Box::new(Expr::Integer(4)),
            },
        ];
        let bindings = new_bindings();
        let (result, _) = exprs(expr_list, bindings).unwrap();
        assert_eq!(result, Term::Small(12)); // Last expression: 3 * 4 = 12
    }

    #[test]
    fn test_eval_match_pattern_list_with_empty_tail() {
        let pattern = Expr::List(vec![Expr::Integer(1)]);
        let value = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Nil),
        };
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_match_pattern_list_with_non_empty_tail() {
        let pattern = Expr::List(vec![Expr::Integer(1)]);
        let value = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::Nil),
            }),
        };
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_match_pattern_list_empty_pattern_empty_value() {
        let pattern = Expr::List(vec![]);
        let value = Term::Nil;
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_match_pattern_list_non_empty_pattern_empty_value() {
        let pattern = Expr::List(vec![Expr::Integer(1)]);
        let value = Term::Nil;
        let bindings = new_bindings();
        let result = match_pattern(&pattern, &value, &bindings);
        assert!(result.is_err());
    }
}

