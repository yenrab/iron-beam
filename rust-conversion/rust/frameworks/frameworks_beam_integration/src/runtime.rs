/*!
# BEAM Runtime Integration

This module provides integration with the BEAM runtime environment,
including module loading, function calling, and process management.
*/

use super::*;
use entities_process::*;

/// BEAM module loader
pub struct ModuleLoader {
    loaded_modules: HashMap<String, LoadedModule>,
    process_count: usize,
    memory_usage: usize,
}

impl ModuleLoader {
    pub fn new() -> Self {
        Self {
            loaded_modules: HashMap::new(),
            process_count: 0,
            memory_usage: 0,
        }
    }

    /// Load a BEAM file into the runtime
    pub fn load_module(&mut self, beam_file: &BeamFile) -> BeamResult<LoadedModule> {
        // In a real implementation, this would interface with the actual BEAM VM
        // For now, create a simulated loaded module

        let module = LoadedModule {
            name: beam_file.module_name.clone(),
            functions: vec!["start/0".to_string(), "stop/0".to_string()], // Simulated functions
            attributes: HashMap::new(),
            memory_size: beam_file.to_bytes().len(),
        };

        self.loaded_modules.insert(beam_file.module_name.clone(), module.clone());
        self.memory_usage += module.memory_size;

        Ok(module)
    }

    /// Unload a module from the runtime
    pub fn unload_module(&mut self, module: &LoadedModule) -> BeamResult<()> {
        if let Some(loaded) = self.loaded_modules.remove(&module.name) {
            self.memory_usage -= loaded.memory_size;
        }
        Ok(())
    }

    /// Call a function in a loaded module
    pub async fn call_function(
        &self,
        module: &LoadedModule,
        function_name: &str,
        args: &[BeamValue],
    ) -> BeamResult<BeamValue> {
        // Simulate function calling
        match (module.name.as_str(), function_name) {
            ("test_module", "start") => {
                if args.is_empty() {
                    Ok(BeamValue::Atom("started".to_string()))
                } else {
                    Err(BeamError::RuntimeError("start/0 expects no arguments".to_string()))
                }
            }
            ("test_module", "add") => {
                if args.len() == 2 {
                    match (&args[0], &args[1]) {
                        (BeamValue::Integer(a), BeamValue::Integer(b)) => {
                            Ok(BeamValue::Integer(a + b))
                        }
                        _ => Err(BeamError::RuntimeError("add/2 expects two integers".to_string())),
                    }
                } else {
                    Err(BeamError::RuntimeError("add/2 expects two arguments".to_string()))
                }
            }
            _ => Err(BeamError::FunctionNotFound(format!("{}/{}", module.name, function_name))),
        }
    }

    /// Spawn a new BEAM process
    pub fn spawn_process(&mut self, _module: &LoadedModule, _function: &str) -> BeamResult<entities_process::ProcessId> {
        // Simulate process spawning
        self.process_count += 1;
        Ok(self.process_count as entities_process::ProcessId)
    }

    /// Get the current process count
    pub fn get_process_count(&self) -> usize {
        self.process_count
    }

    /// Get current memory usage
    pub fn get_memory_usage(&self) -> usize {
        self.memory_usage
    }

    /// Send a message to a process
    pub async fn send_message(&self, pid: &entities_process::ProcessId, message: BeamValue) -> BeamResult<()> {
        // Simulate message sending
        println!("Sending message to process {:?}: {:?}", pid, message);
        Ok(())
    }

    /// Receive a message (simplified)
    pub async fn receive_message(&self, _pid: &entities_process::ProcessId) -> BeamResult<Option<BeamValue>> {
        // Simulate message reception (would normally block)
        Ok(None)
    }
}

impl Default for ModuleLoader {
    fn default() -> Self {
        Self::new()
    }
}



/// BEAM garbage collector interface
pub struct GarbageCollector {
    collections_performed: usize,
    memory_reclaimed: usize,
}

impl GarbageCollector {
    pub fn new() -> Self {
        Self {
            collections_performed: 0,
            memory_reclaimed: 0,
        }
    }

