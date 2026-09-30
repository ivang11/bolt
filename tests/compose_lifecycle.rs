#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

struct Fixture {
    dir: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "bolt-lifecycle-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(dir.join("bin")).unwrap();
        let projects = dir.join("projects");
        for project in ["app", "ignored", "multi/a", "multi/b"] {
            let path = projects.join(project);
            fs::create_dir_all(&path).unwrap();
            fs::write(path.join("docker-compose.yml"), "services: {}\n").unwrap();
        }
        let config_dir = if cfg!(target_os = "macos") {
            dir.join("Library/Application Support/bolt")
        } else {
            dir.join("config/bolt")
        };
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(
            config_dir.join("config.toml"),
            format!("projects_dir = {:?}\nignore = [\"ignored\"]\n", projects),
        )
        .unwrap();
        let docker = dir.join("bin/docker");
        fs::write(
            &docker,
            r#"#!/bin/sh
printf '%s\n' "$*" >> "$BOLT_TEST_DIR/calls"
if [ "$1" = ps ]; then
    [ "$BOLT_TEST_FAIL" = ps ] && exit 1
    printf 'app§%s/projects/app\n' "$BOLT_TEST_DIR"
    printf 'app§%s/projects/app\n' "$BOLT_TEST_DIR"
    printf 'ignored§%s/projects/ignored\n' "$BOLT_TEST_DIR"
    printf 'outside§%s/outside\n' "$BOLT_TEST_DIR"
    exit 0
fi
[ "$1" = compose ] || exit 2
case "$4" in
    ps) [ "$BOLT_TEST_FAIL" = compose-ps ] && exit 1; echo container ;;
    stop) [ "$BOLT_TEST_FAIL" = stop ] && exit 1 ;;
    up) [ "$BOLT_TEST_FAIL" = up ] && exit 1 ;;
    *) exit 2 ;;
esac
exit 0
"#,
        )
        .unwrap();
        fs::set_permissions(docker, fs::Permissions::from_mode(0o755)).unwrap();
        Self { dir }
    }

    fn run(&self, args: &[&str], fail: &str) -> Output {
        let path = std::env::join_paths(std::iter::once(self.dir.join("bin")).chain(
            std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()),
        ))
        .unwrap();
        Command::new(env!("CARGO_BIN_EXE_bolt"))
            .args(args)
            .env("HOME", &self.dir)
            .env("XDG_CONFIG_HOME", self.dir.join("config"))
            .env("PATH", path)
            .env("BOLT_TEST_DIR", &self.dir)
            .env("BOLT_TEST_FAIL", fail)
            .output()
            .unwrap()
    }

    fn calls(&self) -> String {
        fs::read_to_string(self.dir.join("calls")).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.dir).ok();
    }
}

#[test]
fn stop_keeps_containers_and_only_targets_managed_projects() {
    let fixture = Fixture::new();
    let output = fixture.run(&["stop"], "");
    assert!(output.status.success());
    let calls = fixture.calls();
    let compose: Vec<_> = calls
        .lines()
        .filter(|line| line.starts_with("compose "))
        .collect();
    assert_eq!(compose.len(), 1, "{calls}");
    assert!(compose[0].ends_with("/app/docker-compose.yml stop"));
    assert!(!calls.contains(" down"));
}

#[test]
fn restart_stops_and_starts_without_removing_volumes() {
    let fixture = Fixture::new();
    assert!(fixture.run(&["restart", "app"], "").status.success());
    let calls = fixture.calls();
    let operations: Vec<_> = calls.lines().collect();
    assert_eq!(operations.len(), 3, "{calls}");
    assert!(operations[1].ends_with(" stop"));
    assert!(operations[2].ends_with(" up -d"));
}

#[test]
fn failed_stop_prevents_switch_from_starting_another_project() {
    let fixture = Fixture::new();
    let output = fixture.run(&["switch", "app"], "stop");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("compose stop failed"));
    assert!(!fixture.calls().contains(" up -d"));
}

#[test]
fn failed_queries_are_reported() {
    for (args, fail) in [
        (&["stop"][..], "ps"),
        (&["restart", "app"][..], "compose-ps"),
    ] {
        let fixture = Fixture::new();
        assert!(!fixture.run(args, fail).status.success());
        assert!(!fixture.calls().contains(" up -d"));
    }
}

#[test]
fn failed_start_is_reported_for_root_and_parallel_subdirectories() {
    for project in ["app", "multi"] {
        let fixture = Fixture::new();
        let output = fixture.run(&["start", project], "up");
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("compose up -d failed"));
        let calls = fixture.calls();
        let starts = calls
            .lines()
            .filter(|line| line.ends_with(" up -d"))
            .count();
        assert_eq!(starts, if project == "multi" { 2 } else { 1 });
    }
}

