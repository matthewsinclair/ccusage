// ccusage: each Claude account's weekly use, the account `ccusage which` says to work
// on next, and what `ccusage predict` projects will go unused at each reset.
//
// ccusage decides all of it. This panel runs `ccusage which --json` and
// `ccusage predict --json` and lays out what they return, so its next marker and
// every reason are the CLI's own and cannot drift from it. It runs ccusage and
// nothing else. It asks on a timer no shorter than ccusage's usage cache lifetime,
// which `ccusage herdr install` writes into lifetime.js, so however many panels are
// open, ccusage asks Claude's usage endpoint no more often than its cache allows.
// Countdowns tick from the clock and ask nothing.
//
// Opened as a pane: herdr-app do pane.split '{"direction":"right","mod":"ccusage"}'.
// props.every sets the refresh in seconds, longer than the lifetime but never shorter.

import { View, div } from "gpui-kit";
import { v_flex, h_flex } from "gpui-base";
import { exec, theme, props, settings, openOverlay, closeOverlay, nextCommand, run } from "herdr";
import { LIFETIME } from "./lifetime.js";

const SECOND = 1000;
const MINUTE = 60 * SECOND;

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

/** A bar `percent` full, in `color`. */
function bar(id, percent, color, t) {
  const used = Math.min(100, Math.max(0, percent));
  return h_flex()
    .id(id)
    .h(5)
    .rounded(3)
    .overflow_hidden()
    .bg(t.raised)
    .child(div().flex_grow(used).bg(color))
    .child(div().flex_grow(100 - used));
}

function button(id, label, t, onClick) {
  return h_flex()
    .id(id)
    .h(26)
    .px(10)
    .items_center()
    .rounded(6)
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
    const every = Math.max(LIFETIME, Number(props()?.every) || LIFETIME) * SECOND;
    cx.timer.every(every, (cx) => this.refresh(cx));
    cx.timer.every(MINUTE, (cx) => cx.notify());
    this.refresh(cx);
  }

  /** Runs ccusage with `args`: its JSON, or null with the reason kept. */
  async ask(args) {
    try {
      const result = await exec("ccusage", args, { timeout: 2 * MINUTE });
      // which exits 1 when nothing is eligible, with its JSON on stdout all the same.
      if (result.stdout.trim()) return JSON.parse(result.stdout);
      this.problem = result.stderr.trim() || `ccusage ${args[0]} exited ${result.code}`;
    } catch (error) {
      this.problem = `could not run ccusage: ${error}`;
    }
    return null;
  }

  refresh(cx) {
    cx.spawn(async (cx) => {
      this.problem = null;
      const which = await this.ask(["which", "--json"]);
      const predict = await this.ask(["predict", "--json"]);
      if (which) this.which = which;
      if (predict) this.predict = predict;
      cx.notify();
    });
  }

  /** The commands Herdr delivers to this mod: refresh, and use once confirmed. */
  async command(name, cx) {
    if (name === "refresh" && !props()?.account) {
      this.refresh(cx);
    }
    if (name === "use" && this.pending) {
      const account = this.pending;
      this.pending = null;
      try {
        const result = await exec("ccusage", ["use", account], { timeout: MINUTE });
        this.notice = (result.stdout.trim() || result.stderr.trim()).split("\n")[0];
      } catch (error) {
        this.notice = `could not run ccusage: ${error}`;
      }
      run("ccusage.refresh");
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
      .size_full()
      .bg(t.surface)
      .p(10)
      .gap(8)
      .text_size(12)
      .text_color(t.foreground)
      .overflow_y_scrollbar();
    if (!which) {
      return root.child(div().text_color(t.muted).child(this.problem ?? "Asking ccusage…"));
    }
    const stamp = new Date(which.now);
    const pad = (n) => String(n).padStart(2, "0");
    return root
      .child(
        h_flex()
          .items_center()
          .child(div().flex_1().font_semibold().child("ccusage"))
          .child(
            div()
              .text_size(11)
              .text_color(t.dim)
              .child(`as of ${pad(stamp.getHours())}:${pad(stamp.getMinutes())}`),
          ),
      )
      .when(!which.choice, (el) => el.child(this.nothingEligible(which, t)))
      .children(which.accounts.map((account) => this.account(account, t)))
      .when(this.predict, (el) => el.child(this.forecast(t)))
      .when(this.problem, (el) => el.child(div().text_size(11).text_color(t.blocked).child(this.problem)));
  }

  nothingEligible(which, t) {
    const soonest = which.soonest
      ? `; the soonest weekly reset is ${which.soonest.name}'s, in ${countdown(which.soonest.resets_at)}`
      : "";
    return div().text_size(11).text_color(t.blocked).child(`Nothing eligible${soonest}`);
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
    parts.push(account.state === "unknown" ? `unknown: ${account.reason}` : account.reason);
    return v_flex()
      .id(`account-${account.name}`)
      .gap(3)
      .p(6)
      .rounded(6)
      .cursor_pointer()
      .hover((s) => s.bg(t.raised))
      .on_click(() =>
        openOverlay({
          mod: "ccusage",
          kind: "dialog",
          width: 640,
          height: 520,
          props: { account: account.name },
        }),
      )
      .child(
        h_flex()
          .gap(6)
          .items_center()
          .child(div().font_semibold().truncate().child(account.name))
          .children(tags.map(([tag, tint]) => div().text_size(11).text_color(tint).child(tag)))
          .child(div().flex_1())
          .when(account.percent !== null, (el) =>
            el.child(div().text_color(t.muted).child(whole(account.percent))),
          ),
      )
      .when(account.percent !== null, (el) =>
        el.child(bar(`bar-${account.name}`, account.percent, color, t)),
      )
      .child(div().text_size(11).text_color(t.muted).truncate().child(parts.join(" · ")))
      .when(account.error, (el) =>
        el.child(div().text_size(11).text_color(t.blocked).truncate().child(account.error)),
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
    return div().text_size(11).text_color(t.muted).child(text);
  }

  renderDetail(t) {
    const account = props().account;
    const lines = (this.text ?? "Asking ccusage…").split("\n");
    return v_flex()
      .id("ccusage-detail")
      .size_full()
      .p(14)
      .gap(10)
      .rounded(8)
      .bg(t.popover)
      .text_size(12)
      .text_color(t.foreground)
      .child(
        h_flex()
          .items_center()
          .gap(8)
          .child(div().flex_1().text_size(14).font_semibold().truncate().child(account))
          .child(
            button("use", `Use ${account}`, t, () => {
              this.pending = account;
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
          .text_size(11)
          .children(lines.map((line, index) => div().id(`line-${index}`).whitespace_nowrap().child(line || " "))),
      );
  }
}
