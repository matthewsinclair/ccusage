---
verblock: "2026-11-03T09:59:43Z:v0.1: vc - Claude's Keychain items: security only, no argv, update only"
---

# RULES.md

Project-specific mandatory rules for Ccusage. Every statement here is a "must" or a "never".

The cross-language principles -- Highlander, PFIC, Thin Coordinator, No Silent Errors -- are stated in `AGENTS.md` under "Rules of the Road", with their rule IDs, and are deliberately not repeated here. Read any of them with `intent claude rules show <id>`; `intent claude rules list --lang <lang>` filters to one language.

This file is for what is true of THIS project and nothing else.

## Claude's Keychain items (macOS)

The reasons are in `ARCHITECTURE.md`, under "Claude's Keychain items are reached only through `/usr/bin/security`".

- Never open a Claude Code Keychain item in-process, through Security.framework or any crate built on it. Read and renew these items only by running `/usr/bin/security`, and only from `src/keychain.rs`. Tests may use Security.framework only on items in a disposable keychain.
- Never put a credential document, or any part of one, in a process argument. It goes to `security -i` on stdin. A document too long for one `security -i` line is refused, not passed some other way.
- Never create a Claude Code Keychain item. A renewal updates the existing item in the keychain that holds it, and fails when the item is missing.
- Never send `security -i` a write, in a test or an experiment, unless the keychain it names exists and the line is within the 4032-byte limit. Otherwise `security` writes to the login keychain.
