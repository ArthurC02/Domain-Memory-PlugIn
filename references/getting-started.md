# Getting started

Ten minutes from an empty repository to a cited candidate fact, and to the check that notices when the code moves away from it. Every command and output on this page is run by `engine/tests/getting_started.rs` against `examples/getting-started`, so what you see is what the current executable prints.

You need `git` and the executable: [install one](../README.md#install), or use `bin/domain-memory` from a clone. The commands below call it `domain-memory`.

## 1. A repository with some code

Copy the sample into a fresh git repository and change into it:

```sh
cp -r examples/getting-started loans && cd loans
git init -q && git add -A && git commit -qm "Library loans"
```

It holds a requirement, `docs/requirements/loans.md`, and the code that enforces it, `src/loans.ts`:

```ts
export const MAXIMUM_LOANS_PER_MEMBER = 3;
export const LOAN_DAYS = 21;

export function mayBorrow(activeLoans: number): boolean {
  return activeLoans < MAXIMUM_LOANS_PER_MEMBER;
}
```

## 2. Ask what the repository has

`readiness` reads, changes nothing, and names the capability to start with. Inside a repository no command needs `--repo-root` or `--registry-root`: they default to the repository's top level and the one Domain Memory it holds.

```console
$ domain-memory readiness
{
  "format": "domain-memory-readiness/v1",
  "state": "brownfield",
...
  "next_capability": "read-and-maintain",
  "registry_root": null,
  "domain_memories": []
}
```

`brownfield` means there is code to learn from, and `domain_memories` is empty: there is no Domain Memory yet.

## 3. Choose the sources

`discover-sources` proposes where the domain is written down. It decides nothing; you choose.

```console
$ domain-memory discover-sources
...
      "kind": "requirements",
      "authority": "intended behavior and acceptance criteria",
      "paths": [
        "docs/requirements"
      ]
...
      "kind": "implementation",
      "authority": "current executable behavior",
      "paths": [
        "src"
      ]
...
```

## 4. Start the Domain Memory

`init-domain-memory` writes the Registry where you say, from the sources you chose, under a policy you choose. `local-draft-only` keeps every record a candidate until you name who approves facts; `Library team` stands for whoever can say these are the right sources.

```console
$ domain-memory init-domain-memory --output domain-memory --source docs/requirements --source src --storage-mode tracked --data-classification internal --review-mode local-draft-only --source-authority "Library team"
Domain Memory initialized. The sources are agent-asserted: you selected them, and Library team, the source authority, has not said they are the right corpus. Show them the selected paths and ask. When they answer, record it with confirm-sources --confirmed-by <the identity they give you>; until then verify-sources reports the map as unverified, which is the truth and not a problem to work around.
```

## 5. Cite the code that enforces a rule

A fact rests on the lines that enforce it. `cite` shows them and their digest:

```console
$ domain-memory cite --path src/loans.ts --lines 4-6 --show
{
  "path": "src/loans.ts",
  "lines": {
    "start": 4,
    "end": 6
  },
  "content_sha256": "sha256:a3f48bacaab727afca8b8ef948244ce6afd56024199531842fa3b795c045cb9f",
  "excerpt_sha256": "sha256:29b32ee9007e86522fdcc0e1277accf3357269b1ddd4001ae4fc4c9ebc231698"
}
src/loans.ts:4: export function mayBorrow(activeLoans: number): boolean {
src/loans.ts: ... 1 lines not shown
src/loans.ts:6: }
```

## 6. Propose the rule as a candidate

A rule belongs to a Context, so propose the Context first. Write `context.json`:

<!-- write: context.json -->
```json
{
  "id": "loans",
  "name": "Loans",
  "purpose": "Lends books to members and takes them back.",
  "implementation_paths": ["src"]
}
```

and `rule.json`, citing the lines from step 5; `upsert-candidate` adds the digest `cite` printed:

<!-- write: rule.json -->
```json
{
  "id": "loan-limit",
  "contexts": ["loans"],
  "statement": "A member may hold at most three loans at a time.",
  "evidence": [{"path": "src/loans.ts", "lines": {"start": 4, "end": 6}}]
}
```

```console
$ domain-memory upsert-candidate --asset contexts --record-file context.json
Candidate record created.
$ domain-memory upsert-candidate --asset rules --record-file rule.json
Candidate record created.
$ domain-memory validate
Registry is valid.
```

A candidate is handoff material for the next agent, not yet a fact. It becomes `reviewed` only through a Change Package; [the proposal lifecycle](proposal-lifecycle.md) walks through one.

## 7. Notice when the code moves away

`verify-evidence` checks every citation against the code as it is now:

```console
$ domain-memory verify-evidence
{
  "results": [
    {
      "asset": "rules",
      "id": "loan-limit",
      "cited_lines": {
        "start": 4,
        "end": 6
      },
      "status": "current",
...
```

Now change the rule in the code, letting a fourth loan through:

<!-- write: src/loans.ts -->
```ts
export const MAXIMUM_LOANS_PER_MEMBER = 3;
export const LOAN_DAYS = 21;

export function mayBorrow(activeLoans: number): boolean {
  return activeLoans <= MAXIMUM_LOANS_PER_MEMBER;
}
```

<!-- exit: 1 -->
```console
$ domain-memory verify-evidence
...
      "status": "stale",
      "path": "src/loans.ts",
      "reason": "the cited excerpt is no longer in src/loans.ts; check that the fact still holds, then recite it"
...
verify-evidence failed: 1 stale citation; run recite on each to cite its current lines
```

It exits 1, so a pre-commit hook (`install-pre-commit-hook`) or CI stops the change until someone decides whether the rule changed or the code is wrong.

## Next

- [AGENTS.md](../AGENTS.md): the vocabulary, the common tasks and the mistakes that cost a redo.
- [The `domain-memory` Skill](../skills/domain-memory/SKILL.md): which capability to use for the work at hand.
- [script-api.md](script-api.md): every command, what it guarantees and how it refuses.
