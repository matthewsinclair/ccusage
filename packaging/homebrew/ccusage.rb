# Canonical template. Render using tools/formula render TAG OUTPUT.
# The tag and its commit are resolved together; never fill these in by hand.
class Ccusage < Formula
  desc "Named Claude Code accounts and a terminal usage dashboard"
  homepage "https://github.com/matthewsinclair/ccusage"
  url "https://github.com/matthewsinclair/ccusage.git",
      tag:      "@RELEASE_TAG@",
      revision: "@RELEASE_REVISION@"
  license "MIT"
  head "https://github.com/matthewsinclair/ccusage.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
    pkgshare.install "design/icons/ccusage-logo.svg"
  end

  def caveats
    <<~EOS
      Install Claude Code separately and make sure claude is on PATH.
      Check your environment with: ccusage doctor
      Set up accounts with: ccusage init
      Existing credentials and XDG configuration are preserved on upgrades.
    EOS
  end

  test do
    assert_match "ccusage #{version}", shell_output("#{bin}/ccusage --version")
    assert_match "PLAN USAGE", shell_output("#{bin}/ccusage usage --demo")
    ENV["XDG_CONFIG_HOME"] = (testpath/"config").to_s
    system bin/"ccusage", "init"
    system bin/"ccusage", "add", "example"
    assert_path_exists testpath/"config/ccusage/config.toml"
  end
end
