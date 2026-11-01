# Security policy

## Supported versions

Only the latest release on `main` gets fixes.

## Reporting a vulnerability

Do not open a public issue. Report it through [GitHub private vulnerability reporting](https://github.com/ArthurC02/Domain-Memory-PlugIn/security/advisories/new) with the plugin version, the command and what it let you do. You can expect an answer within seven days.

These are in scope:

- a command that writes outside the Registry root or the repository it was given;
- a reviewed record that changes without a Change Package and a signed commit;
- an audit chain that verifies after it was tampered with;
- a secret that reaches a Registry, a brief or MCP output despite `scan-secrets`;
- an MCP tool that writes.
