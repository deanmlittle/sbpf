mod utils;

use utils::EnvVarGuard;

use std::process::Command;
use std::fs;

#[test]
fn test_memo_project_e2e() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let sbpf_path = current_dir.join("target").join("debug").join("sbpf");
    let _guard = EnvVarGuard::new("SBPF_BIN", sbpf_path.to_string_lossy());
    let sbpf_bin = std::env::var("SBPF_BIN").expect("SBPF_BIN not set");
    let temp_dir = std::env::temp_dir().join("sbpf_test_memo_e2e");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).expect("Failed to remove existing test directory");
    }
    fs::create_dir_all(&temp_dir).expect("Failed to create test directory");
    println!("Using test directory: {:?}", temp_dir);
    // Step 1: Initialize the memo project using the built binary
    println!("Step 1: Initializing memo project...");
    let init_output = Command::new(&sbpf_bin)
        .current_dir(&temp_dir)
        .arg("init")
        .arg("memo")
        .output()
        .expect("Failed to execute target/debug/sbpf init memo");
    if !init_output.status.success() {
        let stderr = String::from_utf8_lossy(&init_output.stderr);
        let stdout = String::from_utf8_lossy(&init_output.stdout);
        panic!("target/debug/sbpf init memo failed:\nSTDOUT: {}\nSTDERR: {}", stdout, stderr);
    }
    println!("✅ Project initialized successfully");
    println!("STDOUT: {}", String::from_utf8_lossy(&init_output.stdout));
    // Step 2: Set up memo directory path
    let memo_dir = temp_dir.join("memo");
    println!("Using memo directory: {:?}", memo_dir);
    assert!(memo_dir.join("src").exists(), "src directory should exist");
    assert!(memo_dir.join("deploy").exists(), "deploy directory should exist");
    assert!(memo_dir.join("src/memo").exists(), "src/memo directory should exist");
    assert!(memo_dir.join("src/memo/memo.s").exists(), "src/memo/memo.s should exist");
    assert!(memo_dir.join("src/lib.rs").exists(), "src/lib.rs should exist");
    assert!(memo_dir.join("Cargo.toml").exists(), "Cargo.toml should exist");
    // Step 3: Run light-build using the built binary
    println!("Step 3: Running light-build...");
    let light_build_output = Command::new(&sbpf_bin)
        .current_dir(&memo_dir)
        .arg("light-build")
        .output()
        .expect("Failed to execute target/debug/sbpf light-build");
    if !light_build_output.status.success() {
        let stderr = String::from_utf8_lossy(&light_build_output.stderr);
        let stdout = String::from_utf8_lossy(&light_build_output.stdout);
        panic!("target/debug/sbpf light-build failed:\nSTDOUT: {}\nSTDERR: {}", stdout, stderr);
    }
    println!("✅ Light-build completed successfully");
    println!("STDOUT: {}", String::from_utf8_lossy(&light_build_output.stdout));
    // Verify that .so files were created in the deploy directory
    let deploy_dir = memo_dir.join("deploy");
    assert!(deploy_dir.exists(), "deploy directory should exist");
    let so_files: Vec<_> = fs::read_dir(&deploy_dir)
        .expect("Failed to read deploy directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "so")
                .unwrap_or(false)
        })
        .collect();
    assert!(!so_files.is_empty(), "At least one .so file should be created in deploy directory");
    println!("Found {} .so file(s) in deploy directory", so_files.len());
    // Step 4: Run tests using the built binary
    println!("Step 4: Running tests...");
    let test_output = Command::new(&sbpf_bin)
        .current_dir(&memo_dir)
        .arg("test")
        .output()
        .expect("Failed to execute target/debug/sbpf test");
    if !test_output.status.success() {
        let stderr = String::from_utf8_lossy(&test_output.stderr);
        let stdout = String::from_utf8_lossy(&test_output.stdout);
        panic!("target/debug/sbpf test failed:\nSTDOUT: {}\nSTDERR: {}", stdout, stderr);
    }
    println!("✅ Tests completed successfully");
    println!("STDOUT: {}", String::from_utf8_lossy(&test_output.stdout));
    // Step 5: Clean up
    fs::remove_dir_all(&temp_dir).expect("Failed to remove test directory");
    println!("🎉 All tests passed! Memo project E2E test completed successfully.");
}

