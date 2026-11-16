# inbox: cc -> vc

## (2026-11-16 17:03Z)

cc's plan for this session, for your go. Pickup: no live messages, ST0003 claimed, nothing of mine in flight; the pickup is committed as 87c01e3, and I write nothing more until you say go.

(1) Todo 4, your two nits on ST0003/WP04 (your todo 8), as one follow-up unit in a worktree, with WP04 left closed. (a) sessions::parse reads each variable from the last NAME=value on the ps line, not the first: parse hands variable() the words in reverse, so the environment, which ps -E prints after the arguments, wins over an argument that looks like one. from_proc keeps the first match, as getenv does. AT-04.3's test gains a line where an argument and the environment both name a variable, and the row's text gains that clause. (b) usage::verified_token_at refuses Unverified before auth::fresh and the profile call when the profile has neither an email nor an account record, so it spends no renewal on a login it cannot verify; auth::read stays first, so a missing login still says so. A new row, AT-03.11 under AC-03.1, with its test in usage.rs: such a profile, a login an hour from expiry and the profile endpoint at a port nothing answers, as AT-04.10's test has it; red today as a connection error, green as Unverified.

One Desk slot on the cargo lane, asked on your word: the two tests at the unfixed tree for the red, then bin/devbin build, test, check format and clippy at the fix. I hand you the diff and that run, and record both rows from it.

(2) Todo 5, ST0003/WP06's rollout: nothing to do until hv's word; I stand by for anything it finds in the code. Watch-out 1 stands.

If you would rather the rows took another shape (eg WP04 reopened), say so with the go.

## (2026-11-16 17:50Z) Re: 2026-11-16 17:19Z

cc to vc: ISSUE:0008's unit is ready for your read. Branch cc/ISSUE-0008, worktree /Users/matts/Devel/scratch/ccusage-cc/wt-issue-0008, base 8a8e9b5: 6f077cd the tests alone, 23b177f the fix, 47aa80b the AT rows (src, tests and Cargo files identical to 23b177f, checked by git diff). Nothing pushed.

The run is Desk slot 01M7JQCS4KTNMQVRNWBQQ3KTGM, ended by its release. Red at 6f077cd: bin/devbin test rust, and the seven new tests failed as meant (AT-04.3's argument case read /argument; AT-02.11 got the profile endpoint's connection error; predict's rate 0.5 against 1.5; use went ahead with a session pinned; no shell note; no restart in doctor's remedy; no INFO for an unchecked own login), every other test passing. Green at 23b177f: bin/devbin build, bin/devbin test rust (99 lib and 53 cli passed, 0 failed, 2 ignored), check format and check clippy, both seals empty. Throwaway HOME and XDG trees only, with the ps and lsof stand-ins; no real Keychain item read.

What it does, item by item:
(1) doctor asks usage::whose of each profile that is not seated and whose own store holds a login: the cache first, never a renewal, and an expiring token is an INFO saying it was not checked. Another account's is a FAIL naming the pinned sessions, with the remedy. The seated profile's second login is not asked whose: the two-logins FAIL stands whatever account it is, and asking would cost a profile call on every doctor run.
(2) One remedy, sessions::remedy: "Claude session 401 reads hello's own store, so first restart it from a shell without CLAUDE_SECURESTORAGE_CONFIG_DIR (unset CLAUDE_SECURESTORAGE_CONFIG_DIR), and only then run ccusage login hello", or "no running Claude session reads hello's own store, so run ccusage login hello", or that the sessions could not be listed. doctor's two-logins, shared-empty and shared-login-is-another's FAILs, its new own-store FAIL, and usage's refusal all use it: check_identity now returns a typed Mismatch, and Client::report appends the remedy.
(3) switch::to refuses before anything moves when a running session reads the target's own store, so use, which --use, run NAME and the TUI's a key all do; finish refuses an unfinished move into such a store; ccusage login is now switch::login, which refuses for the seated profile before Claude's login runs. use --none's own move empties only the shared store, which the following sessions read; it is the back-out, so it is not refused for them, only through finish. No override flag, and I do not think one is needed: the refusal names the sessions, restarting them is the way through, and an override would bring the loop back.
(4) shared::shell_pin is the one wording. doctor's old WARN uses it (it now gives the command, and says nothing for an empty value or the shared config's own); usage, which and predict print it to stderr while a profile is seated. stdout and every --json are as they were.
(5) Each history line now records the plan its login recorded (an optional plan field; an older line reads as none), and predict counts each sample on its own plan, an older one on its account's report plan, or, when that report failed, on the latest plan its samples recorded. Every account whose weekly samples the rate leaves out is named in left_out with how many: a value change, not a shape change. On rhadamanth, engineering01's 58 samples were written by 0.1.9 with no plan, so until its next good fetch predict names them as left out instead of counting them; after the live repair's ccusage login engineering01 and a fetch, they count.
The nits: sessions::parse takes the last NAME=value, the environment's; verified_token_at refuses a profile with nothing to check against before any renewal.

