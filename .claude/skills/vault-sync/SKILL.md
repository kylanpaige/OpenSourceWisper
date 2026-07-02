---
name: vault-sync
description: Re-index the vault — refresh Home.md recent-reports, flag reports missing frontmatter.
triggers: vault sync, sync the vault, reindex the vault
runner: builtin
builtin: vault_sync
hud-icon: ⟳
hud-group: vault
---

# Vault Sync

Keep the memory layer healthy.

1. Ensure the vault layout exists (`reports/`, `inbox/`, `directives/`, `log/`, `Home.md`).
2. Refresh the machine-maintained "Recent reports" section of `Home.md`
   (between the `wisp:recent-reports` markers — never touch anything outside them).
3. Flag reports missing `title` frontmatter.
4. Report counts: notes, reports, inbox items.
