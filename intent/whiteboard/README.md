# Whiteboard

Live coordination between the Claude Code sessions and the human working on Ccusage. Each participant is a **node** with its own directory under `intent/whiteboard/`. `intent wb` is the only way to write a board: every file under a node directory is a view rendered from the Intent store, and each node's `board.json` is its committed extract. `intent/wip.md` stays the post-session snapshot; the whiteboard is the live channel.

The protocol is the `/in-whiteboard` skill. This file is the Ccusage roster.

## Nodes

Registering a node is hv's declaration, never an agent's tidy-up: `intent wb register <moniker> --name <display> --role <role>`. `intent wb status` lists the registered roster.

| Node | Name                   | Scope (Ccusage)                                                                                                                                                      |
| ---- | ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `hv` | Hypervisor (the human) | adjudicates scope, sequences work, owns releases, tags and pushes; the escalation landing                                                                            |
| `vc` | Validation Claude      | specifies and coordinates: briefs, acceptance lists, the Intent records; validates what `dc` lands against them; **the named reader of `hv/inbox.*`**; advisory only |
| `dc` | DevX Claude            | builds: the Rust crate, its tests, CI, the release tooling and the Intent gate wiring; works to `vc`'s briefs (hv, 2026-11-03)                                       |

`vc` never edits `dc`'s code. A finding goes to `dc`'s inbox, or to hv when it needs a ruling.

## The hv inbox has a named reader: vc

A write surface with no named reader is a queue, not a channel. `vc` reads `hv/inbox.*` at every pickup and surfaces what it holds to the human, and hv adjudicates. A node's escalation is done when `vc` has it, not when the write returns. Where hv is reachable live, say it there as well.

## One checkout, on main

hv, 2026-11-03: the work happens on `main` in this one checkout, with no worktrees for now. Each node commits only the files its own verbs and edits wrote, named one by one, so a commit never sweeps up a peer's work in progress.
