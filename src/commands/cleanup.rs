use crate::config::Config;
use anyhow::{Context, Result, ensure};
use dialoguer::Confirm;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};
use std::process::Command;

const PROJECT: &str = "com.docker.compose.project";
const WORKING_DIR: &str = "com.docker.compose.project.working_dir";
type Labels = BTreeMap<String, String>;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Container {
    id: String,
    name: String,
    config: ContainerConfig,
    state: ContainerState,
    mounts: Vec<Mount>,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ContainerConfig {
    labels: Option<Labels>,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ContainerState {
    status: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Mount {
    #[serde(default)]
    name: String,
    #[serde(rename = "Type")]
    kind: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Network {
    id: String,
    name: String,
    labels: Option<Labels>,
    containers: Option<BTreeMap<String, serde_json::Value>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Volume {
    name: String,
    labels: Option<Labels>,
}
struct Inventory {
    containers: Vec<Container>,
    networks: Vec<Network>,
    volumes: Vec<Volume>,
}
#[derive(Default)]
struct Plan {
    containers: BTreeSet<String>,
    projects: BTreeSet<String>,
    volumes: BTreeSet<String>,
    networks: BTreeSet<String>,
}

fn docker(args: &[&str]) -> Result<String> {
    let output = Command::new("docker")
        .args(args)
        .output()
        .context("Cannot run Docker")?;
    ensure!(
        output.status.success(),
        "docker {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(String::from_utf8(output.stdout)?)
}

fn inspect<T: serde::de::DeserializeOwned>(kind: &str, ids: &str) -> Result<Vec<T>> {
    let mut result = Vec::new();
    let ids: Vec<_> = ids.split_whitespace().collect();
    for chunk in ids.chunks(100) {
        let mut args = vec![kind, "inspect"];
        args.extend_from_slice(chunk);
        result.extend(serde_json::from_str::<Vec<T>>(&docker(&args)?)?);
    }
    Ok(result)
}

impl Inventory {
    fn load() -> Result<Self> {
        Ok(Self {
            containers: inspect("container", &docker(&["ps", "-aq", "--no-trunc"])?)?,
            networks: inspect("network", &docker(&["network", "ls", "-q", "--no-trunc"])?)?,
            volumes: inspect("volume", &docker(&["volume", "ls", "-q"])?)?,
        })
    }
}

// Resolve existing ancestors too: missing paths must not bypass symlink boundaries.
fn resolve_path(path: &Path) -> Result<PathBuf> {
    ensure!(
        path.is_absolute(),
        "Expected an absolute project path: {}",
        path.display()
    );
    ensure!(
        !path
            .components()
            .any(|part| matches!(part, Component::ParentDir)),
        "Ambiguous project path: {}",
        path.display()
    );
    match path.canonicalize() {
        Ok(path) => Ok(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if let Ok(metadata) = path.symlink_metadata() {
                ensure!(
                    !metadata.file_type().is_symlink(),
                    "Cannot safely resolve broken symlink: {}",
                    path.display()
                );
            }
            let parent = path.parent().context("Cannot resolve project path")?;
            Ok(resolve_path(parent)?.join(path.file_name().context("Missing path component")?))
        }
        Err(error) => Err(error.into()),
    }
}

fn missing_project(
    labels: Option<&Labels>,
    config: &Config,
    root: &Path,
) -> Result<Option<String>> {
    let Some(labels) = labels else {
        return Ok(None);
    };
    let Some(project) = labels.get(PROJECT).filter(|name| !name.is_empty()) else {
        return Ok(None);
    };
    let Some(directory) = labels.get(WORKING_DIR).filter(|path| !path.is_empty()) else {
        return Ok(None);
    };
    let directory = Path::new(directory);
    // Unfamiliar paths (e.g. a remote host) aren't deletion candidates.
    if !directory.is_absolute()
        || directory
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Ok(None);
    }
    let resolved = resolve_path(directory)?;
    let Ok(relative) = resolved.strip_prefix(root) else {
        return Ok(None);
    };
    let Some(Component::Normal(folder)) = relative.components().next() else {
        return Ok(None);
    };
    if config.ignore.contains(project)
        || config
            .ignore
            .iter()
            .any(|ignored| folder == ignored.as_str())
    {
        return Ok(None);
    }
    if directory.try_exists()? {
        return Ok(None);
    }
    Ok(Some(project.clone()))
}

fn plan(inventory: &Inventory, config: &Config, root: &Path) -> Result<Plan> {
    let mut result = Plan::default();
    let mut protected = BTreeSet::new();
    for container in &inventory.containers {
        let labels = container.config.labels.as_ref();
        let project = labels.and_then(|labels| labels.get(PROJECT));
        let missing = missing_project(labels, config, root)?;
        if matches!(
            container.state.status.as_str(),
            "exited" | "created" | "dead"
        ) && let Some(project) = missing
        {
            result.containers.insert(container.id.clone());
            result.projects.insert(project);
        } else if let Some(project) = project {
            protected.insert(project.clone());
        }
    }
    result
        .projects
        .retain(|project| !protected.contains(project));
    result.containers.retain(|id| {
        inventory.containers.iter().any(|container| {
            &container.id == id
                && container
                    .config
                    .labels
                    .as_ref()
                    .and_then(|labels| labels.get(PROJECT))
                    .is_some_and(|project| result.projects.contains(project))
        })
    });
    for network in &inventory.networks {
        if network
            .labels
            .as_ref()
            .and_then(|labels| labels.get(PROJECT))
            .is_some_and(|project| result.projects.contains(project))
            && network
                .containers
                .as_ref()
                .is_none_or(|containers| containers.keys().all(|id| result.containers.contains(id)))
        {
            result.networks.insert(network.id.clone());
        }
    }
    let mut protected_volumes = BTreeSet::new();
    for container in &inventory.containers {
        for mount in &container.mounts {
            if mount.kind == "volume" {
                if result.containers.contains(&container.id) {
                    result.volumes.insert(mount.name.clone());
                } else {
                    protected_volumes.insert(mount.name.clone());
                }
            }
        }
    }
    for volume in &inventory.volumes {
        if volume
            .labels
            .as_ref()
            .and_then(|labels| labels.get(PROJECT))
            .is_some_and(|project| result.projects.contains(project))
        {
            result.volumes.insert(volume.name.clone());
        }
    }
    result
        .volumes
        .retain(|name| !protected_volumes.contains(name));
    Ok(result)
}

fn confirm(prompt: &str) -> Result<bool> {
    Ok(Confirm::new()
        .with_prompt(prompt)
        .default(false)
        .interact()?)
}

fn revalidate_projects(
    original: &Inventory,
    selected: &Plan,
    config: &Config,
    root: &Path,
) -> Result<()> {
    ensure!(
        config.projects_dir.canonicalize()? == root,
        "projects_dir changed; run cleanup again"
    );
    for container in &original.containers {
        if selected.containers.contains(&container.id) {
            ensure!(
                missing_project(container.config.labels.as_ref(), config, root)?.is_some(),
                "Project directory changed; run cleanup again"
            );
        }
    }
    let current: Vec<Container> = inspect("container", &docker(&["ps", "-aq", "--no-trunc"])?)?;
    for container in current {
        if container
            .config
            .labels
            .as_ref()
            .and_then(|labels| labels.get(PROJECT))
            .is_some_and(|project| selected.projects.contains(project))
        {
            ensure!(
                matches!(
                    container.state.status.as_str(),
                    "exited" | "created" | "dead"
                ) && missing_project(container.config.labels.as_ref(), config, root)?.is_some(),
                "Project is now active or its directory exists; run cleanup again"
            );
        }
    }
    Ok(())
}

pub fn run(
    config: &Config,
    apply: bool,
    volumes: bool,
    manual_volumes: &[String],
    images: bool,
    build_cache: bool,
) -> Result<()> {
    if !manual_volumes.is_empty() && !volumes && !images && !build_cache {
        return run_selected_volumes(manual_volumes, apply);
    }
    ensure!(
        config.is_configured(),
        "Configure projects_dir first with bolt config set-dir <path>"
    );
    let root = config
        .projects_dir
        .canonicalize()
        .context("projects_dir must exist and be accessible before cleanup")?;
    ensure!(root.is_dir(), "projects_dir must be a directory");
    println!(
        "Docker storage (current context):\n{}",
        docker(&["system", "df"])?
    );
    let inventory = Inventory::load()?;
    let mut selected = plan(&inventory, config, &root)?;
    let referenced: BTreeSet<_> = inventory
        .containers
        .iter()
        .flat_map(|container| &container.mounts)
        .filter(|mount| mount.kind == "volume")
        .map(|mount| mount.name.as_str())
        .collect();
    for name in manual_volumes {
        ensure!(
            inventory.volumes.iter().any(|volume| &volume.name == name),
            "Volume '{}' does not exist",
            name
        );
        ensure!(
            !referenced.contains(name.as_str()),
            "Volume '{}' is referenced by a container; manual cleanup only accepts unused volumes",
            name
        );
    }
    println!("\nDeleted projects under {}:", root.display());
    for project in &selected.projects {
        println!("  {project}");
    }
    if selected.projects.is_empty() {
        println!(
            "  None verified (volumes/networks without a container's working-directory label cannot establish ownership)."
        );
    }
    for container in &inventory.containers {
        if selected.containers.contains(&container.id) {
            println!(
                "  Container: {} [{}], {}",
                container.name.trim_start_matches('/'),
                container.id,
                container.config.labels.as_ref().unwrap()[WORKING_DIR]
            );
        }
    }
    for network in &inventory.networks {
        if selected.networks.contains(&network.id) {
            println!("  Network: {} [{}]", network.name, network.id);
        }
    }
    println!("\nVolumes of verified deleted projects (may contain persistent data):");
    for name in &selected.volumes {
        println!("  {name}");
    }
    println!("\nOther unused volumes (ownership unverified; review with --volume NAME):");
    for volume in &inventory.volumes {
        if !referenced.contains(volume.name.as_str()) && !selected.volumes.contains(&volume.name) {
            println!("  {}", volume.name);
        }
    }
    if !volumes {
        selected.volumes.clear();
    }
    selected.volumes.extend(manual_volumes.iter().cloned());
    let image_ids = if images {
        docker(&[
            "image",
            "ls",
            "--filter",
            "dangling=true",
            "--no-trunc",
            "-q",
        ])?
    } else {
        String::new()
    };
    let image_ids: BTreeSet<_> = image_ids.split_whitespace().collect();
    if images {
        println!("\nDangling images across the daemon: {}", image_ids.len());
        for id in &image_ids {
            println!("  {id}");
        }
    }
    if build_cache {
        println!(
            "\nBuild cache: shared across projects in the selected builder; Docker will report reclaimed space after pruning."
        );
    }
    for name in manual_volumes {
        println!("\nExplicit volume selection: {name} (may contain persistent data)");
    }
    if !apply {
        println!(
            "\nPreview only. Run again with --apply to confirm removal. Volumes require --volumes or --volume NAME; images/cache require --images/--build-cache."
        );
        return Ok(());
    }
    if !selected.containers.is_empty()
        && confirm(
            "Remove the listed stopped containers? Their writable layers will be lost; volumes are preserved",
        )?
    {
        revalidate_projects(&inventory, &selected, config, &root)?;
        for id in &selected.containers {
            print!("{}", docker(&["container", "rm", id])?);
        }
    }
    if !selected.networks.is_empty() && confirm("Remove the listed networks of deleted projects?")?
    {
        revalidate_projects(&inventory, &selected, config, &root)?;
        for id in &selected.networks {
            print!("{}", docker(&["network", "rm", id])?);
        }
    }
    for name in &selected.volumes {
        let unused = docker(&["volume", "ls", "-q", "--filter", "dangling=true"])?;
        if !unused.lines().any(|unused| unused == name) {
            println!(
                "Skipping volume '{name}': still referenced by a container or already removed."
            );
            continue;
        }
        if confirm(&format!(
            "Permanently delete volume '{name}' and ALL its data?"
        ))? {
            revalidate_projects(&inventory, &selected, config, &root)?;
            // Docker refuses referenced volumes. Never force or prune globally.
            print!("{}", docker(&["volume", "rm", name])?);
        }
    }
    if !image_ids.is_empty()
        && confirm(
            "Remove the listed dangling images across the Docker daemon? They may need rebuilding",
        )?
    {
        for id in image_ids {
            let dangling = docker(&[
                "image",
                "ls",
                "--filter",
                "dangling=true",
                "--no-trunc",
                "-q",
            ])?;
            if !dangling.lines().any(|current| current == id) {
                println!("Skipping image '{id}': no longer dangling.");
                continue;
            }
            print!("{}", docker(&["image", "rm", id])?);
        }
    }
    if build_cache
        && confirm(
            "Remove unused build cache across the selected builder? Subsequent builds may be slower",
        )?
    {
        let status = Command::new("docker")
            .args(["builder", "prune", "--all", "--force"])
            .status()?;
        ensure!(status.success(), "Build cache cleanup failed ({status})");
    }
    println!(
        "\nDocker storage after cleanup:\n{}",
        docker(&["system", "df"])?
    );
    Ok(())
}

fn run_selected_volumes(names: &[String], apply: bool) -> Result<()> {
    let names: BTreeSet<_> = names.iter().map(String::as_str).collect();
    let requested = names.iter().copied().collect::<Vec<_>>().join("\n");
    let volumes: Vec<Volume> = inspect("volume", &requested)?;
    let containers: Vec<Container> = inspect("container", &docker(&["ps", "-aq", "--no-trunc"])?)?;
    // Validate the whole selection before offering to delete any volume.
    for name in &names {
        ensure!(
            volumes.iter().any(|volume| volume.name == *name),
            "Volume '{}' does not exist",
            name
        );
        ensure!(
            !containers
                .iter()
                .flat_map(|container| &container.mounts)
                .any(|mount| mount.kind == "volume" && mount.name == *name),
            "Volume '{}' is referenced by a container; manual cleanup only accepts unused volumes",
            name
        );
        println!(
            "Volume: {name}\n  Status: unused (no container references). Deleting it permanently removes its data."
        );
    }
    if !apply {
        println!("\nPreview only. Add --apply to confirm deletion of the selected volume(s).");
        return Ok(());
    }
    for name in names {
        let unused = docker(&["volume", "ls", "-q", "--filter", "dangling=true"])?;
        if !unused.lines().any(|current| current == name) {
            println!(
                "Skipping volume '{name}': still referenced by a container or already removed."
            );
            continue;
        }
        if confirm(&format!(
            "Permanently delete volume '{name}' and ALL its data?"
        ))? {
            print!("{}", docker(&["volume", "rm", name])?);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn container(id: &str, project: &str, dir: &Path, status: &str, volume: &str) -> Container {
        serde_json::from_value(json!({
            "Id": id, "Name": id,
            "Config": {"Labels": {PROJECT: project, WORKING_DIR: dir.to_str().unwrap()}},
            "State": {"Status": status},
            "Mounts": [{"Type": "volume", "Name": volume}]
        }))
        .unwrap()
    }

    fn fixture() -> (Config, PathBuf, PathBuf) {
        let root = std::env::temp_dir().canonicalize().unwrap();
        let missing = root.join(format!("bolt-cleanup-missing-{}", std::process::id()));
        assert!(!missing.exists());
        (
            Config {
                projects_dir: root.clone(),
                ..Config::default()
            },
            root,
            missing,
        )
    }

    #[test]
    fn deleted_project_resources_are_selected_but_shared_volumes_are_preserved() {
        let (config, root, missing) = fixture();
        let inventory = Inventory {
            containers: vec![container("dead", "gone", &missing, "exited", "shared"), container("live", "existing", &root, "running", "shared")],
            networks: vec![serde_json::from_value(json!({"Id": "network", "Name": "gone_default", "Labels": {PROJECT: "gone"}, "Containers": {}})).unwrap()],
            volumes: vec![serde_json::from_value(json!({"Name": "database", "Labels": {PROJECT: "gone"}})).unwrap()],
        };
        let result = plan(&inventory, &config, &root).unwrap();
        assert!(result.containers.contains("dead"));
        assert!(result.networks.contains("network"));
        assert!(result.volumes.contains("database"));
        assert!(!result.volumes.contains("shared"));
    }

    #[test]
    fn ignored_outside_existing_and_running_projects_are_preserved() {
        let (mut config, root, missing) = fixture();
        config.ignore = vec![
            "ignored".into(),
            missing.file_name().unwrap().to_string_lossy().into_owned(),
        ];
        assert!(
            missing_project(
                Some(
                    &container("x", "gone", &missing, "exited", "v")
                        .config
                        .labels
                        .unwrap()
                ),
                &config,
                &root
            )
            .unwrap()
            .is_none()
        );
        config.ignore = vec!["ignored".into()];
        let outside = root.parent().unwrap().join("bolt-cleanup-outside-missing");
        let inventory = Inventory {
            containers: vec![
                container("ignored", "ignored", &missing, "exited", "v"),
                container("outside", "outside", &outside, "exited", "v"),
                container("existing", "existing", &root, "exited", "v"),
                container("running", "running", &missing, "running", "v"),
            ],
            networks: vec![],
            volumes: vec![],
        };
        assert!(
            plan(&inventory, &config, &root)
                .unwrap()
                .containers
                .is_empty()
        );
    }

    #[test]
    fn same_project_name_elsewhere_protects_all_its_resources() {
        let (config, root, missing) = fixture();
        let inventory = Inventory {
            containers: vec![
                container("gone", "same", &missing, "exited", "v"),
                container("kept", "same", &root, "exited", "v"),
            ],
            networks: vec![],
            volumes: vec![],
        };
        let result = plan(&inventory, &config, &root).unwrap();
        assert!(result.projects.is_empty());
        assert!(result.containers.is_empty());
        assert!(result.volumes.is_empty());
    }

    #[test]
    fn ambiguous_paths_are_never_candidates() {
        let (config, root, _) = fixture();
        for path in [PathBuf::from("relative/gone"), root.join("../gone")] {
            let c = container("c", "gone", &path, "exited", "v");
            assert!(
                missing_project(c.config.labels.as_ref(), &config, &root)
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[test]
    fn symlinks_cannot_make_outside_resources_look_managed() {
        use std::os::unix::fs::symlink;
        let (config, root, missing) = fixture();
        let link = root.join(format!("bolt-cleanup-link-{}", std::process::id()));
        symlink(root.parent().unwrap(), &link).unwrap();
        let path = link.join(missing.file_name().unwrap());
        let c = container("c", "gone", &path, "exited", "v");
        let result = missing_project(c.config.labels.as_ref(), &config, &root);
        std::fs::remove_file(link).unwrap();
        assert!(result.unwrap().is_none());
    }
}
