# Contributing to Domain Memory

Agents start at `AGENTS.md`, the entry for an Agent without Skill support; this page is for people changing the plugin itself. Report a bug or a friction as an issue; a pull request runs the checks below on Linux, macOS and Windows. `skills/` holds the Skills the manifest points to: the `domain-memory` router and the seven capabilities it chooses among, and `references/` and `templates/` are shared by all of them. Every command is one executable, `domain-memory`, built from `engine/` in Rust, with its Linux, macOS and Windows builds committed under `engine/bin/`; its [README](engine/README.md) has the rules, the checks, and the rebuild every engine change needs.

Requirements: a Rust toolchain (`engine/rust-toolchain.toml` pins it) and `git`; `init-signing-key` alone also needs OpenSSH's `ssh-keygen`. A copy of the plugin that only runs the commands needs neither Rust nor anything else: the committed executable for its platform is enough.

Run from `engine/`:

```sh
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
DOMAIN_MEMORY_REFERENCE_REPO=<reference clone> cargo test --release --locked --test expected_output
```

`cargo test` runs the unit tests in each module, `tests/hooks.rs` (the installed git hooks against real repositories) and `tests/docs.rs` (the links, command reference and contract this plugin's pages hold). `expected_output` runs every recorded case through the executable and takes a few minutes; `--only <text>` keeps the cases whose name contains the text. Run all of them before shipping.

Some recorded cases read a real repository with a reviewed Registry and a long history: [SkillHub](https://github.com/ArthurC02/SkillHub), at the commit `engine/parity/expected/posix.json` names as `pin`. Every case runs inside a checkout of that commit, taken from the clone `DOMAIN_MEMORY_REFERENCE_REPO` names. Without one, `expected_output` runs the rest inside an empty repository and says how many it skipped (except under CI, where `CI` is set and it refuses instead): those that name `{repo}` in their arguments or recording, or that set `"reference": true` because they reach it through a relative path. CI always runs all of them. To run them yourself:

```sh
git clone --no-checkout https://github.com/ArthurC02/SkillHub.git ../reference
```

`-- --bless` writes the executable's output into `engine/parity/expected/` instead of comparing: run it on Linux to write `posix.json`, then on Windows, which records only the cases whose output differs.

## How the code is laid out

`src/main.rs` is the only entry point. It parses the arguments against `contract/cli.json` with `cli.rs` and dispatches each command, most of them to one function in `read_commands.rs`, `write_commands.rs`, `evidence_commands.rs`, `source_commands.rs`, `lifecycle_commands.rs`, `hitl_commands.rs`, `review_commands.rs`, `governance_commands.rs` or `audit_commands.rs`, or to `mcp.rs` for `mcp-serve`. A command prints its result on stdout; a refusal, returned as an `Err` with its reason, reaches `main`, which prints `ERROR: <reason>` on stderr and exits 1. The rules belong in the modules below, all in `engine/src/`:

| Module | Owns |
| --- | --- |
| `common.rs`, `files.rs`, `json.rs` | Asset names and formats, bounded JSON loading, the atomic `write_json`, the writer lock |
| `validate.rs`, `context.rs`, `coverage.rs`, `crossrefs.rs`, `indexes.rs`, `listing.rs`, `search.rs` | Validating a Registry, lookups, terms, coverage, search |
| `store.rs`, `records.rs`, `registry_write.rs` | Reading and writing Registry documents, whichever layout holds them, and reading them at a commit |
| `okf.rs`, `okf_write.rs` | One record as one Open Knowledge Format concept document, and the YAML subset its frontmatter uses |
| `compare.rs` | Scoring an extracted Registry against a reviewed one |
| `transaction.rs` | `mutate_registry`, the one write route: lock, stage a copy, validate, swap, audit, recover |
| `audit_log.rs`, `audit_append.rs` | The hash-chained `audit/events.jsonl` and its manifest |
| `evidence.rs`, `cite.rs` | Building a citation and judging one (`current`, `shifted`, `stale` and the rest) |
| `relocate.rs`, `recitation_package.rs` | Re-citing moved or edited evidence, and the Change Package for it |
| `sources.rs` | Walking the repository, the source map and its snapshots, the source policy |
| `readiness.rs`, `gates.rs`, `impact.rs`, `secrets.rs`, `coupling.rs`, `dependents.rs` | Read-only reports: readiness, quality gates, boundary impact, secret scan, change coupling |
| `policy.rs` | The policy file: defaults, validation, amendments |
| `changes.rs`, `lifecycle.rs`, `proposals.rs` | Validating a Change Package at each lifecycle stage |
| `hitl.rs` | Moving a proposal through submit, approve, verify and finalize |
| `updates.rs`, `promotion.rs`, `harvest*.rs` | Candidates, and applying an approved package |
| `revision.rs` | The Registry digest a proposal is pinned to, and when it counts as stale |
| `attestations.rs`, `signing.rs`, `git_governance.rs`, `git.rs` | Outside proof: signed commits and the pre-push hook; every git call and its deadline |
| `counterfactual.rs`, `test_output.rs` | Breaking one line to prove a test protects it; reading a test run's verdicts |

Three rules keep this shape. Once a Registry exists, its records change only through `mutate_registry`; every Registry read and write goes through `store.rs`, so no other module knows the file layout. Every JSON document the plugin builds is written through `write_json`. A rule that two modules need lives once, in the lower one, and the other uses it.

Any change to what ships — everything except `engine/tests/` and `engine/parity/` — raises `version` in `plugins/domain-memory/.claude-plugin/plugin.json`, which the executable embeds and the MCP server reports, so installed copies see it; rebuilds the committed executables; and adds an entry to `CHANGELOG.md`.

The plugin is the folder `plugins/domain-memory/`, and an install copies that folder only, so everything the plugin reads or runs lives in it: the manifest, Skills, references, templates, the launchers in `scripts/` and the executables in `engine/bin/`. The engine source, its tests and their recordings stay at the repository root. The folder keeps to the [plugin directory's checks](https://claude.com/docs/plugins/pre-submission-checklist): no top-level `bin/`, every file under 5 MiB, which is why the release profile optimizes for size, and no `export-ignore` in any `.gitattributes`. The manifest names only fields the [manifest reference](https://code.claude.com/docs/en/plugins/manifest-reference) defines; a host strips anything else.

Before release, validate the Plugin manifest with the host's validator and run the checks above. A valid manifest proves the packaging shape; it does not prove that a host invokes every capability.
