use std::sync::Arc;
use std::sync::atomic::AtomicBool;

fn main() {
    println!("Testing JIT execution fix...");
    
    // Test that the emulator loop can be called
    let mut emulator_loop = infrastructure_emulator_loop::EmulatorLoop::new();
    let process = entities_process::Process::new(1);
    emulator_loop.set_current_process(Some(process.into()));
    
    let init_done = Arc::new(AtomicBool::new(true));
    
    // This should not crash with the new BEAM-compatible signature
    match infrastructure_emulator_loop::process_main(&mut emulator_loop, init_done) {
        Ok(result) => {
            println!("JIT execution completed successfully: {:?}", result);
        }
        Err(e) => {
            println!("JIT execution failed: {:?}", e);
        }
    }
}