AT rows: ST0003 AT-02.11 is new under AC-02.3, not AT-03.11 under AC-03.1 as my plan said, since AC-02.3 is the refusal-before-any-write criterion and a renewal writes the store; AT-04.3's note gains the argument case. ST0002 AT-05.3 and AT-05.4 note their changed tests: record takes the plan, and AT-05.4's expected reason gains the count. 29 rows re-cited from their id comments, 12 in ST0003 and 17 in ST0002, AT-04.3 in each. intent at lint ST0003 32 of 32, ST0002 27 of 27. The ISSUE:0008 tests cite its Acceptance 1 to 4 and the fix outline's item 4 in their comments.

One finding for you to place, not built: a switch to a profile whose own store holds another account's login seats that login, since switch::to verifies the outgoing login and never the incoming one. doctor's new FAIL says so; refusing it would be a small follow-up, the incoming login checked as whose checks one, before the move. Items 6 and 7 wait for your word.

## (2026-11-16 17:53Z)

cc to vc, as you asked after my hand-over, no code: how I would verify the incoming login.

Where. Beside the outgoing check, which it mirrors. switch::relocate's verify closure verifies request.from against the shared store today; it would verify request.to as well, with usage::verified_token(store, to, &places.to.dir), and pass both tokens to seat::run, whose plan refuses the seat when the incoming store's access token is not the verified one, exactly as it refuses an outgoing login today. finish verifies the incoming too while its login is still in its own store (step Begun or Parked), as it re-verifies the outgoing at Begun. reseat, for ccusage login of the seated profile, passes the fresh login as the incoming, so a browser sign-in as the wrong account is refused before it reaches the shared store; it stays in the profile's own store, where doctor's new check names whose it is. use, which --use, run NAME and the TUI's a key all reach relocate through switch::to.

Against what. Expected::of, the rule the outgoing check and doctor's whose already share: the profile's configured email, else the account its own .claude.json records. switch::to already refuses a target with no record, so a switch always has something to check against. A /login typed in a pinned session writes its record into the shared .claude.json, since that session's config directory is the shared one, so the profile's own record still names the right account and catches it even with no email. Cost: nothing when a usage fetch verified that exact token where it lives, as every fetch of that account by usage, which, the TUI or the Herdr panel does; otherwise a renewal if the token is expiring and one profile call per switch.

The refusal. A new seat::Refusal, say Foreign(NAME): "NAME's own store holds a login that could not be verified as NAME's, so it was not seated; nothing was moved; see ccusage doctor". relocate's one retry on Unverified, for a renewal between the check and the move's locks, would cover it too.

The first red. A case in AT-02.6's table in src/seat.rs, on file stores: a move from a to b whose incoming token is not b's (or none) is refused as Foreign before any write, journal or selection, and its message carries no token. Today it is a compile red, since plan takes no incoming token; with the token threaded through and no check it goes red as a behaviour, because plan seats b; then green with the check. Cases for adoption (to only, replace) and for reseat sit in the same table.

What it costs beyond that. Every plan and run call and every AT-02.x fixture passes an incoming token, which is mechanical. The CLI tests: every shared-mode use would now ask the real profile endpoint unless the incoming login is verified in its cache, so the sandbox's login_with_record would write a verified usage cache for the profile's own store, and the existing tests stay offline. Criterion: AC-02.3 lists an outgoing login that does not verify; this adds an incoming one, which is your call, or a line on ISSUE:0008 if it rides this unit. My estimate: one small unit, one Desk slot.

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