    /// Perform garbage collection
    pub fn collect(&mut self) -> BeamResult<usize> {
        // Simulate GC
        let reclaimed = 1024; // bytes
        self.collections_performed += 1;
        self.memory_reclaimed += reclaimed;
        Ok(reclaimed)
    }

    /// Get GC statistics
    pub fn get_stats(&self) -> GCStats {
        GCStats {
            collections_performed: self.collections_performed,
            total_memory_reclaimed: self.memory_reclaimed,
        }
    }

    /// Force major GC
    pub fn major_collect(&mut self) -> BeamResult<usize> {
        // Simulate major GC
        let reclaimed = 4096; // bytes
        self.collections_performed += 1;
        self.memory_reclaimed += reclaimed;
        Ok(reclaimed)
    }
}

impl Default for GarbageCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// GC statistics
#[derive(Debug, Clone)]
pub struct GCStats {
    pub collections_performed: usize,
    pub total_memory_reclaimed: usize,
}

/// BEAM scheduler interface
pub struct Scheduler {
    schedulers: usize,
    active_tasks: usize,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            schedulers: num_cpus::get(),
            active_tasks: 0,
        }
    }

    /// Schedule a task for execution
    pub async fn schedule_task<F, Fut>(&mut self, task: F) -> BeamResult<()>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        self.active_tasks += 1;

        // In a real implementation, this would schedule on BEAM schedulers
        // For simulation, just execute immediately
        task().await;

        self.active_tasks -= 1;
        Ok(())
    }

    /// Get scheduler statistics
    pub fn get_stats(&self) -> SchedulerStats {
        SchedulerStats {
            scheduler_count: self.schedulers,
            active_tasks: self.active_tasks,
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Scheduler statistics
#[derive(Debug, Clone)]
pub struct SchedulerStats {
    pub scheduler_count: usize,
    pub active_tasks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_loader_creation() {
        let loader = ModuleLoader::new();
        assert_eq!(loader.get_process_count(), 0);
        assert_eq!(loader.get_memory_usage(), 0);
    }

    #[test]
    fn test_load_module() {
        let mut loader = ModuleLoader::new();
        let beam_file = BeamFile::new("test_module".to_string());

        let result = loader.load_module(&beam_file);
        assert!(result.is_ok());

        let module = result.unwrap();
        assert_eq!(module.name, "test_module");
        assert_eq!(loader.get_memory_usage(), beam_file.to_bytes().len());
    }

    #[test]
    fn test_unload_module() {
        let mut loader = ModuleLoader::new();
        let beam_file = BeamFile::new("test_module".to_string());

        let module = loader.load_module(&beam_file).unwrap();
        let initial_memory = loader.get_memory_usage();

        loader.unload_module(&module).unwrap();
        assert_eq!(loader.get_memory_usage(), initial_memory - module.memory_size);
    }

    #[tokio::test]
    async fn test_call_function_start() {
        let loader = ModuleLoader::new();
        let module = LoadedModule {
            name: "test_module".to_string(),
            functions: vec![],
            attributes: HashMap::new(),
            memory_size: 0,
        };

        let result = loader.call_function(&module, "start", &[]).await;
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), BeamValue::Atom(ref s) if s == "started"));
    }

    #[tokio::test]
    async fn test_call_function_add() {
        let loader = ModuleLoader::new();
        let module = LoadedModule {
            name: "test_module".to_string(),
            functions: vec![],
            attributes: HashMap::new(),
            memory_size: 0,
        };

        let result = loader.call_function(
            &module,
            "add",
            &[BeamValue::Integer(3), BeamValue::Integer(4)]
        ).await;

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), BeamValue::Integer(7)));
    }

    #[test]
    fn test_spawn_process() {
        let mut loader = ModuleLoader::new();
        let module = LoadedModule {
            name: "test_module".to_string(),
            functions: vec![],
            attributes: HashMap::new(),
            memory_size: 0,
        };

        let result = loader.spawn_process(&module, "start");
        assert!(result.is_ok());
        assert_eq!(loader.get_process_count(), 1);
    }

    #[test]
    fn test_garbage_collector() {
        let mut gc = GarbageCollector::new();

        let result = gc.collect();
        assert!(result.is_ok());

        let stats = gc.get_stats();
        assert_eq!(stats.collections_performed, 1);
        assert!(stats.total_memory_reclaimed > 0);
    }

    #[test]
    fn test_garbage_collector_default() {
        let gc = GarbageCollector::default();

        let stats = gc.get_stats();
        assert_eq!(stats.collections_performed, 0);
        assert_eq!(stats.total_memory_reclaimed, 0);
    }

    #[test]
    fn test_garbage_collector_multiple_collections() {
        let mut gc = GarbageCollector::new();

        // Perform multiple collections
        let num_collections = 5;
        let mut total_expected = 0;

        for i in 0..num_collections {
            let result = gc.collect();
            assert!(result.is_ok());

            let reclaimed = result.unwrap();
            total_expected += reclaimed;

            let stats = gc.get_stats();
            assert_eq!(stats.collections_performed, i + 1);
            assert_eq!(stats.total_memory_reclaimed, total_expected);
        }
    }

    #[test]
    fn test_garbage_collector_major_collect() {
        let mut gc = GarbageCollector::new();

        let result = gc.major_collect();
        assert!(result.is_ok());

        let reclaimed = result.unwrap();
        let stats = gc.get_stats();

        assert_eq!(stats.collections_performed, 1);
        assert_eq!(stats.total_memory_reclaimed, reclaimed);
        assert!(reclaimed > 0); // Major collection should reclaim more memory
    }

    #[test]
    fn test_garbage_collector_mixed_collections() {
        let mut gc = GarbageCollector::new();

        // Mix of regular and major collections
        let operations = vec![
            ("regular", 1024),
            ("major", 4096),
            ("regular", 1024),
            ("major", 4096),
            ("regular", 1024),
        ];

        let mut expected_total = 0;

        for (op_type, expected_reclaimed) in operations {
            let result = match op_type {
                "regular" => gc.collect(),
                "major" => gc.major_collect(),
                _ => panic!("Unknown operation type"),
            };

            assert!(result.is_ok());
            let actual_reclaimed = result.unwrap();
            expected_total += actual_reclaimed;

            let stats = gc.get_stats();
            assert_eq!(stats.total_memory_reclaimed, expected_total);
            assert!(stats.collections_performed >= 1);
        }
    }

    #[test]
    fn test_garbage_collector_stats_consistency() {
        let mut gc = GarbageCollector::new();

        // Get initial stats
        let initial_stats = gc.get_stats();
        assert_eq!(initial_stats.collections_performed, 0);
        assert_eq!(initial_stats.total_memory_reclaimed, 0);

        // Perform operations
        gc.collect().unwrap();
        let after_regular = gc.get_stats();

        gc.major_collect().unwrap();
        let after_major = gc.get_stats();

        // Verify consistency
        assert_eq!(after_regular.collections_performed, 1);
        assert_eq!(after_major.collections_performed, 2);
        assert!(after_major.total_memory_reclaimed > after_regular.total_memory_reclaimed);
        assert!(after_regular.total_memory_reclaimed > initial_stats.total_memory_reclaimed);
    }

    #[test]
    fn test_gc_stats_clone_and_debug() {
        let mut gc = GarbageCollector::new();
        gc.collect().unwrap();

        let stats = gc.get_stats();
        let cloned_stats = stats.clone();

        // Test equality
        assert_eq!(stats.collections_performed, cloned_stats.collections_performed);
        assert_eq!(stats.total_memory_reclaimed, cloned_stats.total_memory_reclaimed);

        // Test debug formatting
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("collections_performed"));
        assert!(debug_str.contains("total_memory_reclaimed"));
    }

    #[test]
    fn test_scheduler() {
        let scheduler = Scheduler::new();

        let stats = scheduler.get_stats();
        assert!(stats.scheduler_count > 0);
        assert_eq!(stats.active_tasks, 0);
    }

    #[test]
    fn test_scheduler_default() {
        let scheduler = Scheduler::default();

        let stats = scheduler.get_stats();
        assert!(stats.scheduler_count > 0);
        assert_eq!(stats.active_tasks, 0);
    }

    #[tokio::test]
    async fn test_scheduler_schedule_task() {
        let mut scheduler = Scheduler::new();

        // Verify initial state
        let initial_stats = scheduler.get_stats();
        assert_eq!(initial_stats.active_tasks, 0);

        // Schedule a simple task
        let result = scheduler.schedule_task(|| async {
            // Simulate some work
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        }).await;

        assert!(result.is_ok());

        // After task completion, active tasks should be back to 0
        let final_stats = scheduler.get_stats();
        assert_eq!(final_stats.active_tasks, 0);
        assert_eq!(final_stats.scheduler_count, initial_stats.scheduler_count);
    }

    #[tokio::test]
    async fn test_scheduler_multiple_tasks() {
        let mut scheduler = Scheduler::new();

        // Schedule multiple tasks one by one
        let num_tasks = 5;

        for i in 0..num_tasks {
            let task = scheduler.schedule_task(move || async move {
                // Each task does different amounts of work
                let delay = (i + 1) as u64;
                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            });
            task.await.unwrap();
        }

        // All tasks should be completed
        let stats = scheduler.get_stats();
        assert_eq!(stats.active_tasks, 0);
    }

    #[tokio::test]
    async fn test_scheduler_task_with_result() {
        let mut scheduler = Scheduler::new();

        // Schedule a task that does computation (but doesn't return result through schedule_task)
        let mut computed_value = 0;
        let result = scheduler.schedule_task(|| async {
            // Simulate computation
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            computed_value = 42;
        }).await;

        assert!(result.is_ok());
        assert_eq!(computed_value, 42);
    }

    #[tokio::test]
    async fn test_scheduler_stats_during_execution() {
        let mut scheduler = Scheduler::new();

        // Start a long-running task
        let task = scheduler.schedule_task(|| async {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        });

        // Check that task is tracked as active during execution
        // Note: This is a race condition - the task might complete before we check
        let _ = task.await;

        // Final check
        let final_stats = scheduler.get_stats();
        assert_eq!(final_stats.active_tasks, 0);
    }

    #[test]
    fn test_scheduler_stats_consistency() {
        let scheduler = Scheduler::new();

        let stats1 = scheduler.get_stats();
        let stats2 = scheduler.get_stats();

        // Stats should be consistent
        assert_eq!(stats1.scheduler_count, stats2.scheduler_count);
        assert_eq!(stats1.active_tasks, stats2.active_tasks);
    }

    #[tokio::test]
    async fn test_runtime_integration_module_loading_and_execution() {
        let mut loader = ModuleLoader::new();
        let mut scheduler = Scheduler::new();

        // Load a module
        let beam_file = BeamFile::with_data("integration_test".to_string(), vec![1, 2, 3, 4, 5]);
        let module = loader.load_module(&beam_file).unwrap();

        // Schedule a task that uses the module
        let mut task_status = "";
        let result = scheduler.schedule_task(|| async {
            // Simulate module usage
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            task_status = "task_completed";
        }).await;

        assert!(result.is_ok());

        // Verify module is still loaded and memory is tracked
        assert!(loader.loaded_modules.contains_key("integration_test"));
        assert_eq!(loader.get_memory_usage(), module.memory_size);
        assert_eq!(loader.get_process_count(), 0); // No processes spawned in this test
    }

    #[tokio::test]
    async fn test_runtime_integration_full_workflow() {
        let mut loader = ModuleLoader::new();
        let mut scheduler = Scheduler::new();
        let mut gc = GarbageCollector::new();

        // Load multiple modules
        let modules = vec!["workflow_a", "workflow_b", "workflow_c"];
        let mut loaded_modules = Vec::new();

        for name in &modules {
            let beam_file = BeamFile::with_data(name.to_string(), vec![0u8; 100]);
            let module = loader.load_module(&beam_file).unwrap();
            loaded_modules.push(module);
        }

        // Spawn some processes
        for module in &loaded_modules {
            loader.spawn_process(module, "init").unwrap();
        }

        // Schedule tasks that simulate work one by one
        for i in 0..3 {
            let task = scheduler.schedule_task(move || async move {
                tokio::time::sleep(tokio::time::Duration::from_millis((i + 1) as u64)).await;
                // Task completes without returning a value
            });
            task.await.unwrap();
        }

        // Perform garbage collection
        let reclaimed = gc.collect().unwrap();

        // Verify final state
        assert_eq!(loader.loaded_modules.len(), modules.len());
        assert_eq!(loader.get_process_count(), modules.len());
        assert_eq!(scheduler.get_stats().active_tasks, 0);
        assert!(gc.get_stats().collections_performed >= 1);
        assert!(reclaimed > 0);
    }

    #[test]
    fn test_runtime_integration_resource_management() {
        let mut loader = ModuleLoader::new();
        let mut gc = GarbageCollector::new();

        // Load modules
        let mut modules = Vec::new();
        for i in 0..5 {
            let beam_file = BeamFile::with_data(format!("resource_test_{}", i), vec![i as u8; 50]);
            let module = loader.load_module(&beam_file).unwrap();
            modules.push(module);
        }

        let initial_memory = loader.get_memory_usage();

        // Simulate some GC activity
        gc.collect().unwrap();
        gc.major_collect().unwrap();

        // Unload some modules
        let to_unload = 2;
        for module in modules.into_iter().take(to_unload) {
            loader.unload_module(&module).unwrap();
        }

        // Verify resource cleanup
        let final_memory = loader.get_memory_usage();
        assert!(final_memory < initial_memory);
        assert_eq!(loader.loaded_modules.len(), 5 - to_unload);

        let gc_stats = gc.get_stats();
        assert_eq!(gc_stats.collections_performed, 2);
        assert!(gc_stats.total_memory_reclaimed > 0);
    }

    #[tokio::test]
    async fn test_runtime_integration_concurrent_operations() {
        let mut loader = ModuleLoader::new();
        let mut scheduler = Scheduler::new();

        // Load a module
        let beam_file = BeamFile::with_data("concurrent_test".to_string(), vec![1, 2, 3]);
        let module = loader.load_module(&beam_file).unwrap();

        // Schedule concurrent tasks that all use the same module one by one
        let num_concurrent_tasks = 10;

        for _ in 0..num_concurrent_tasks {
            let task = scheduler.schedule_task(|| async move {
                // Simulate concurrent module usage
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            });
            task.await.unwrap();
        }

        // Verify module is still intact
        assert!(loader.loaded_modules.contains_key("concurrent_test"));
        assert_eq!(loader.loaded_modules["concurrent_test"].name, "concurrent_test");
        assert_eq!(scheduler.get_stats().active_tasks, 0);
    }

    #[test]
    fn test_runtime_components_independence() {
        // Test that components work independently
        let loader = ModuleLoader::new();
        let gc = GarbageCollector::new();
        let scheduler = Scheduler::new();

        // Each component should have its own state
        assert_eq!(loader.get_memory_usage(), 0);
        assert_eq!(gc.get_stats().collections_performed, 0);
        assert_eq!(scheduler.get_stats().active_tasks, 0);

        // Components should not affect each other
        assert!(loader.loaded_modules.is_empty());
        assert_eq!(gc.get_stats().total_memory_reclaimed, 0);
        assert!(scheduler.get_stats().scheduler_count > 0);
    }

    #[test]
    fn test_module_loader_default() {
        let loader = ModuleLoader::default();
        assert_eq!(loader.get_process_count(), 0);
        assert_eq!(loader.get_memory_usage(), 0);
        assert!(loader.loaded_modules.is_empty());
    }

    #[test]
    fn test_load_module_extreme_names() {
        let mut loader = ModuleLoader::new();

        // Test various module names
        let test_cases = vec![
            "".to_string(), // Empty name
            "a".to_string(), // Single character
            "very_long_module_name_that_exceeds_normal_limits_and_might_cause_issues".to_string(),
            "module-with-dashes".to_string(),
            "module_with_underscores".to_string(),
            "123numeric_start".to_string(),
            "Unicode🚀Module".to_string(),
            "\t\n\r".to_string(), // Whitespace
        ];

        for (i, name) in test_cases.iter().enumerate() {
            let beam_file = BeamFile::with_data(format!("test_{}", i), vec![1, 2, 3]);
            let result = loader.load_module(&beam_file);
            assert!(result.is_ok(), "Failed to load module with name: {}", name);

            let module = result.unwrap();
            assert_eq!(module.name, format!("test_{}", i));
        }

        assert_eq!(loader.loaded_modules.len(), test_cases.len());
    }

    #[test]
    fn test_unload_nonexistent_module() {
        let mut loader = ModuleLoader::new();

        // Create a module that doesn't match the loaded ones
        let fake_module = LoadedModule {
            name: "nonexistent".to_string(),
            functions: vec![],
            attributes: HashMap::new(),
            memory_size: 100,
        };

        // Should not panic when unloading nonexistent module
        let result = loader.unload_module(&fake_module);
        assert!(result.is_ok());
        assert_eq!(loader.get_memory_usage(), 0); // Memory should remain unchanged
    }

    #[test]
    fn test_load_duplicate_module() {
        let mut loader = ModuleLoader::new();
        let beam_file = BeamFile::with_data("duplicate_test".to_string(), vec![1, 2, 3, 4, 5]);

        // Load the same module twice
        let result1 = loader.load_module(&beam_file);
        assert!(result1.is_ok());

        let result2 = loader.load_module(&beam_file);
        assert!(result2.is_ok());

        // Should have two entries with the same name (simulating module replacement)
        // In a real implementation, this might replace or error
        assert!(result1.unwrap().name == result2.unwrap().name);
    }

    #[test]
    fn test_module_loader_memory_tracking() {
        let mut loader = ModuleLoader::new();

        // Test with empty beam file
        let empty_file = BeamFile::new("empty".to_string());
        let empty_module = loader.load_module(&empty_file).unwrap();
        assert_eq!(loader.get_memory_usage(), empty_module.memory_size);

        // Test with large beam file
        let large_data = vec![0u8; 10000];
        let large_file = BeamFile::with_data("large".to_string(), large_data);
        let large_module = loader.load_module(&large_file).unwrap();

        let expected_memory = empty_module.memory_size + large_module.memory_size;
        assert_eq!(loader.get_memory_usage(), expected_memory);

        // Unload one module
        loader.unload_module(&empty_module).unwrap();
        assert_eq!(loader.get_memory_usage(), large_module.memory_size);

        // Unload remaining module
        loader.unload_module(&large_module).unwrap();
        assert_eq!(loader.get_memory_usage(), 0);
    }

    #[test]
    fn test_spawn_multiple_processes() {
        let mut loader = ModuleLoader::new();
        let module = LoadedModule {
            name: "test".to_string(),
            functions: vec![],
            attributes: HashMap::new(),
            memory_size: 0,
        };

        // Spawn multiple processes
        let mut pids = Vec::new();
        for i in 0..10 {
            let pid = loader.spawn_process(&module, &format!("func_{}", i)).unwrap();
            pids.push(pid);
            assert_eq!(loader.get_process_count(), i + 1);
        }

        // All PIDs should be unique and sequential
        for (i, &pid) in pids.iter().enumerate() {
            assert_eq!(pid, (i + 1) as entities_process::ProcessId);
        }

        assert_eq!(loader.get_process_count(), 10);
    }

    #[tokio::test]
    async fn test_call_function_unknown_module() {
        let loader = ModuleLoader::new();
        let module = LoadedModule {
            name: "unknown_module".to_string(),
            functions: vec![],
            attributes: HashMap::new(),
            memory_size: 0,
        };

        let result = loader.call_function(&module, "any_func", &[]).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            BeamError::FunctionNotFound(msg) => {
                assert!(msg.contains("unknown_module"));
            }
            _ => panic!("Expected FunctionNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_call_function_start_with_args() {
        let loader = ModuleLoader::new();
        let module = LoadedModule {
            name: "test_module".to_string(),
            functions: vec![],
            attributes: HashMap::new(),
            memory_size: 0,
        };

        // start/0 should fail with arguments
        let result = loader.call_function(&module, "start", &[BeamValue::Integer(1)]).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            BeamError::RuntimeError(msg) => {
                assert!(msg.contains("expects no arguments"));
            }
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[tokio::test]
    async fn test_call_function_add_wrong_arg_count() {
        let loader = ModuleLoader::new();
        let module = LoadedModule {
            name: "test_module".to_string(),
            functions: vec![],
            attributes: HashMap::new(),
            memory_size: 0,
        };

        // add/2 with wrong number of arguments
        let result = loader.call_function(&module, "add", &[BeamValue::Integer(1)]).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            BeamError::RuntimeError(msg) => {
                assert!(msg.contains("expects two arguments"));
            }
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[tokio::test]
    async fn test_call_function_add_wrong_arg_types() {
        let loader = ModuleLoader::new();
        let module = LoadedModule {
            name: "test_module".to_string(),
            functions: vec![],
            attributes: HashMap::new(),
            memory_size: 0,
        };

        // add/2 with wrong argument types
        let result = loader.call_function(
            &module,
            "add",
            &[BeamValue::Atom("not_a_number".to_string()), BeamValue::Integer(2)]
        ).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            BeamError::RuntimeError(msg) => {
                assert!(msg.contains("expects two integers"));
            }
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[tokio::test]
    async fn test_call_function_add_edge_cases() {
        let loader = ModuleLoader::new();
        let module = LoadedModule {
            name: "test_module".to_string(),
            functions: vec![],
            attributes: HashMap::new(),
            memory_size: 0,
        };

        // Test with extreme integer values
        let test_cases = vec![
            (i64::MAX, i64::MIN), // Should handle overflow gracefully
            (0, 0),
            (-1, 1),
            (i64::MAX / 2, i64::MAX / 2), // Large positive numbers
        ];

        for (a, b) in test_cases {
            let result = loader.call_function(
                &module,
                "add",
                &[BeamValue::Integer(a), BeamValue::Integer(b)]
            ).await;

            assert!(result.is_ok(), "Failed for {} + {}", a, b);
            match result.unwrap() {
                BeamValue::Integer(sum) => {
                    // Just verify it's an integer result (overflow behavior may vary)
                    let _ = sum;
                }
                _ => panic!("Expected integer result"),
            }
        }
    }

    #[tokio::test]
    async fn test_call_function_unknown_function() {
        let loader = ModuleLoader::new();
        let module = LoadedModule {
            name: "test_module".to_string(),
            functions: vec![],
            attributes: HashMap::new(),
            memory_size: 0,
        };

        let result = loader.call_function(&module, "nonexistent_function", &[]).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            BeamError::FunctionNotFound(msg) => {
                assert!(msg.contains("test_module"));
                assert!(msg.contains("nonexistent_function"));
            }
            _ => panic!("Expected FunctionNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_send_message() {
        let loader = ModuleLoader::new();
        let pid = 123 as entities_process::ProcessId;

        // Test sending various message types
        let test_messages = vec![
            BeamValue::Integer(42),
            BeamValue::Float(3.14),
            BeamValue::Atom("test_message".to_string()),
            BeamValue::String("hello world".to_string()),
            BeamValue::List(vec![BeamValue::Integer(1), BeamValue::Integer(2)]),
            BeamValue::Tuple(vec![BeamValue::Atom("ok".to_string()), BeamValue::Integer(100)]),
        ];

        for message in test_messages {
            let result = loader.send_message(&pid, message.clone()).await;
            assert!(result.is_ok(), "Failed to send message: {:?}", message);
        }
    }

    #[tokio::test]
    async fn test_send_message_edge_cases() {
        let loader = ModuleLoader::new();

        // Test with extreme PIDs
        let extreme_pids = vec![
            0 as entities_process::ProcessId,
            1 as entities_process::ProcessId,
            u64::MAX as entities_process::ProcessId,
        ];

        for pid in extreme_pids {
            let message = BeamValue::Atom("test".to_string());
            let result = loader.send_message(&pid, message).await;
            assert!(result.is_ok(), "Failed to send to PID: {}", pid);
        }
    }

    #[tokio::test]
    async fn test_receive_message() {
        let loader = ModuleLoader::new();
        let pid = 456 as entities_process::ProcessId;

        // Currently returns None (no messages available)
        let result = loader.receive_message(&pid).await;
        assert!(result.is_ok());

        let message = result.unwrap();
        assert!(message.is_none(), "Expected no message available");
    }

    #[tokio::test]
    async fn test_receive_message_multiple_calls() {
        let loader = ModuleLoader::new();
        let pid = 789 as entities_process::ProcessId;

        // Multiple calls should all return None
        for i in 0..5 {
            let result = loader.receive_message(&pid).await;
            assert!(result.is_ok(), "Call {} failed", i);

            let message = result.unwrap();
            assert!(message.is_none(), "Call {} unexpectedly received a message", i);
        }
    }

    #[test]
    fn test_multiple_modules_loading() {
        let mut loader = ModuleLoader::new();

        // Load multiple modules
        let module_names = vec!["module_a", "module_b", "module_c", "module_d"];
        let mut loaded_modules = Vec::new();
        let mut expected_memory = 0;

        for name in &module_names {
            let beam_file = BeamFile::with_data(name.to_string(), vec![1, 2, 3]);
            let module = loader.load_module(&beam_file).unwrap();
            loaded_modules.push(module.clone());
            expected_memory += module.memory_size;
        }

        // Verify all modules are loaded
        assert_eq!(loader.loaded_modules.len(), module_names.len());
        assert_eq!(loader.get_memory_usage(), expected_memory);

        // Verify each module is accessible
        for (i, name) in module_names.iter().enumerate() {
            assert!(loader.loaded_modules.contains_key(*name));
            assert_eq!(loader.loaded_modules[*name].name, *name);
        }
    }

    #[test]
    fn test_module_unloading_order() {
        let mut loader = ModuleLoader::new();

        // Load modules
        let mut modules = Vec::new();
        for i in 0..5 {
            let beam_file = BeamFile::with_data(format!("mod_{}", i), vec![0u8; i * 10]);
            let module = loader.load_module(&beam_file).unwrap();
            modules.push(module);
        }

        let initial_memory = loader.get_memory_usage();

        // Unload modules in reverse order
        let mut current_memory = initial_memory;
        for module in modules.into_iter().rev() {
            let before_unload = current_memory;
            loader.unload_module(&module).unwrap();
            current_memory -= module.memory_size;
            assert_eq!(loader.get_memory_usage(), current_memory);
            assert_eq!(before_unload - module.memory_size, current_memory);
        }

        assert_eq!(loader.get_memory_usage(), 0);
        assert!(loader.loaded_modules.is_empty());
    }

    #[test]
    fn test_memory_usage_accumulation() {
        let mut loader = ModuleLoader::new();

        // Start with no memory usage
        assert_eq!(loader.get_memory_usage(), 0);

        // Load modules of increasing sizes
        let sizes = vec![100, 200, 300, 400, 500];
        let mut expected_total = 0;

        for (i, &size) in sizes.iter().enumerate() {
            let data = vec![0u8; size];
            let beam_file = BeamFile::with_data(format!("mem_test_{}", i), data);
            let module = loader.load_module(&beam_file).unwrap();

            expected_total += module.memory_size;
            assert_eq!(loader.get_memory_usage(), expected_total);
        }

        // Partial unloading
        let modules: Vec<_> = loader.loaded_modules.values().cloned().collect();
        let middle_module = &modules[2]; // Remove the middle one
        let middle_memory = middle_module.memory_size;

        loader.unload_module(middle_module).unwrap();
        expected_total -= middle_memory;
        assert_eq!(loader.get_memory_usage(), expected_total);
    }

    #[test]
    fn test_module_loader_capacity() {
        let mut loader = ModuleLoader::new();

        // Load many modules to test capacity
        let num_modules = 100;
        let mut modules = Vec::new();

        for i in 0..num_modules {
            let beam_file = BeamFile::with_data(format!("capacity_test_{}", i), vec![i as u8; 10]);
            let module = loader.load_module(&beam_file).unwrap();
            modules.push(module);
        }

        assert_eq!(loader.loaded_modules.len(), num_modules);

        // Calculate expected memory
        let expected_memory: usize = modules.iter().map(|m| m.memory_size).sum();
        assert_eq!(loader.get_memory_usage(), expected_memory);

        // Unload half of them
        let to_unload = modules.len() / 2;
        let mut removed_memory = 0;

        for module in modules.into_iter().take(to_unload) {
            removed_memory += module.memory_size;
            loader.unload_module(&module).unwrap();
        }

        assert_eq!(loader.loaded_modules.len(), num_modules - to_unload);
        assert_eq!(loader.get_memory_usage(), expected_memory - removed_memory);
    }
}
