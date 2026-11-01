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

Every release is a tag, `v<version>`, with a [GitHub Release](https://github.com/ArthurC02/Domain-Memory-PlugIn/releases) carrying the executables, their `SHA256SUMS` and a build provenance attestation. Pin a tag, so the Skills an agent reads and the executable your hooks and CI run are the same version.

### Claude Code

```text
/plugin marketplace add ArthurC02/Domain-Memory-PlugIn#v6.5.0
/plugin install domain-memory@domain-memory
```

The install name is `<plugin>@<marketplace>`, and both are called `domain-memory`. Without `#<tag>` the marketplace follows `main`. To declare it for everyone who works in a repository, put this in its `.claude/settings.json`; Claude Code honours it once the folder is trusted, and offers each person the install:

```json
{
  "extraKnownMarketplaces": {
    "domain-memory": {
      "source": {
        "source": "github",
        "repo": "ArthurC02/Domain-Memory-PlugIn",
        "ref": "v6.5.0"
      }
    }
  },
  "enabledPlugins": {
    "domain-memory@domain-memory": true
  }
}
```

The `domain-memory` Skill is the entry point: it runs `readiness` first and picks the smallest of the seven capabilities.

### Git hooks, CI and scripts

One executable, checked against the release's `SHA256SUMS` before it lands:

```bash
curl -fsSL https://github.com/ArthurC02/Domain-Memory-PlugIn/releases/download/v6.5.0/install.sh | sh -s -- --version v6.5.0 --to .domain-memory/bin
```

```powershell
& ([scriptblock]::Create((irm https://github.com/ArthurC02/Domain-Memory-PlugIn/releases/download/v6.5.0/install.ps1))) -Version v6.5.0 -To .domain-memory\bin
```

Each prints where the executable landed. The hooks that `install-git-hitl-hook` and `install-pre-commit-hook` write run the executable that `git config domain-memory.executable <path>` names, and print `git config domain-memory.install-hint <text>` when it is missing, so a repository that installs it this way points the hooks at it once. Run a script from inside a git hook? Unset the variables git sets for hooks first, `unset $(git rev-parse --local-env-vars)`, or a nested `git -C <other repository>` answers for the hook's repository.

### Any other agent, or from a clone

Point your agent at [`AGENTS.md`](AGENTS.md). A clone runs the executable committed for its platform through one launcher, `bin/domain-memory` (`bin\domain-memory.cmd` from `cmd` or PowerShell):

```bash
git clone --branch v6.5.0 https://github.com/ArthurC02/Domain-Memory-PlugIn.git domain-memory
domain-memory/bin/domain-memory readiness --repo-root path/to/your/repo
```

Linux x86-64, macOS (Apple silicon and Intel) and Windows x86-64 have a committed executable. On any other platform, build it with `cargo build --release --locked --manifest-path engine/Cargo.toml`; the launcher runs that build.

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
