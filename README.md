# Domain Memory

[![CI](https://github.com/ArthurC02/Domain-Memory-PlugIn/actions/workflows/ci.yml/badge.svg)](https://github.com/ArthurC02/Domain-Memory-PlugIn/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A governed, file-backed domain model that coding agents read before they code and keep current after.

Every new agent session starts out knowing nothing about your domain. It works out the business rules again from the code, guesses where they live, and leaves nothing for the next session. Domain Memory gives those rules a home: a **Registry** of reviewed Domain-Driven Design facts that sits in your repository next to the code it describes.

- **Facts are tied to code.** Each Context, rule, invariant, contract and term cites the exact source lines that enforce it, along with a SHA-256 of those lines. When the code changes, the citation is reported as `shifted` or `stale`, so the model cannot drift away from the code without anyone noticing.
- **Only reviewed facts count.** Agents may propose `candidate` records at any time, but nothing becomes a fact until a Change Package passes review: frozen at submission, tested, approved, and proven by a signed commit.
- **Every write is safe.** A write is staged on a copy, validated whole, swapped in atomically and appended to a hash-chained audit log. A crashed write can be finished or rolled back.
- **One binary, no runtime.** A single Rust executable runs on Linux, macOS and Windows and needs only `git`. Prebuilt binaries are committed, so installing the plugin is enough.
- **Works with any agent.** It ships as a standard Agent Skill / Agent Plugin, a plain CLI, and an MCP server whose tools only read.

## Install

Pick the line for where you work. Each installs the latest release; [pinning](#pin-a-version-for-a-team) shows how to hold a team to one.

**Claude Code**, in any session:

```text
/plugin marketplace add ArthurC02/Domain-Memory-PlugIn
/plugin install domain-memory@domain-memory
```

The `domain-memory` Skill is the entry point: it runs `readiness` first and picks the capability the work needs.

**The command line**, on macOS, Linux or Git Bash (needs `curl`):

```bash
curl -fsSL https://github.com/ArthurC02/Domain-Memory-PlugIn/releases/latest/download/install.sh | sh
```

**The command line** on Windows, in PowerShell; it adds itself to your PATH:

```powershell
irm https://github.com/ArthurC02/Domain-Memory-PlugIn/releases/latest/download/install.ps1 | iex
```

Both check the executable against the release's `SHA256SUMS` before it lands, print the version they installed, and say so when the install folder (`~/.local/bin` by default) is not on PATH. Then, inside your repository: `domain-memory readiness`.

**GitHub Actions**:

```yaml
- uses: ArthurC02/Domain-Memory-PlugIn@v6.8.1
- run: |
    domain-memory validate
    domain-memory verify-evidence
    domain-memory verify-audit
```

The action installs the release its own commit belongs to, so pinning it by tag or by commit SHA pins the executable too, and puts it on PATH. `version:` chooses another release (`latest` follows each new one, which makes CI unrepeatable); `sha256sums:` pins the digest of the release's `SHA256SUMS`, so a replaced release asset stops the install. Inside a repository no command needs `--repo-root` or `--registry-root`.

### Git hooks

With `domain-memory` on PATH, from inside the repository:

```bash
domain-memory install-pre-commit-hook
domain-memory install-git-hitl-hook
```

The pre-commit hook stops a commit whose staged code no longer matches what the Registry cites; the pre-push hook checks the signatures on Registry commits. They find the executable through `git config domain-memory.executable` if set, then the plugin a clone of this repository holds, then PATH. An installer run from a release records its own path in the clone's git config, relative to the repository's top when it sits inside it, so each worktree runs its own copy; `--executable <path>` records exactly the path you give instead. A configured path this worktree lacks falls back to PATH, and the hook says which executable runs instead whenever it needs one, that is when the commit or push touches cited code or the Registry; with none on PATH either, the hook stops with the install line above. Hooks regenerate nothing on an upgrade: they call whatever executable they find.

### Pin a version for a team

Pin the same tag everywhere, so the Skills an agent reads, the hooks and CI run one release. In Claude Code, `/plugin marketplace add ArthurC02/Domain-Memory-PlugIn#v6.8.1`; to declare it for everyone who opens the repository, add this to its `.claude/settings.json`, or run `claude plugin marketplace add ArthurC02/Domain-Memory-PlugIn#v6.8.1 --scope project` then `claude plugin install domain-memory@domain-memory --scope project`:

```json
{
  "extraKnownMarketplaces": {
    "domain-memory": {
      "source": {
        "source": "github",
        "repo": "ArthurC02/Domain-Memory-PlugIn",
        "ref": "v6.8.1"
      }
    }
  },
  "enabledPlugins": {
    "domain-memory@domain-memory": true
  }
}
```

Claude Code honours it once the folder is trusted. On the command line, pin with `DOMAIN_MEMORY_VERSION=v6.8.1` in front of `sh` (`$env:DOMAIN_MEMORY_VERSION = 'v6.8.1'` before `irm` in PowerShell); `DOMAIN_MEMORY_INSTALL_DIR` chooses the folder, and `DOMAIN_MEMORY_SHA256SUMS_SHA256` pins the digest of the release's `SHA256SUMS`. In GitHub Actions the action's tag or commit SHA is the pin. Every release is a tag `v<version>`, and its [GitHub Release](https://github.com/ArthurC02/Domain-Memory-PlugIn/releases) carries the executables, `SHA256SUMS` and a build provenance attestation (`gh attestation verify <file> -R ArthurC02/Domain-Memory-PlugIn`).

### Upgrading

- Claude Code: `claude plugin marketplace update domain-memory`, then `claude plugin update domain-memory@domain-memory`; with a pinned `ref`, move it to the new tag first.
- The command line: run the install line again. `domain-memory --version` says what you have.
- GitHub Actions: move the action's tag.
- Git hooks keep working; the [CHANGELOG](CHANGELOG.md) says when a release changes them enough to re-run `install-pre-commit-hook` or `install-git-hitl-hook`.

### Any other agent, or from a clone

Point your agent at [`AGENTS.md`](AGENTS.md). A clone runs the executable committed for its platform through one launcher, `bin/domain-memory` (`bin\domain-memory.cmd` from `cmd` or PowerShell). Linux x86-64, macOS (Apple silicon and Intel) and Windows x86-64 have one; on any other platform, `cargo build --release --locked --manifest-path engine/Cargo.toml` builds it and the launcher runs that build.

### MCP

`domain-memory mcp-serve --registry-root <root> --repo-root <repo>` serves thirteen read commands as Model Context Protocol tools over stdio. It offers no tool that writes. See [`mcp-serve`](references/script-api.md#mcp-serve).

## How it works

```text
         read reviewed facts                 propose / re-cite
  Agent ─────────────────────▶ Registry ◀───────────────────── Agent
                                  ▲
                                  │ ship-package (signed commit)
                                  │
        draft ─▶ submitted ─▶ verified ─▶ approved ─▶ applied
                   Change Package (frozen once submitted)
```

1. **Read.** Before coding, run `readiness`. It tells an empty repository apart from a greenfield design, a brownfield codebase, or a memory whose evidence has gone stale, and it names the capability to start with.
2. **Extract.** Turn requirements or existing code into cited `candidate` records.
3. **Design.** Draft a material change as a Change Package that names the Contexts, rules and contracts it affects.
4. **Maintain.** Once the code is merged, re-cite evidence that moved and record what the model learned.
5. **Review.** Check the Registry, the evidence and the audit chain, or review a branch for what it does to the domain model with `brief --base <branch point>`.

The Skills under [`skills/`](skills/) cover these steps, and the [`domain-memory`](skills/domain-memory/SKILL.md) Skill routes a request to the smallest one that fits.

## Documentation

| Page | For |
| --- | --- |
| [`references/getting-started.md`](references/getting-started.md) | Ten minutes from an empty repository to a cited fact, on a sample you can run. Start here. |
| [`AGENTS.md`](AGENTS.md) | Vocabulary, common tasks and costly mistakes. |
| [`references/script-api.md`](references/script-api.md) | Every command: options, guarantees, refusals |
| [`references/proposal-lifecycle.md`](references/proposal-lifecycle.md) | The Change Package states |
| [`references/reliability-architecture.md`](references/reliability-architecture.md) | The write protocol and the audit chain |
| [`references/host-integration.md`](references/host-integration.md) | Hosts that dispatch work to several agents |
| [`CHANGELOG.md`](CHANGELOG.md) | Release history |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | Building, testing and changing the plugin |

## License

[MIT](LICENSE)
