// ccusage: each Claude account's weekly use, the account `ccusage which` says to work
// on next, and what `ccusage predict` projects will go unused at each reset.
//
// ccusage decides all of it. This panel runs `ccusage which --json` and
// `ccusage predict --json` and lays out what they return, so its next marker and
// every reason are the CLI's own and cannot drift from it. It runs ccusage and
// nothing else. It asks on a timer no shorter than ccusage's usage cache lifetime,
// which `ccusage herdr install` writes into lifetime.js, so however many panels are
// open, their timers ask Claude's usage endpoint no more often than its cache allows.
// The panel takes the keyboard when it opens and when it is clicked, as Herdr's own mods
// do, and r then forces a fetch, as the dashboard's r does:
// `ccusage which --json --refresh`. Countdowns tick from the clock and ask nothing.
//
// Opened as a tab of its own by `ccusage herdr launch`, or as a pane:
// herdr-app do pane.split '{"direction":"right","mod":"ccusage"}'.
// props.every sets the refresh in seconds, longer than the lifetime but never shorter.

import { View, div } from "gpui-kit";
import { v_flex, h_flex } from "gpui-base";
import { exec, theme, props, settings, openOverlay, closeOverlay, nextCommand, run } from "herdr";
import { LIFETIME } from "./lifetime.js";

const SECOND = 1000;
const MINUTE = 60 * SECOND;

/**
 * The panel's one scale. Every text size, bar, space, corner and the card's dialog below
 * is a base size times SCALE, so they grow together: 1.5 takes the base 11, 12 and 14px
 * text to 17, 18 and 21px, and a bar's 5px to 8px.
 */
const SCALE = 1.5;

/** A base size in pixels, at the panel's scale, to the whole pixel. */
const scaled = (base) => Math.round(base * SCALE);

/** A countdown to the RFC 3339 time `at`, in ccusage's form: <1m, 45m, 3h05, 2d04h. */
function countdown(at) {
  const minutes = Math.max(0, Math.floor((Date.parse(at) - Date.now()) / MINUTE));
  const pad = (n) => String(n).padStart(2, "0");
  if (minutes < 1) return "<1m";
  if (minutes < 60) return `${minutes}m`;
  if (minutes < 1440) return `${Math.floor(minutes / 60)}h${pad(minutes % 60)}`;
  return `${Math.floor(minutes / 1440)}d${pad(Math.floor((minutes % 1440) / 60))}h`;
}

/** A span of hours in ccusage's form. */
const hours = (count) => countdown(new Date(Date.now() + count * 60 * MINUTE).toISOString());

/** A percent to the whole number, as ccusage shows one. */
const whole = (percent) => `${Math.round(percent)}%`;

/**
 * Runs ccusage with `args`: `{ json }` with what it printed, or `{ problem }` with why
 * there is none. which exits 1 when nothing is eligible, with its JSON on stdout all the
 * same, so stdout is read whatever the exit code.
 */
async function ask(args) {
  try {
    const result = await exec("ccusage", args, { timeout: 2 * MINUTE });
    if (result.stdout.trim()) return { json: JSON.parse(result.stdout) };
    return { problem: result.stderr.trim() || `ccusage ${args[0]} exited ${result.code}` };
  } catch (error) {
    return { problem: `could not run ccusage: ${error}` };
  }
}

/**
 * The forced fetch in flight, if any. Copies share this module, so a press that reaches
 * two shown panels fetches once, and each lays out the same answer.
 */
let forcing = null;

/**
 * which, with every account fetched at once, though not one in a rate-limit cooldown,
 * as the dashboard's r does; the fetch in flight when there is one.
 */
function forced() {
  forcing ??= ask(["which", "--json", "--refresh"]).finally(() => {
    forcing = null;
  });
  return forcing;
}

/**
 * A bar `percent` full, in `color`. Its fill and the rest of its track have no content,
 * and an h_flex centers its children on the cross axis, so each takes the track's height
 * only through h_full(); without it each is 0px tall and the bar draws no fill.
 */
function bar(id, percent, color, t) {
  const used = Math.min(100, Math.max(0, percent));
  return h_flex()
    .id(id)
    .h(scaled(5))
    .rounded(scaled(3))
    .overflow_hidden()
    .bg(t.raised)
    .child(div().h_full().flex_grow(used).bg(color))
    .child(div().h_full().flex_grow(100 - used));
}

function button(id, label, t, onClick) {
  return h_flex()
    .id(id)
    .h(scaled(26))
    .px(scaled(10))
    .items_center()
    .rounded(scaled(6))
    .border_1()
    .border_color(t.input)
    .bg(t.raised)
    .text_color(t.foreground)
    .cursor_pointer()
    .hover((s) => s.opacity(0.88))
    .on_click(onClick)
    .child(label);
}

