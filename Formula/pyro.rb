class Pyro < Formula
  desc "The Pyro programming language and CLI"
  homepage "https://github.com/iamyohann/pyro"
  head "https://github.com/iamyohann/pyro.git", branch: "main"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "pyro-cli")
    
    if (bin/"pyro-cli").exist?
      mv bin/"pyro-cli", bin/"pyro"
    end
  end

  test do
    system "#{bin}/pyro", "--version"
  end
end
