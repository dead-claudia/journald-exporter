pub fn get_binary_path() -> std::path::PathBuf {
    // Cargo puts the integration test binary in target/debug/deps
    let exe_path =
        std::env::current_exe().expect("Failed to get the path of the integration test binary");

    let exe_dir = exe_path
        .parent()
        .expect("Failed to get the directory of the integration test binary");

    let test_bin_dir = exe_dir.parent().expect("Failed to get the binary folder");

    let path = test_bin_dir.join("journald-exporter");

    assert!(path.exists());

    path
}
