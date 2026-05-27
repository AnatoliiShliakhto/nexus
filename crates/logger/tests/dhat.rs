#![allow(unused_crate_dependencies, clippy::print_stdout)]

// #[global_allocator]
// static ALLOC: dhat::Alloc = dhat::Alloc;
//
// #[test]
// fn test_memory_usage() {
//     let profiler = dhat::Profiler::new_heap();
//
//     let _logger = nx_logger::Logger::builder()
//         .name("dhat-test")
//         .json(true)
//         .console(false)
//         .init()
//         .expect("Logger initialization failed");
//
//     for i in 0..1000 {
//         tracing::info!(val = i, "small test message");
//     }
//
//     for i in 0..1000 {
//         let dynamic_string = format!("this is dynamic message number {i} and it is very long");
//         tracing::info!(val = i, dynamic_string);
//     }
//
//     drop(profiler);
//
//     if let (Ok(root_dir), Ok(manifest_dir)) =
//         (std::env::var("CARGO_WORKSPACE_DIR"), std::env::var("CARGO_MANIFEST_DIR"))
//     {
//         let root = std::path::PathBuf::from(root_dir);
//         let manifest = std::path::PathBuf::from(manifest_dir);
//         let default_path = manifest.join("dhat-heap.json");
//
//         let target_dir = root.join("target").join("dhat");
//         std::fs::create_dir_all(&target_dir).ok();
//
//         let new_path = target_dir.join("logger-heap.json");
//
//         if default_path.exists() && std::fs::rename(&default_path, &new_path).is_ok() {
//             println!("\n[DHAT] Memory report generated at: {}", new_path.display());
//             println!(
//                 "[DHAT] You can visualize it here: https://nnethercote.github.io/dh_view/dh_view.html\n"
//             );
//         }
//     }
// }
