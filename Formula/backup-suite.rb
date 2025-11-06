class BackupSuite < Formula
  desc "Fast, secure & intelligent local backup tool"
  homepage "https://github.com/sanae-abe/backup-suite"
  url "https://github.com/sanae-abe/backup-suite/archive/refs/tags/v1.0.0.tar.gz"
  sha256 ""
  license "MIT"
  head "https://github.com/sanae-abe/backup-suite.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "backup-suite", shell_output("#{bin}/backup-suite --version")
  end
end
