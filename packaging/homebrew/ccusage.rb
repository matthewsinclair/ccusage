# packaging/homebrew/ccusage.rb: the template of Ccusage's Homebrew formula, which
# bin/devbin formula render fills from a release: @@VERSION@@, each Mac archive's
# @@SHA256:...@@ from the release's SHA256SUMS, @@REVISION@@ (the tag's commit, which
# Linux builds from), and @@INSTALL@@ (the archive into libexec, 0755 on ccusage, the
# manifest and the link in bin). This header is dropped from the render. Edit this
# file, never a rendered one.
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
    if OS.mac?
      @@INSTALL@@
      verify_macos_signature
    else
      system "cargo", "install", *std_cargo_args
      pkgshare.install "design/icons/ccusage-logo.svg"
    end
  end

  def verify_macos_signature
    requirement = '=identifier "com.geodica.ccusage" and anchor apple generic ' \
                  'and certificate leaf[subject.OU] = "76BQL8L47U" ' \
                  "and certificate leaf[field.1.2.840.113635.100.6.1.13] exists"
    system "/usr/bin/codesign", "--verify", "--strict", "-R", requirement, libexec/"ccusage"
  end

  def caveats
    <<~EOS
      Install Claude Code separately and make sure claude is on PATH.
      Check your environment with: ccusage doctor
      Set up accounts with: ccusage init
      Existing credentials and XDG configuration are preserved on upgrades.
      macOS may request Keychain access once per account when moving to signed releases.
    EOS
  end

  test do
    verify_macos_signature if OS.mac?
    assert_match "ccusage #{version}", shell_output("#{bin}/ccusage --version")
    assert_match "PLAN USAGE", shell_output("#{bin}/ccusage usage --demo")
    ENV["XDG_CONFIG_HOME"] = (testpath/"config").to_s
    system bin/"ccusage", "init"
    system bin/"ccusage", "add", "example"
    assert_path_exists testpath/"config/ccusage/config.toml"
  end
end
