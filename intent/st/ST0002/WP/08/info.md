---
wp_id: WP-08
title: Herdr mod: a ccusage panel in a Herdr pane, its install verb, and one refresh policy
scope: M
status: Not Started
---

# WP-08: Herdr mod: a ccusage panel in a Herdr pane, its install verb, and one refresh policy

## Objective

A Herdr mod ships inside ccusage, and one verb puts it into the local Herdr. Opened as a pane, it shows what the TUI's overview shows, drawn natively: each account's weekly bar and reset, the active and `next` markers, and on a click the verbose detail. It refreshes by itself and never makes ccusage ask Claude's usage endpoint more often than its cache allows.

- **Builder:** ic. **Depends on:** WP03 and WP05 for the JSON it draws, and WP06 for the TUI's side of the refresh. hv asked for it on 2026-11-13.
- **What Herdr gives** (read from `~/.config/herdr-app/AGENTS.md` at herdr-app 0.1.15): a mod is a folder, `mods/<name>/`, with a `main.js` and a `mod.json`, drawn natively from JavaScript. It can run a program with `exec`, repeat on `cx.timer.every`, keep state and open overlays. It has no `setTimeout`, no HTML and no Node. `herdr-app check <folder>` loads it once and reports what is wrong. `{"place": "none"}` docks it nowhere until asked, and `pane.split {direction, mod}` opens it as a pane.
- **The mod draws and ccusage decides (Thin Coordinator):** no rule, rank or state is worked out in JavaScript. The panel calls `ccusage which --json`, `ccusage predict --json` and `ccusage usage --all --json` and lays out what they return, so its `next` marker and reason are WP02's and cannot drift from the CLI's.
- **What it shows:** for each account, its name, the weekly bar with its exact percent, the reset as a countdown, each scoped window (Fable today), and its state: active, `next` with the reason, at cap, or unknown with why. Below them, predict's burn rate and projected loss, and when the data was fetched. A click on an account opens its detail, with every field the `--verbose` view prints for it. It follows Herdr's theme and its own width, from a narrow column to a full pane.
- **Refresh, and the endpoint:** only ccusage talks to the usage endpoint, and only through its cache. The panel asks ccusage on a timer whose default is the cache's lifetime and which can be set longer, never shorter. So an account is fetched at most once per lifetime however many panels, TUIs and scripts are open, and a 429 parks that account for at least 30 minutes for all of them at once, which the panel shows as the report's own line. Countdowns tick from the clock and fetch nothing. Herdr keeps a hidden mod's timers running, and this bound covers that. With five accounts the ceiling is 60 requests an hour, the rate Claude Usage polls at today.
- **The TUI follows the same policy:** it re-fetches an account once its data has passed the cache's lifetime, so `ccusage tui` left open in a plain Herdr pane stays current and WP06's marker stays on screen.
- **Not yet:** a longer lifetime for accounts nothing is running on, which would cut the requests further. It waits until the endpoint is seen to push back.
- **Ships inside the binary:** the mod's files are embedded at build time, so every install of ccusage carries the mod that matches it, with no change to the formula or the archive.
- **The verb:** proposed `ccusage herdr install`, since the dev/opt/brew contract keeps a bare `install` for devbin; its spelling is ic's and hv's. It finds Herdr's home (`$HERDR_APP_HOME`, else `~/.config/herdr-app`) and refuses when there is none. It writes `mods/ccusage/` and records what it wrote. It replaces only a copy it wrote that nobody has edited, and refuses anything else without `--force`. It then runs `herdr-app check` when that is on PATH and reports the verdict. It never touches a layout or `keys.json`. It ends by saying that the mod runs `ccusage` and nothing else, and how to open it: `herdr-app do pane.split '{"direction":"right","mod":"ccusage"}'`. `ccusage herdr remove` takes the copy out on the same terms.
- **A click that switches:** on an account the panel offers `ccusage use NAME`, behind Herdr's confirm dialog. With ST0003 that switches every session.
- **RECIPE.md and a preview** sit beside the mod's source, as Herdr asks of a mod another person may build from.
- **Tests:** the verb in the `Sandbox`, with a scratch `HERDR_APP_HOME` and a stand-in `herdr-app`, as `launchctl` has one: a fresh install, its own copy replaced, an edited copy refused, the check's verdict reported, and remove. For the mod itself, `herdr-app check` passes on the builder's machine, and a `herdr-app shot` of the panel goes to hv. Running the verb against hv's own Herdr is hv's command.
- **Done when:** AC-08.1 and AC-08.2 are green, and AC-08.3 is satisfied.

## Acceptance

Acceptance Criteria for this work package are RENDERED into `ST0002/acceptance.md`, under the `WP-08` heading. THAT FILE IS A GENERATED VIEW -- a row authored there is discarded by the next sync. The contract is canon in the thread's model, and the verbs write it: `intent ac new` and `intent at new` mint a row, `intent ac edit` and `intent at edit` reword or re-cite one, and `intent ac satisfy|unsatisfy|descope|rescope|withdraw|reinstate` and `intent at green|red|na` move its state. This cover never restates them.

---

_Generated by Intent v3 from the thread canon. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
