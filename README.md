# Lynk Client

## Overview

Lynk Client is a desktop SSH and SFTP client for Windows, macOS and Linux, built with Tauri 2, React and Rust. It is local-first: there is no account, no server and no telemetry. Configuration can be synchronised between machines through a Git repository chosen by the user, encrypted on the client before it leaves the machine.

## Problem

SSH clients that synchronise hosts across machines usually do so through the vendor's cloud account, which means trusting a third party with server addresses, usernames and sometimes credentials. Lynk explores the alternative: everything stays on the machine, secrets stay in the operating system's keychain, and the only synchronised artefact is an encrypted file stored in a repository the user controls.

## Approach / Architecture

The application has two layers that communicate over Tauri IPC (commands and events):

- **Frontend (webview):** React 19 and TypeScript, terminal rendering with xterm.js, state in Zustand. Sidebar of hosts, groups and tags; tabs with recursive split panes; dual-pane SFTP view; transfer queue; command palette; settings.
- **Backend (Rust):** session manager and SSH client on `russh`, SFTP on `russh-sftp` with cancellable streamed transfers, local terminal on `portable-pty`, persistence in SQLite through `sqlx` with versioned migrations, secrets through the `keyring` crate, an OpenSSH `~/.ssh/config` parser, encrypted export and Git synchronisation.

### Security model

- **Host key verification.** On first connection the server key fingerprint is recorded (trust on first use). On later connections a different fingerprint aborts the SSH handshake and the user is shown the expected and actual fingerprints. Known hosts can be reviewed and forgotten individually.
- **Secrets.** Passwords, key passphrases and the synchronisation token are stored in the operating system's keychain (Windows Credential Manager, macOS Keychain, Secret Service on Linux), never in the SQLite database or in the synchronised repository.
- **Encrypted export and synchronisation.** Hosts, groups, tags, identities, snippets and port forwards are serialised to JSON and encrypted with AES-256-GCM. The key is derived from a user password with Argon2id (m = 19 MiB, t = 2, p = 1), with a fresh 16-byte salt and 12-byte nonce for every export. Private keys, known-host fingerprints, command history, appearance settings and passwords are never exported.
- **Git synchronisation.** The encrypted file (`vault.enc`) is pushed to a private repository 30 seconds after the last change and pulled at start-up, with a remote-wins strategy. Authentication uses an HTTPS token or the system SSH configuration.
- **Application lock.** An optional numeric PIN, hashed with Argon2id, locks the interface; automatic lock after inactivity closes open sessions and port forwards. The PIN is an application-level lock; it does not encrypt data on disk.
- **Updates.** The Tauri updater verifies a minisign signature before installing an update.

### Features

- SSH with password, keys or agent; key generation (Ed25519, RSA 4096) and import (OpenSSH, PEM); several keys per host with priority; reusable identities.
- Multi-hop ProxyJump with cycle detection; local, remote and dynamic (SOCKS5) port forwarding; agent forwarding (Unix socket or Windows OpenSSH agent pipe).
- Terminal tabs with recursive split panes, broadcast input across panes, search in the buffer, configurable shortcuts.
- SFTP: browse, create, rename, delete, upload and download with progress; drag and drop between local and remote panes; editing a remote file in the default local editor, with automatic re-upload on save.
- Snippets with variables, cross-session command history, pre- and post-connect scripts, import of `~/.ssh/config`.
- Optional per-host recording of terminal output to local log files.

## Tech stack

Rust (Tauri 2, russh, russh-sftp, sqlx with SQLite, keyring, argon2, aes-gcm, portable-pty) · TypeScript (React 19, xterm.js, Zustand) · Vite · Biome · GitHub Actions.

## How to run

Installers for Windows, macOS (Intel and Apple Silicon) and Linux are on the [releases page](https://github.com/moha528/lynk/releases/latest). They are not code-signed yet, so the operating system may show a warning at installation; see the [FAQ](./docs/faq.md).

To build from source you need Node.js 20 or later, pnpm 9 or later, stable Rust and the [Tauri system dependencies](https://tauri.app/start/prerequisites/) for your platform.

```bash
git clone https://github.com/moha528/lynk.git
cd lynk
pnpm install
pnpm tauri dev
```

Tests and lint:

```bash
cd src-tauri && cargo test
cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings
pnpm lint
```

The two SSH integration tests are ignored by default; they need an OpenSSH server in Docker:

```bash
docker run --rm -d -p 2222:2222 -e USER_NAME=test -e USER_PASSWORD=test -e PASSWORD_ACCESS=true linuxserver/openssh-server
cd src-tauri && cargo test --all-features -- --ignored ssh
```

User documentation (in French) is in [`docs/`](./docs/): getting started, SSH config import, keyboard shortcuts, synchronisation setup.

## Status

- Version 1.0.5, released on 26 May 2026, with installers for the three platforms built by GitHub Actions.
- 98 backend tests (unit and SQLite tests), of which 2 SSH integration tests are ignored by default. Continuous integration runs Clippy with warnings as errors and the test suite on Ubuntu, macOS and Windows, plus lint and build of the frontend.
- Known limitations: the remote-wins strategy can discard concurrent edits made on two machines before either has pushed; installers are unsigned; the PIN does not encrypt local data.

## License

[Functional Source License 1.1, MIT Future License (FSL-1.1-MIT)](./LICENSE.md). The code can be read, used, modified and redistributed, including for education and research, but not to offer a competing commercial product. Each version becomes available under the MIT license two years after its release.
