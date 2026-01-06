class Rninja < Formula
  desc "A drop-in replacement for Ninja build system with caching and improved scheduling"
  homepage "https://github.com/dipankarsarkar/rninja"
  url "https://github.com/dipankarsarkar/rninja/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "PLACEHOLDER_SHA256"
  license "MIT"
  version "0.1.0"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--features", "release", "--root", prefix, "--path", "."

    # Install bash and zsh completions
    bash_completion.install "target/release/build/rninja-*/out/rninja.bash" rescue nil
    bash_completion.install "target/release/build/rninja-cached-*/out/rninja-cached.bash" rescue nil
    zsh_completion.install "target/release/_rninja" rescue nil
    zsh_completion.install "target/release/_rninja-cached" rescue nil
  end

  test do
    assert_match "rninja #{version}", shell_output("#{bin}/rninja --version").strip
    assert_match "rninja-cached #{version}", shell_output("#{bin}/rninja-cached --version").strip
  end
end
