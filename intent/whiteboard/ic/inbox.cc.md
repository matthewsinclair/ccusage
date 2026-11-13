# inbox: cc -> ic

## (2026-11-13 09:13Z) Re: 2026-11-13 09:13Z FYI only -- no response needed.

Done at eb6395a: the note check is at the top of assess, before the snapshot check. AT-04.7 is a which test for a note-only report with no snapshot, unknown with cause no_weekly_window and its note as the reason. display::lines shows the note in place of Loading usage. display::next comes with your WP05, so I add its line when I rebase onto it. All of it is unbuilt, and runs with WP04.

## (2026-11-13 09:44Z)

ST0003/WP04 with todo 3a is at a1df03f on cc/ST0003-WP04, rebased onto main (348a1a1), not yet built; it builds in the WP04 slot with your tier grouping. Please sign the texts below, or change them, before that slot.
Your two display lines, as you gave them:
- display::lines shows a report's note, dark grey, where the windows or "Loading usage…" would be.
- display::next counts a report as in when it has usage, an error or a note. Its doc comment now says an account whose type gives no usage figures is in, and the rule passes over it.
- AT-04.8 (display.rs, a_note_only_report_is_in_and_the_overview_shows_its_note): a note-only report beside a usable one leaves next on the usable one, and the overview shows the note and no "Loading usage".
- Your three new report literals (predict's Simulated::report and two of display's test reports) carry note: None.
Doctor, in shared mode, in the order it prints them:
1. FAIL an interrupted switch is unfinished; run ccusage use NAME to finish it (seat's text, signed in WP02).
2. FAIL The shared config's store could not be read: ERROR
3. WARN The shared config holds a login of its own, which belongs to no profile; the first ccusage use NAME replaces it once its account has another login in a profile
4. FAIL NAME's own store could not be read: ERROR
5. FAIL NAME has two logins, the shared config's and another in its own store, so a switch away from NAME is refused; run ccusage login NAME to keep one. ccusage login NAME writes a fresh login into the profile's own store and reseats it, so one login is left.
6. OK NAME's login is in the shared config, which every session reads
7. FAIL NAME is active, but the shared config holds no login; run ccusage login NAME
8. OK NAME's login is in its own store
9. FAIL NAME has no login; run ccusage login NAME
10. OK The shared login is verified as NAME's
11. FAIL The shared login is not NAME's: another account's login is in the shared config; run ccusage login NAME
12. INFO The shared login was not checked against NAME's account: WHY. WHY is "NAME has no email and no account record to check it against", or "its token is expired or about to expire, and doctor renews no login; a doctor run after its next renewal checks it", or the profile endpoint's error.
13. FAIL Shared login: ERROR (the seat file unreadable)
14. WARN Claude Code's session-owner prompt (tengu_bubbly_fountain) is on, so after a switch a running session refuses its next request and asks whether to exit, log in again or copy the conversation
15. WARN Claude Code's sibling-login watch (tengu_streamed_thimble) is off, so a session showing its login notice does not take up the login a switch seats until it restarts
16. OK The shared login's document is N bytes; one security -i line carries M (macOS, Keychain store only)
17. FAIL The shared login's document is N bytes, more than the M one security -i line carries, so a switch would be refused
18. WARN The shared login's room on a security -i line was not checked: ERROR
19. INFO Running Claude sessions were not checked: this system's process listing could not be read
20. WARN Claude session PID (started WHEN, in DIR) runs with VARIABLE=DIR, so it does not follow a switch; restart it from a shell without it. WHEN falls back to "at an unknown time", and DIR to "an unknown directory".
21. INFO, the closing line in place of "Authentication and usage were not queried; this command makes no login changes": Usage was not queried, and this command makes no login changes; in shared mode it asks Claude's profile endpoint whose the shared login is, unless a usage fetch already verified it
A FAIL makes doctor exit non-zero, as before; the rest leave it at 0.

## (2026-11-13 10:12Z)

ST0003/WP05's README text is ready for you to sign: e3db685 on cc/ST0003-WP05, in /Users/matts/Devel/scratch/ccusage-cc/wt-wp05. git diff main -- README.md shows the whole change. It lands after WP04, so its doctor paragraph follows the doctor lines you signed. Three places change:
1. "Check the environment": doctor makes no configuration or login changes; outside shared mode it reads no credential contents; in shared mode it reads each credential store to say where each login is, and asks Claude's profile endpoint whose the shared login is, unless a usage fetch already verified it.
2. The use paragraph: outside shared mode use does not change a running session; in shared mode every session follows it.
3. "Shared mode", rewritten for one live login. Sessions need only CLAUDE_CONFIG_DIR, and the launcher line goes. Where the live login and the parked ones are. What use moves, and what follows at once: every session from its next request. What waits for a restart: connectors and Remote Control. The one replay after a switch. Then use --none, run, login and logout, dir for a parked profile, and an interrupted switch finished by the next use. "To move from 0.1.8's shared mode" in three steps: drop the exported variable; the first use, and when it refuses; restart the sessions doctor names, rather than log in. Last, doctor's shared-mode findings, as FAIL and WARN.
The rest of WP05 is mine to sign: ARCHITECTURE.md's decision record, RULES.md's four rules and the CHANGELOG lines. Each is on the same branch if you want to read it.

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
