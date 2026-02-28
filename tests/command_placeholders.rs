use std::process::Command;

fn run_and_read_stdout(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_camptask"))
        .args(args)
        .output()
        .expect("camptask command should run");
    assert!(
        output.status.success(),
        "command failed: {:?}\nstderr: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("stdout should be UTF-8")
}

#[test]
fn camp_check_lite_placeholder_smoke() {
    let stdout = run_and_read_stdout(&["camp", "check-lite"]);
    assert_eq!(stdout.trim(), "camptask camp check-lite skeleton ready");
}

#[test]
fn resources_update_placeholder_smoke() {
    let stdout = run_and_read_stdout(&["resources", "update"]);
    assert!(stdout.contains("updated resources at"));
}
