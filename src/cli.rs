use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bolt", about = "Docker project manager", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run the setup wizard
    Setup,
    /// Stop all active projects and start the specified one (interactive picker if no project given)
    Switch { project: Option<String> },
    /// Start a project without stopping others (interactive picker if no project given)
    Start { project: Option<String> },
    /// List available projects in projects_dir
    List {
        /// Print project names only, one per line (for shell completions)
        #[arg(long, hide = true)]
        raw: bool,
    },
    /// Show running projects and containers
    Status,
    /// Stop all active projects in projects_dir
    Stop,
    /// Restart a project (stop + up), preserving containers and volumes
    Restart { project: String },
    /// Rebuild Docker images for a project
    Build { project: String },
    /// Preview resources left by deleted projects; use --apply to confirm removal
    Cleanup {
        /// Ask for confirmation and remove the selected resources
        #[arg(long)]
        apply: bool,
        /// Include orphan project volumes, with a separate confirmation per volume
        #[arg(long)]
        volumes: bool,
        /// Review only the named unused volumes unless other cleanup options are included (repeatable)
        #[arg(long, value_name = "NAME")]
        volume: Vec<String>,
        /// Include unused dangling images across the Docker daemon
        #[arg(long)]
        images: bool,
        /// Include unused build cache across the selected Docker builder
        #[arg(long)]
        build_cache: bool,
    },
    /// Launch the web UI (saved port or 7000; override with --port)
    Ui {
        /// Override the saved web UI port for this launch (default: 7000; save with bolt config set-ui-port)
        #[arg(long, value_parser = clap::value_parser!(u16).range(1..))]
        port: Option<u16>,
        /// Run in the background (detach from terminal)
        #[arg(long, short = 'd')]
        daemon: bool,
        /// Stop a running background UI server
        #[arg(long)]
        stop: bool,
    },
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn help_explains_port_configuration() {
        let mut command = Cli::command();
        let ui_help = command
            .find_subcommand_mut("ui")
            .unwrap()
            .render_long_help()
            .to_string();
        assert!(ui_help.contains("--port"));
        assert!(ui_help.contains("7000"));
        assert!(ui_help.contains("bolt config set-ui-port"));
        let config_help = command
            .find_subcommand_mut("config")
            .unwrap()
            .render_long_help()
            .to_string();
        assert!(config_help.contains("set-ui-port"));
    }

    #[test]
    fn port_arguments_require_a_valid_port() {
        for port in ["0", "65536", "-1", "abc"] {
            assert!(Cli::try_parse_from(["bolt", "ui", "--port", port]).is_err());
            assert!(Cli::try_parse_from(["bolt", "config", "set-ui-port", port]).is_err());
        }
        for port in ["1", "8080", "65535"] {
            assert!(Cli::try_parse_from(["bolt", "ui", "--port", port]).is_ok());
            assert!(Cli::try_parse_from(["bolt", "config", "set-ui-port", port]).is_ok());
        }
    }
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Show current configuration
    Show,
    /// Change the root projects directory
    SetDir { path: String },
    /// Set the default web UI port (takes effect on next launch)
    SetUiPort {
        #[arg(value_parser = clap::value_parser!(u16).range(1..))]
        port: u16,
    },
    /// Add a project to the ignore list
    Ignore { project: String },
    /// Remove a project from the ignore list
    Unignore { project: String },
    /// Define which subdirectories to start for a project
    /// Example: bolt config set-subdirs acme acme,acme-api
    SetSubdirs {
        project: String,
        #[arg(value_delimiter = ',')]
        subdirs: Vec<String>,
    },
    /// Clear subdir config for a project (will start all subdirs)
    ClearSubdirs { project: String },
}
