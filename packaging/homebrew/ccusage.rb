# Canonical template. Render using tools/formula render TAG OUTPUT.
# The tag and its commit are resolved together; never fill these in by hand.
class Ccusage < Formula
  desc "Named Claude Code accounts and a terminal usage dashboard"
  homepage "https://github.com/matthewsinclair/ccusage"
  version "@RELEASE_VERSION@"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/matthewsinclair/ccusage/releases/download/@RELEASE_TAG@/ccusage-@RELEASE_TAG@-aarch64-apple-darwin.tar.gz"
      sha256 "@RELEASE_ARM_SHA256@"
    end
    on_intel do
      url "https://github.com/matthewsinclair/ccusage/releases/download/@RELEASE_TAG@/ccusage-@RELEASE_TAG@-x86_64-apple-darwin.tar.gz"
      sha256 "@RELEASE_INTEL_SHA256@"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/matthewsinclair/ccusage.git",
          tag:      "@RELEASE_TAG@",
          revision: "@RELEASE_REVISION@"
    end
    on_intel do
      url "https://github.com/matthewsinclair/ccusage.git",
          tag:      "@RELEASE_TAG@",
          revision: "@RELEASE_REVISION@"
    end
    depends_on "rust" => :build
  end

  def install
    if OS.mac?
      bin.install "ccusage"
      verify_macos_signature
      pkgshare.install "ccusage-logo.svg"
    else
      system "cargo", "install", *std_cargo_args
      pkgshare.install "design/icons/ccusage-logo.svg"
    end
  end

  def verify_macos_signature
    requirement = '=identifier "com.geodica.ccusage" and anchor apple generic ' \
                  'and certificate leaf[subject.OU] = "76BQL8L47U" ' \
                  "and certificate leaf[field.1.2.840.113635.100.6.1.13] exists"
    system "/usr/bin/codesign", "--verify", "--strict", "-R", requirement, bin/"ccusage"
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
    if OS.mac?
      verify_macos_signature
    end
    assert_match "ccusage #{version}", shell_output("#{bin}/ccusage --version")
    assert_match "PLAN USAGE", shell_output("#{bin}/ccusage usage --demo")
    ENV["XDG_CONFIG_HOME"] = (testpath/"config").to_s
    system bin/"ccusage", "init"
    system bin/"ccusage", "add", "example"
    assert_path_exists testpath/"config/ccusage/config.toml"
  end
end