export default class Ccusage extends View {
  init(_props, cx) {
    this.which = null; // ccusage which --json
    this.predict = null; // ccusage predict --json
    this.problem = null; // why the last ask failed
    this.pending = null; // the account a confirm dialog may switch to
    this.text = null; // a detail dialog's verbose usage
    this.notice = null; // what ccusage use said
    this.refreshing = null; // what the panel says while a forced fetch runs
    this.again = false; // an ask from the cache that waits for the forced fetch
    cx.spawn(async (cx) => {
      for (;;) {
        await this.command(await nextCommand(), cx);
        cx.notify();
      }
    });
    if (props()?.account) {
      this.detail(cx);
      return;
    }
    // A key reaches the panel only while it holds the keyboard, through a handle tracked
    // on its root, as in Herdr's own mods: taken when the panel opens, when it is
    // clicked, and on "focus", which pane.go and a closing popup send.
    this.handle = cx.focus_handle();
    this.handle.focus();
    const every = Math.max(LIFETIME, Number(props()?.every) || LIFETIME) * SECOND;
    cx.timer.every(every, (cx) => this.refresh(cx));
    cx.timer.every(MINUTE, (cx) => cx.notify());
    this.refresh(cx);
  }

  /**
   * Asks ccusage again: from its cache, or, when `force` is set, with every account
   * fetched first, then predict from the fresh cache. While a forced fetch runs, a second
   * press starts no other, as in the dashboard, and an ask from the cache waits until it
   * answers, then runs, so a switch made meanwhile still shows.
   */
  refresh(cx, force = false) {
    if (this.refreshing) {
      if (force) this.refreshing = "Refresh already in progress";
      else this.again = true;
      return;
    }
    if (force) this.refreshing = "Refreshing accounts";
    cx.spawn(async (cx) => {
      const which = await (force ? forced() : ask(["which", "--json"]));
      const predict = await ask(["predict", "--json"]);
      this.problem = which.problem ?? predict.problem ?? null;
      if (which.json) this.which = which.json;
      if (predict.json) this.predict = predict.json;
      this.refreshing = null;
      cx.notify();
      if (this.again) {
        this.again = false;
        this.refresh(cx);
      }
    });
  }

  /** r, while the panel holds the keyboard, forces a fetch; every other key passes on. */
  key(event, cx) {
    if (event.keystroke !== "r") return;
    this.refresh(cx, true);
    cx.stop_propagation();
    cx.notify();
  }

  /**
   * The commands Herdr delivers to this mod's shown copies: refresh, which forces a fetch
   * as r does, for run() or a key binding of one's own; reload, which a switch sends and
   * which asks from the cache; focus, which pane.go sends and Herdr sends again when a
   * popup over the panel closes; and use, once confirmed.
   */
  async command(name, cx) {
    if ((name === "refresh" || name === "reload") && !props()?.account) {
      this.refresh(cx, name === "refresh");
    }
    if (name === "focus") this.handle?.focus();
    if (name === "use" && this.pending) {
      const account = this.pending;
      this.pending = null;
      try {
        const result = await exec("ccusage", ["use", account], { timeout: MINUTE });
        this.notice = (result.stdout.trim() || result.stderr.trim()).split("\n")[0];
      } catch (error) {
        this.notice = `could not run ccusage: ${error}`;
      }
      // A switch needs no fetch: every shown panel asks again from the cache.
      run("ccusage.reload");
    }
  }

  detail(cx) {
    cx.spawn(async (cx) => {
      try {
        const result = await exec("ccusage", ["usage", props().account, "--verbose"], {
          timeout: 2 * MINUTE,
        });
        this.text = result.stdout.trimEnd() || result.stderr.trim();
      } catch (error) {
        this.text = `could not run ccusage: ${error}`;
      }
      cx.notify();
    });
  }

  render(cx) {
    const t = theme();
    if (props()?.account) return this.renderDetail(t);
    const which = this.which;
    const root = v_flex()
      .id("ccusage")
      .track_focus(this.handle)
      .on_mouse_down("left", () => this.handle.focus())
      .on_key_down((event, cx) => this.key(event, cx))
      .size_full()
      .bg(t.surface)
      .p(scaled(10))
      .gap(scaled(8))
      .text_size(scaled(12))
      .text_color(t.foreground)
      .overflow_y_scrollbar();
    if (!which) {
      return root.child(div().text_color(t.muted).child(this.problem ?? "Asking ccusage…"));
    }
    const stamp = new Date(which.now);
    const pad = (n) => String(n).padStart(2, "0");
    return root
      .child(
        // Herdr's pane header names the mod, so the panel gives only when ccusage answered,
        // and on the left whether a forced fetch is running.
        h_flex()
          .child(
            div()
              .flex_1()
              .text_size(scaled(11))
              .text_color(t.muted)
              .when(this.refreshing, (el) => el.child(this.refreshing)),
          )
          .child(
            div()
              .text_size(scaled(11))
              .text_color(t.dim)
              .child(`as of ${pad(stamp.getHours())}:${pad(stamp.getMinutes())}`),
          ),
      )
      .when(!which.choice, (el) => el.child(this.nothingEligible(which, t)))
      .children(which.accounts.map((account) => this.account(account, t)))
      .when(this.predict, (el) => el.child(this.forecast(t)))
      .when(this.problem, (el) => el.child(div().text_size(scaled(11)).text_color(t.blocked).child(this.problem)));
  }

