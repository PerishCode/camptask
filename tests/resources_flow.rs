use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic enough")
        .as_nanos();
    std::env::temp_dir().join(format!("camptask-res-test-{nanos}"))
}

#[test]
fn resources_init_status_doctor_flow() {
    let resource_home = unique_temp_dir();
    let home = unique_temp_dir();

    let init_output = Command::new(env!("CARGO_BIN_EXE_camptask"))
        .args(["resources", "init"])
        .env("CAMPTASK_RESOURCE_HOME", &resource_home)
        .env("HOME", &home)
        .output()
        .expect("resources init should run");
    assert!(
        init_output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&init_output.stderr)
    );

    let status_output = Command::new(env!("CARGO_BIN_EXE_camptask"))
        .args(["resources", "status"])
        .env("CAMPTASK_RESOURCE_HOME", &resource_home)
        .env("HOME", &home)
        .output()
        .expect("resources status should run");
    assert!(
        status_output.status.success(),
        "status failed: {}",
        String::from_utf8_lossy(&status_output.stderr)
    );
    assert!(
        String::from_utf8(status_output.stdout)
            .expect("status stdout should be UTF-8")
            .contains("resources installed at")
    );

    let doctor_output = Command::new(env!("CARGO_BIN_EXE_camptask"))
        .args(["resources", "doctor"])
        .env("CAMPTASK_RESOURCE_HOME", &resource_home)
        .env("HOME", &home)
        .output()
        .expect("resources doctor should run");
    assert!(
        doctor_output.status.success(),
        "doctor failed: {}",
        String::from_utf8_lossy(&doctor_output.stderr)
    );
    assert!(
        String::from_utf8(doctor_output.stdout)
            .expect("doctor stdout should be UTF-8")
            .contains("resources doctor: pass")
    );

    let broken_file = resource_home.join("prompts/UNIFIED.md");
    std::fs::remove_file(&broken_file).expect("should be able to remove one resource file");

    let dry_run_output = Command::new(env!("CARGO_BIN_EXE_camptask"))
        .args(["resources", "update", "--dry-run"])
        .env("CAMPTASK_RESOURCE_HOME", &resource_home)
        .env("HOME", &home)
        .output()
        .expect("resources update --dry-run should run");
    assert!(
        dry_run_output.status.success(),
        "dry-run failed: {}",
        String::from_utf8_lossy(&dry_run_output.stderr)
    );
    assert!(
        String::from_utf8(dry_run_output.stdout)
            .expect("dry-run stdout should be UTF-8")
            .contains("resources update dry-run")
    );
    assert!(
        !broken_file.exists(),
        "dry-run must not mutate resource files"
    );

    let update_output = Command::new(env!("CARGO_BIN_EXE_camptask"))
        .args(["resources", "update"])
        .env("CAMPTASK_RESOURCE_HOME", &resource_home)
        .env("HOME", &home)
        .output()
        .expect("resources update should run");
    assert!(
        update_output.status.success(),
        "update failed: {}",
        String::from_utf8_lossy(&update_output.stderr)
    );

    let doctor_after_update = Command::new(env!("CARGO_BIN_EXE_camptask"))
        .args(["resources", "doctor"])
        .env("CAMPTASK_RESOURCE_HOME", &resource_home)
        .env("HOME", &home)
        .output()
        .expect("resources doctor should run after update");
    assert!(
        doctor_after_update.status.success(),
        "doctor after update failed: {}",
        String::from_utf8_lossy(&doctor_after_update.stderr)
    );

    let _ = std::fs::remove_dir_all(&resource_home);
    let _ = std::fs::remove_dir_all(&home);
}
