class BackupSuite < Formula
  desc "Fast, secure & intelligent local backup tool with AES-256 encryption and Zstd compression"
  homepage "https://github.com/sanae-abe/backup-suite"
  url "https://github.com/sanae-abe/backup-suite/archive/refs/tags/v1.0.0.tar.gz"
  sha256 "c248fbac3d07cc392723443ed4e6dd44e58f2caae65060d15aba055323302eca"
  license "MIT"
  head "https://github.com/sanae-abe/backup-suite.git", branch: "main"

  depends_on "rust" => :build

  # Require Rust 1.70.0 or later (MSRV)
  # uses_from_macos "rust@1.70" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    # Version check
    assert_match "backup-suite 1.0.0", shell_output("#{bin}/backup-suite --version")

    # Help command check
    assert_match "Fast, secure & intelligent local backup tool", shell_output("#{bin}/backup-suite --help")

    # Status command check (should work without config)
    system bin/"backup-suite", "status"
  end
end