impl Fixture {
    fn cleanup_inventory(&self) {
        let containers = serde_json::json!([
            {"Id": "gone-id", "Name": "/gone", "Config": {"Labels": {
                "com.docker.compose.project": "gone",
                "com.docker.compose.project.working_dir": self.dir.join("projects/gone")
            }}, "State": {"Status": "exited"}, "Mounts": [{"Type": "volume", "Name": "data"}]},
            {"Id": "kept-id", "Name": "/kept", "Config": {"Labels": {
                "com.docker.compose.project": "app",
                "com.docker.compose.project.working_dir": self.dir.join("projects/app")
            }}, "State": {"Status": "exited"}, "Mounts": [{"Type": "volume", "Name": "kept-data"}]}
        ]);
        fs::write(self.dir.join("containers.json"), containers.to_string()).unwrap();
        fs::write(self.dir.join("networks.json"), r#"[{"Id":"network-id","Name":"gone_default","Labels":{"com.docker.compose.project":"gone"},"Containers":{}}]"#).unwrap();
        fs::write(self.dir.join("volumes.json"), r#"[{"Name":"data","Labels":null},{"Name":"kept-data","Labels":null},{"Name":"unknown","Labels":null}]"#).unwrap();
        fs::write(
            self.dir.join("bin/docker"),
            r#"#!/bin/sh
printf '%s\n' "$*" >> "$BOLT_TEST_DIR/calls"
[ "$BOLT_TEST_FAIL" = inventory ] && exit 1
case "$1 $2" in
    'system df') echo storage ;;
    'ps -aq') printf 'gone-id\nkept-id\n' ;;
    'container inspect') cat "$BOLT_TEST_DIR/containers.json" ;;
    'network ls') echo network-id ;;
    'network inspect') cat "$BOLT_TEST_DIR/networks.json" ;;
    'volume ls') printf 'data\nkept-data\nunknown\n' ;;
    'volume inspect') cat "$BOLT_TEST_DIR/volumes.json" ;;
    'image ls') echo sha256:dangling ;;
    *) echo 'Unexpected mutation' >&2; exit 9 ;;
esac
"#,
        )
        .unwrap();
    }
}

#[test]
fn cleanup_preview_is_read_only_and_lists_exact_candidates() {
    let fixture = Fixture::new();
    fixture.cleanup_inventory();
    let output = fixture.run(&["cleanup", "--volumes", "--images", "--build-cache"], "");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("Container: gone [gone-id]"));
    assert!(!text.contains("Container: kept"));
    assert!(text.contains("gone_default"));
    assert!(text.contains("unknown"));
    assert!(text.contains("sha256:dangling"));
    assert!(text.contains("Preview only"));
    assert!(!fixture.calls().contains(" rm "));
    assert!(!fixture.calls().contains("prune"));
}

#[test]
fn cleanup_rejects_manual_volumes_used_by_stopped_containers() {
    let fixture = Fixture::new();
    fixture.cleanup_inventory();
    let output = fixture.run(&["cleanup", "--apply", "--volume", "kept-data"], "");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("referenced by a container"));
    assert!(!fixture.calls().contains(" rm "));
}

#[test]
fn cleanup_propagates_inventory_failure_without_deleting() {
    let fixture = Fixture::new();
    fixture.cleanup_inventory();
    let output = fixture.run(&["cleanup", "--apply"], "inventory");
    assert!(!output.status.success());
    assert!(!fixture.calls().contains(" rm "));
}

#[test]
fn explicit_volume_preview_only_reports_and_queries_the_selection() {
    let fixture = Fixture::new();
    fixture.cleanup_inventory();
    let output = fixture.run(&["cleanup", "--volume", "unknown"], "");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("Volume: unknown"));
    assert!(text.contains("Status: unused"));
    assert!(text.contains("Add --apply"));
    assert!(!text.contains("Docker storage"));
    assert!(!text.contains("Deleted projects"));
    assert!(!text.contains("gone_default"));
    assert!(!text.contains("kept-data"));
    let calls = fixture.calls();
    assert!(calls.contains("volume inspect unknown"));
    assert!(!calls.contains("network"));
    assert!(!calls.contains("system df"));
    assert!(!calls.contains(" rm "));
}

#[test]
fn explicit_volume_apply_does_not_offer_unrelated_project_cleanup() {
    let fixture = Fixture::new();
    fixture.cleanup_inventory();
    // No terminal is provided: confirmation must fail before any deletion.
    let output = fixture.run(&["cleanup", "--apply", "--volume", "unknown"], "");
    assert!(!output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("Volume: unknown"));
    assert!(!text.contains("Container: gone"));
    let calls = fixture.calls();
    assert!(!calls.contains("network"));
    assert!(!calls.contains(" rm "));
}
