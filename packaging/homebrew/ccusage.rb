# packaging/homebrew/ccusage.rb: the template of Ccusage's Homebrew formula, which
# bin/devbin formula render fills from a release: @@VERSION@@, each Mac archive's
# @@SHA256:...@@ from the release's SHA256SUMS, @@REVISION@@ (the tag's commit, which
# Linux builds from), and @@INSTALL@@, devbin's one install body (devbin#0326): on
# macOS the signed archive laid out in libexec with 0755 on bin/ccusage and the
# manifest, on Linux bin/devbin install from the tag's checkout, which builds and
# writes the same tree, then the link in bin. This header is dropped from the
# render. Edit this file, never a rendered one.
#
# The codesign requirement repeats packaging/macos/signing's, which a Ruby formula
# cannot read; tests/macos.bats fails when the two differ.
class Ccusage < Formula
  desc "Named Claude Code accounts and a terminal usage dashboard"
  homepage "https://github.com/matthewsinclair/ccusage"
  url "https://github.com/matthewsinclair/ccusage.git",
      tag:      "v@@VERSION@@",
      revision: "@@REVISION@@"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/matthewsinclair/ccusage/releases/download/v@@VERSION@@/ccusage-v@@VERSION@@-aarch64-apple-darwin.tar.gz"
      sha256 "@@SHA256:ccusage-v{version}-aarch64-apple-darwin.tar.gz@@"
    end
    on_intel do
      url "https://github.com/matthewsinclair/ccusage/releases/download/v@@VERSION@@/ccusage-v@@VERSION@@-x86_64-apple-darwin.tar.gz"
      sha256 "@@SHA256:ccusage-v{version}-x86_64-apple-darwin.tar.gz@@"
    end
  end

  on_linux do
    depends_on "rust" => :build
  end

  def install
    @@INSTALL@@
    verify_macos_signature if OS.mac?
  end

  def verify_macos_signature
    requirement = '=identifier "com.geodica.ccusage" and anchor apple generic ' \
                  'and certificate leaf[subject.OU] = "76BQL8L47U" ' \
                  "and certificate leaf[field.1.2.840.113635.100.6.1.13] exists"
    system "/usr/bin/codesign", "--verify", "--strict", "-R", requirement, libexec/"bin/ccusage"
  end

  def caveats
    <<~EOS
      Install Claude Code separately and make sure claude is on PATH.
      Check your environment with: ccusage doctor
      Set up accounts with: ccusage init
      Existing credentials and XDG configuration are preserved on upgrades.

      A maintainer with a dev checkout or an opt install of ccusage chooses
      between them and this install with bin/devbin use, run in the checkout
      (bin/devbin use brew returns to this one).
    EOS
  end

  test do
    verify_macos_signature if OS.mac?
    assert_match "ccusage #{version} (brew, ", shell_output("#{bin}/ccusage --version")
    assert_match "PLAN USAGE", shell_output("#{bin}/ccusage usage --demo")
    ENV["XDG_CONFIG_HOME"] = (testpath/"config").to_s
    system bin/"ccusage", "init"
    system bin/"ccusage", "add", "example"
    assert_path_exists testpath/"config/ccusage/config.toml"
  end
end
