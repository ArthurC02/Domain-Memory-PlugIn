# Domain Memory

![Domain Memory](assets/icon.png)

A governed, file-backed domain model that coding agents read before they code and keep current after. Domain Memory keeps reviewed Domain-Driven Design facts (Contexts, rules, contracts, vocabulary) in a Registry inside your repository, each cited to the code that states it. Agents read those facts before changing code, propose new ones as candidates, and change a reviewed fact only through a Change Package that a person approves.

## Install

In Claude Code:

```text
/plugin marketplace add ArthurC02/Domain-Memory-PlugIn
/plugin install domain-memory@domain-memory
```

The `domain-memory` Skill is the entry point: it runs `readiness` first and picks the capability the work needs. [Getting started](references/getting-started.md) runs the first commands on a sample repository, and [AGENTS.md](AGENTS.md) is the guide for any agent. Installing the command line on its own, pinning a release for a team and upgrading are in the [source repository](https://github.com/ArthurC02/Domain-Memory-PlugIn#install).

## What it runs

- The Skills run one executable, `domain-memory`, through `scripts/domain-memory`, which picks the build this plugin bundles in `engine/bin/` for Linux x86-64, macOS or Windows x86-64. `engine/bin/manifest.json` records each build's SHA-256 and the SHA-256 of the source it was built from, and each [release](https://github.com/ArthurC02/Domain-Memory-PlugIn/releases) carries a build provenance attestation for the same files.
- The executable runs `git` and reads and writes files only under the repository it is given: the Registry, Change Packages, and, when asked to, the clone's git hooks and its `domain-memory.executable` git config.
- Nothing in the plugin opens a network connection, sends data anywhere, or reads credentials.

## License

MIT, as `plugin.json` states.