#[test]
fn test_memo_project_e2e_second() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let sbpf_path = current_dir.join("target").join("debug").join("sbpf");
    let _guard = EnvVarGuard::new("SBPF_BIN", sbpf_path.to_string_lossy());
    let sbpf_bin = std::env::var("SBPF_BIN").expect("SBPF_BIN not set");
    let temp_dir = std::env::temp_dir().join("sbpf_test_memo_e2e_second");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).expect("Failed to remove existing test directory");
    }
    fs::create_dir_all(&temp_dir).expect("Failed to create test directory");
    println!("Using system temp directory for second test: {:?}", temp_dir);
    // Step 1: Initialize the memo project using the built binary
    println!("Step 1: Initializing second memo project...");
    let init_output = Command::new(&sbpf_bin)
        .current_dir(&temp_dir)
        .arg("init")
        .arg("memo2")
        .output()
        .expect("Failed to execute target/debug/sbpf init memo2");
    if !init_output.status.success() {
        let stderr = String::from_utf8_lossy(&init_output.stderr);
        let stdout = String::from_utf8_lossy(&init_output.stdout);
        panic!("target/debug/sbpf init memo2 failed:\nSTDOUT: {}\nSTDERR: {}", stdout, stderr);
    }
    println!("✅ Second project initialized successfully");
    println!("STDOUT: {}", String::from_utf8_lossy(&init_output.stdout));
    // Step 2: Set up memo2 directory path
    let memo_dir = temp_dir.join("memo2");
    println!("Using memo2 directory: {:?}", memo_dir);
    assert!(memo_dir.join("src").exists(), "src directory should exist");
    assert!(memo_dir.join("deploy").exists(), "deploy directory should exist");
    assert!(memo_dir.join("src/memo2").exists(), "src/memo2 directory should exist");
    assert!(memo_dir.join("src/memo2/memo2.s").exists(), "src/memo2/memo2.s should exist");
    assert!(memo_dir.join("src/lib.rs").exists(), "src/lib.rs should exist");
    assert!(memo_dir.join("Cargo.toml").exists(), "Cargo.toml should exist");
    // Update the memo2.s content to the specified content
    let new_memo_content = r#".equ NUM_ACCOUNTS, 0x00
.equ DATA_LEN, 0x08
.equ DATA, 0x10

.globl entrypoint
entrypoint:
  ldxdw r0, [r1+NUM_ACCOUNTS]
  ldxdw r2, [r1+DATA_LEN]
  add64 r1, DATA
  call sol_log_
  exit"#;
    fs::write(memo_dir.join("src/memo2/memo2.s"), new_memo_content).expect("Failed to write new memo2.s content");
    println!("✅ Updated memo2.s with specified content");
    // Step 3: Run light-build using the built binary
    println!("Step 3: Running light-build on second project...");
    let light_build_output = Command::new(&sbpf_bin)
        .current_dir(&memo_dir)
        .arg("light-build")
        .output()
        .expect("Failed to execute target/debug/sbpf light-build");
    if !light_build_output.status.success() {
        let stderr = String::from_utf8_lossy(&light_build_output.stderr);
        let stdout = String::from_utf8_lossy(&light_build_output.stdout);
        panic!("target/debug/sbpf light-build failed:\nSTDOUT: {}\nSTDERR: {}", stdout, stderr);
    }
    println!("✅ Second light-build completed successfully");
    println!("STDOUT: {}", String::from_utf8_lossy(&light_build_output.stdout));
    // Verify that .so files were created in the deploy directory
    let deploy_dir = memo_dir.join("deploy");
    assert!(deploy_dir.exists(), "deploy directory should exist");
    let so_files: Vec<_> = fs::read_dir(&deploy_dir)
        .expect("Failed to read deploy directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "so")
                .unwrap_or(false)
        })
        .collect();
    assert!(!so_files.is_empty(), "At least one .so file should be created in deploy directory");
    println!("Found {} .so file(s) in deploy directory", so_files.len());
    // Step 4: Run tests using the built binary
    println!("Step 4: Running tests on second project...");
    let test_output = Command::new(&sbpf_bin)
        .current_dir(&memo_dir)
        .arg("test")
        .output()
        .expect("Failed to execute target/debug/sbpf test");
    if !test_output.status.success() {
        let stderr = String::from_utf8_lossy(&test_output.stderr);
        let stdout = String::from_utf8_lossy(&test_output.stdout);
        panic!("target/debug/sbpf test failed:\nSTDOUT: {}\nSTDERR: {}", stdout, stderr);
    }
    println!("✅ Second tests completed successfully");
    println!("STDOUT: {}", String::from_utf8_lossy(&test_output.stdout));
    // Step 5: Clean up
    fs::remove_dir_all(&temp_dir).expect("Failed to remove test directory");
    println!("🎉 Second test passed! Memo2 project E2E test completed successfully.");
}
