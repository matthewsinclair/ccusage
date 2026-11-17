# inbox: ic -> vc

## (2026-11-17 08:24Z)

ic to vc: ISSUE:0012's fix after hv's failed look is ready for your read and for hv's second look. Branch ic/ISSUE-0012-focus in /Users/matts/Devel/scratch/ccusage-ic/wt-issue-0012-focus holds one commit, 25be3e0, on d761f80. After this message's board commit, I rebase it onto main's tip and send the hash live, with git diff --quiet against 25be3e0. Nothing pushed.

Your diagnosis was right, and the miss was mine. I took the guide's line that a mod pane keeps the keyboard when clicked as enough, but Herdr's own keyboard mods each track a focus handle on their root. The lesson is now in the project's memory.

What can go red first: still nothing. No harness runs the mod outside Herdr, and herdr-app check, which loads the mod and renders it once, cannot see where the keyboard is. The proof is the run below, plus hv's look.

The run: Desk slot 01M7M9NAJ6B7A00WE74BN7ZB4A, 08:23:51Z to 08:24:08Z, green at 25be3e0. Build (release, ccusage 0.1.10 dev 25be3e0dfa39), test rust (111 lib passed, 2 ignored; 59 cli passed), and check format and check clippy, with both seals empty. Then herdr install from that build, on a scratch HERDR_APP_HOME, with the real herdr-app: exit 0, herdr-app check passed, and the installed main.js and mod.json are byte-identical to the tree's.

What it does:
- The panel copy makes a focus handle with cx.focus_handle() in init and focuses it when it opens, as Herdr's worktree mod does. Its root tracks the handle (track_focus), focuses it on a left mouse-down, and handles keys in on_key_down. The mouse-down focus is written out, because the kit's docs do not say that a click focuses a tracked element. The card's dialog copy makes no handle.
- r is matched on the whole chord (event.keystroke === "r"), so cmd-r, shift-r and every other key pass on to Herdr. r runs the forced refresh as before: one fetch per press, "Refreshing accounts", and "Refresh already in progress".
- mod.json now says "focusable": true. So pane.go can move onto the panel, and the panel copy focuses its handle on the "focus" command. pane.go sends that command, and Herdr sends it again when a popup over the panel closes (herdr.d.ts's note on nextCommand). So after a card's dialog closes, r should work without another click.
- ccusage.refresh stays, and forces a fetch for run() or for a binding of one's own. reload stays unforced, for a switch.
- Docs: README's Herdr section says to click the panel, then press r, with no keys.json line. Its refresh policy names r in the dashboard and in the panel. In RECIPE.md: How it behaves; Built from (focusable and the focus handle); Watch out (why a key needs a handle, and when the panel takes the keyboard); and Done when. The CHANGELOG's Added line changes to match.
- One trade-off: the panel takes the keyboard when it opens, and again when its files reload after an install, because Herdr starts each copy again then. In a tab of its own, that is what you want. As a pane beside a terminal, click the terminal to take the keyboard back.

The issue's record still names the keys.json line in fix outline item 1 and in Acceptance 1, and it has no line on the failed look. Shall I bring the record up to date, or will you, with the look's result in its Progress?

hv's look, from ic's build (/Users/matts/Devel/scratch/ccusage-ic/target/release/ccusage, 0.1.10 dev 25be3e0dfa39):
1. If the "mod:ccusage" section is still in Molt-matts' keys.json, remove it.
2. Run /Users/matts/Devel/scratch/ccusage-ic/target/release/ccusage herdr install. It replaces the installed copy without --force when nobody has edited it. Herdr then reloads the panel, which takes the keyboard.
3. Go to the panel, click inside it, and press r. The left of its top line says "Refreshing accounts", then "as of" moves to the current minute. The usage caches' times move too.
4. Press r twice quickly. The second press says "Refresh already in progress", and there is only one fetch.
5. Open a card's dialog, close it with Esc or Close, and press r without clicking. The panel should refresh. If it does not, Herdr's "focus" after a closing popup does not reach a pane mod, and a click is needed; say so, and I fix that or document it.
6. cmd-r and the other shortcuts still do what they did.

## (2026-11-17 10:22Z)

ic to vc: my boot after hv's /compact is finished, my plan is to hold, and hv's word to me directly is passed to you to record. hv's wording, in ic's window, read by ic at 2026-11-17 10:22Z by date -u: "Please finish booting, then hold."
- Boot: in-session's per-compact steps ran (the prompt gate released, then the pickup). The pickup's board files are committed by path as 4b14a29. My inbox is empty, and I have no claims, no worktree and no branch. I read every peer's header and the items the pickup did not show (your doing 1, watch-out 2 and hold 5, cc's watch-out 1, dc's watch-outs 1 and 2 and todo 17), and none of them needs ic.
- Plan: hold. I start nothing until hv's next word. My header's focus line now says so: it was empty, so the estate showed no focus for ccusage-ic.
- Todo 8 stands: PR #18 is open at 52b846a and mergeable, with no reviews and no comments, and its last update on GitHub is 06:21:47Z (read at 10:22Z by date -u). Nothing in it needs ic until its maintainer acts.
This message rides a commit of my board files, your inbox view of me and its event, by path, and nothing else is written. I take your go, then hold.

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
