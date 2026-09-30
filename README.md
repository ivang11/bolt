# bolt — Docker Project Manager

CLI to manage Docker Compose projects.

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/ivang11/bolt/main/install.sh | sudo bash
```

The installer downloads the latest release binary for your platform (Linux x86_64,
macOS Intel, or macOS Apple Silicon).

### From source (requires Rust)

```bash
git clone https://github.com/IvanG11/bolt
cd bolt
make build
sudo make install
```

## Commands

| Command                                     | Description                                           |
| ------------------------------------------- | ----------------------------------------------------- |
| `bolt setup`                                | Run the setup wizard                                  |
| `bolt switch <project>`                     | Stop all active projects and start the specified one  |
| `bolt switch <project> --keep`              | Start without stopping others                         |
| `bolt list`                                 | List projects with status ▶/⏹                         |
| `bolt status`                               | Show active containers grouped by project             |
| `bolt cleanup`                             | Preview resources left by deleted projects           |
| `bolt stop`                                 | Stop all active projects in projects_dir              |
| `bolt restart <project>`                    | Restart a project (stop + up), preserving volumes |
| `bolt config show`                          | Show current configuration                            |
| `bolt config set-dir <path>`                | Change the root projects directory                    |
| `bolt config set-ui-port <port>`            | Save the default web UI port                          |
| `bolt config ignore <project>`              | Add a project to the ignore list                      |
| `bolt config unignore <project>`            | Remove a project from the ignore list                 |
| `bolt config set-subdirs <project> <s1,s2>` | Define which subdirs to start for a project           |
| `bolt config clear-subdirs <project>`       | Reset to starting all subdirs                         |
| `bolt ui`                                   | Launch the web UI (browser opens automatically)       |

## Web UI

`bolt ui` serves the full interface on a single port. The browser opens automatically.

```bash
bolt ui            # default port 7000
bolt ui --port 8080
```

To save a different default port (for example, if port 7000 is already in use on macOS):

```bash
bolt config set-ui-port 8080
bolt ui                       # uses the saved port 8080
bolt ui --port 9000            # overrides it for this launch only
```

The default remains 7000 until you change it. The saved port also applies to
`bolt ui --daemon`. If the UI is already running, stop it and launch it again;
for a background server, use `bolt ui --stop` and then `bolt ui --daemon`.

### Running in development

Run the Rust backend and the Vite dev server separately so you get hot-reload on the frontend:

```bash
# terminal 1 — API server
cargo run -- ui

# terminal 2 — Vite dev server (proxies /api to localhost:7000)
cd ui && npm run dev
# open http://localhost:5173
```

### Building for production

The frontend is embedded in the binary at compile time. The release workflow handles this automatically — just run:

```bash
make release
```

If you need to build the binary locally without the workflow, do it manually:

```bash
make build
```

## Storage cleanup

```bash
bolt cleanup                          # preview only
bolt cleanup --apply                  # confirm removal of stopped containers and networks
bolt cleanup --apply --volumes        # also confirm each volume separately
bolt cleanup --volume NAME            # preview a specific unused volume
bolt cleanup --apply --volume NAME    # confirm deletion of that volume and its data
bolt cleanup --apply --images         # also review dangling images across the daemon
bolt cleanup --apply --build-cache    # also review unused cache across the builder
```

Containers are candidates only when Compose labels identify a missing working
folder inside `projects_dir`. Existing, running, ignored and outside projects
are preserved. Networks and volumes must be attributable to those verified
projects; shared resources are excluded. If only an old volume or network
remains, its project name alone cannot prove where the project used to live.
Other unused volumes are listed for manual review with `--volume NAME`.
When used alone, `--volume NAME` shows only the named volume and `--apply`
only offers to delete that selection. Repeat `--volume` to select several.
Adding `--volumes`, `--images` or `--build-cache` includes the broader cleanup report.

The preview reports Docker storage use and exact candidate names. With
`--apply`, confirmations default to No. Removing containers loses their
writable layers; removing a volume permanently deletes its data. Containers
are removed without deleting their volumes. Volumes are excluded unless
explicitly selected, and each selected volume gets its own confirmation.
A manual volume must be unreferenced by every container, including stopped ones.
The project's path and state are checked again before deleting resources.

Images and build cache are shared: their explicit flags apply to the current
Docker daemon / selected builder, rather than just `projects_dir`. `--images`
selects dangling images only. Bolt never runs a global volume or container prune.
See `bolt cleanup --help` for all options.

## Shell completions

`bolt switch <tab>` autocompletes project names. Shell completions are installed automatically during `bolt setup`.

## Configuration

The config file is created automatically at `~/.config/bolt/config.toml` on Linux
and `~/Library/Application Support/bolt/config.toml` on macOS.
Use `bolt config show` to see the exact path.

```toml
projects_dir = "/home/user/Projects"
ignore = ["docker-services"]
ui_port = 7000

[projects.acme]
subdirs = ["acme", "acme-api"]
```

## Stop behaviour

Stopping, switching away from, or restarting a project uses `docker compose stop`.
Containers, networks and volumes are preserved so the next `up -d` can reuse them,
including anonymous volumes. This applies to both the CLI and web UI.
Compose may still recreate containers when their image or configuration changes.
Bolt reports Compose failures instead of claiming the operation succeeded.

- Only stops projects inside `projects_dir`
- Queries `docker ps` directly — no directory iteration
- Respects the `ignore` list — never touches those projects
- Never touches containers outside `projects_dir`

## Projects with subdirectories

If a project has no `docker-compose.yml` at the root but in subdirectories:

```
Projects/
└── acme/
    ├── acme/          ← docker-compose.yml
    ├── acme-api/      ← docker-compose.yml
    ├── acme-legacy/   ← docker-compose.yml (will be skipped)
    └── acme-old/      ← docker-compose.yml (will be skipped)
```

Configure which subdirs to start:

```bash
bolt config set-subdirs acme acme,acme-api
```

## Publishing a new version

**1. Update the version in `Cargo.toml`:**

```toml
version = "1.1.0"
```

**2. Commit the change:**

```bash
git add Cargo.toml
git commit -m "bump v1.1.0"
```

**3. Publish:**

```bash
make release
```

This creates the tag `v1.1.0` and pushes it. GitHub Actions detects the tag,
builds Linux and macOS binaries, and uploads them to GitHub Releases
automatically.

## License

bolt is open-sourced software licensed under the [MIT license](https://opensource.org/licenses/MIT).
