use std::process::Command;

#[test]
fn hello_world_smoke() {
    let output = Command::new(env!("CARGO_BIN_EXE_camptask"))
        .arg("hello")
        .output()
        .expect("camptask hello should run");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert_eq!(stdout.trim(), "hello world");
}
