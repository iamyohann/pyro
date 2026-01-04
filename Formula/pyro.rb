class Pyro < Formula
  desc "The Pyro programming language and CLI"
  homepage "https://github.com/iamyohann/pyro"
  version "0.0.0"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/iamyohann/pyro/releases/download/v0.0.0/pyro-v0.0.0-x86_64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    if Hardware::CPU.arm?
      url "https://github.com/iamyohann/pyro/releases/download/v0.0.0/pyro-v0.0.0-aarch64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    url "https://github.com/iamyohann/pyro/releases/download/v0.0.0/pyro-v0.0.0-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  end

  def install
    bin.install "pyro"
  end
end
