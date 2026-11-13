# ccusage

A panel of your Claude accounts' weekly plan use: each account's bar and reset, which one is active, which one `ccusage which` says to work on next and why, and how much of each weekly allowance `ccusage predict` projects will go unused. Everything it shows is ccusage's own answer, laid out natively.

![ccusage](preview.png)

## What you see

- A heading, `ccusage`, with the time ccusage answered (`as of 14:05`), in 11px dim text on the right.
- When nothing is eligible, one line in the blocked color: `Nothing eligible; the soonest weekly reset is NAME's, in 2d04h`.
- One card per account, in `which`'s order: the chosen account first, then the eligible ones and those at the cap by rank, then the unknown ones by name. Each card is the account's name in semibold with its tags beside it (`active` in muted, `next` in the accent color), its weekly percent on the right, a 5px bar filled to that percent (accent; the blocked color at the cap; dim when unknown), and one 11px line: `resets in 3d04h · Fable 10% · 28% unused at reset · soonest reset with headroom`. An unknown account's line ends with `unknown:` and ccusage's reason, and a report's own error shows under it in the blocked color.
- Under the cards, predict's line: `Burn rate 0.6% of a weekly allowance an hour, from 48 samples over 1d07h. Projected unused at the current resets: 97% of one weekly allowance.`, or `Too little history to project yet: ...`.
- It fills its block and scrolls, from a narrow column to a full pane; long lines truncate.

## How it behaves

- It asks ccusage on a timer whose period is ccusage's usage cache lifetime, `LIFETIME` in `lifetime.js`, or longer when `props.every` (seconds) says so; never shorter. ccusage answers from its cache and fetches only what is older than that lifetime, so however many panels, TUIs and scripts are open, each account is fetched at most once a lifetime, and a rate-limited account waits out its cooldown for all of them.
- Countdowns tick from the clock once a minute and ask nothing.
- A click on a card opens a 640 x 520 dialog with that account's full `ccusage usage NAME --verbose` text in the terminal font, a `Use NAME` button and `Close`. `Use NAME` asks first, through Herdr's `confirm` dialog; confirmed, it runs `ccusage use NAME`, shows the line ccusage printed, and has every open panel ask ccusage again.
- Esc closes the dialog.

## Built from

- One mod, `ccusage`, with `"place": "none"`: it docks nowhere until opened, eg as a pane with `herdr-app do pane.split '{"direction":"right","mod":"ccusage"}'`.
- It runs one program, `ccusage`: `which --json` and `predict --json` on its timer, `usage NAME --verbose` for a card's dialog, and `use NAME` when a switch is confirmed. It reaches no network itself and reads no file outside its folder.
- `exec`, `theme`, `props`, `settings`, `openOverlay` (itself as a dialog with `props.account`, and the shipped `confirm` with `command: { name: "ccusage.use" }`), `closeOverlay`, `nextCommand` (`use`, `refresh`) and `run("ccusage.refresh")`.
- `lifetime.js` is written by `ccusage herdr install` from ccusage's own cache lifetime, beside `main.js` and `mod.json`, which ccusage embeds when it is built.

## Watch out

- No rule runs here. The next marker, the ranks, the reasons and the projection are ccusage's; the panel only lays them out, so it cannot drift from the CLI.
- `which` exits 1 when nothing is eligible, with its JSON on stdout all the same; read stdout whatever the exit code.
- `nextCommand` carries a name and no arguments, so the dialog keeps the account it may switch to and acts on `use` only when it has one; every other open copy ignores it.
- Herdr keeps a hidden copy's timers running, so the period bounds those too.
- Install it with `ccusage herdr install`, which replaces only its own unedited copy and runs `herdr-app check`; an edited copy takes `--force`.

## Done when

- `herdr-app check` passes on the installed folder.
- Opened as a pane, it shows each account's bar, reset, tags and reason as `ccusage which` gives them, and predict's line.
- A card's dialog shows the same text as `ccusage usage NAME --verbose`, and a confirmed `Use NAME` makes it active, as `ccusage which` then says.