  nothingEligible(which, t) {
    const soonest = which.soonest
      ? `; the soonest weekly reset is ${which.soonest.name}'s, in ${countdown(which.soonest.resets_at)}`
      : "";
    return div().text_size(scaled(11)).text_color(t.blocked).child(`Nothing eligible${soonest}`);
  }

  account(account, t) {
    const which = this.which;
    const projected = this.predict?.accounts?.find((entry) => entry.name === account.name);
    const tags = [];
    if (account.name === which.active) tags.push(["active", t.muted]);
    if (account.state === "chosen") tags.push(["next", t.accent]);
    const color =
      account.state === "at_cap" ? t.blocked : account.state === "unknown" ? t.dim : t.accent;
    const parts = [];
    if (account.resets_at) parts.push(`resets in ${countdown(account.resets_at)}`);
    for (const scoped of account.scoped) {
      parts.push(`${scoped.model ?? "Weekly scoped"} ${whole(scoped.percent)}`);
    }
    if (account.session) parts.push(`session ${whole(account.session.percent)}`);
    if (projected) parts.push(`${whole(projected.unused)} unused at reset`);
    // A reason that is the report's own error is said once, by the error line under this one.
    const reason = account.reason === account.error ? "" : account.reason;
    if (account.state === "unknown") parts.push(reason ? `unknown: ${reason}` : "unknown");
    else if (reason) parts.push(reason);
    return v_flex()
      .id(`account-${account.name}`)
      .gap(scaled(3))
      .p(scaled(6))
      .rounded(scaled(6))
      .cursor_pointer()
      .hover((s) => s.bg(t.raised))
      .on_click(() =>
        openOverlay({
          mod: "ccusage",
          kind: "dialog",
          width: scaled(640),
          height: scaled(520),
          props: { account: account.name },
        }),
      )
      .child(
        h_flex()
          .gap(scaled(6))
          .items_center()
          .child(div().font_semibold().truncate().child(account.name))
          .children(tags.map(([tag, tint]) => div().text_size(scaled(11)).text_color(tint).child(tag)))
          .child(div().flex_1())
          .when(account.percent !== null, (el) =>
            el.child(div().text_color(t.muted).child(whole(account.percent))),
          ),
      )
      .when(account.percent !== null, (el) =>
        el.child(bar(`bar-${account.name}`, account.percent, color, t)),
      )
      .child(div().text_size(scaled(11)).text_color(t.muted).truncate().child(parts.join(" · ")))
      .when(account.error, (el) =>
        el.child(div().text_size(scaled(11)).text_color(t.blocked).truncate().child(account.error)),
      );
  }

  forecast(t) {
    const predict = this.predict;
    const rate = predict.rate;
    const rests = `${rate.samples} samples over ${hours(rate.hours)}`;
    const text =
      rate.percent_per_hour === null
        ? `Too little history to project yet: ${rests}.`
        : `Burn rate ${rate.percent_per_hour}% of a weekly allowance an hour, from ${rests}. ` +
          `Projected unused at the current resets: ${whole(predict.unused)} of one weekly allowance.`;
    return div().text_size(scaled(11)).text_color(t.muted).child(text);
  }

  renderDetail(t) {
    const account = props().account;
    const lines = (this.text ?? "Asking ccusage…").split("\n");
    return v_flex()
      .id("ccusage-detail")
      .size_full()
      .p(scaled(14))
      .gap(scaled(10))
      .rounded(scaled(8))
      .bg(t.popover)
      .text_size(scaled(12))
      .text_color(t.foreground)
      .child(
        h_flex()
          .items_center()
          .gap(scaled(8))
          .child(div().flex_1().text_size(scaled(14)).font_semibold().truncate().child(account))
          .child(
            button("use", `Use ${account}`, t, () => {
              this.pending = account;
              // Herdr's own confirm dialog draws its text at its own size, so its box keeps its own.
              openOverlay({
                mod: "confirm",
                kind: "dialog",
                width: 420,
                height: 170,
                props: {
                  title: `Use ${account}?`,
                  detail: `Runs ccusage use ${account}, which says what it changed.`,
                  command: { name: "ccusage.use" },
                },
              });
            }),
          )
          .child(button("close", "Close", t, () => closeOverlay(props().overlay))),
      )
      .when(this.notice, (el) => el.child(div().text_color(t.accent).child(this.notice)))
      .child(
        v_flex()
          .id("ccusage-detail-text")
          .flex_1()
          .min_h(0)
          .overflow_y_scrollbar()
          .font_family(settings().font_family ?? "JetBrains Mono")
          .text_size(scaled(11))
          .children(lines.map((line, index) => div().id(`line-${index}`).whitespace_nowrap().child(line || " "))),
      );
  }
}
