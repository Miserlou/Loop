class Loop < Formula
  desc "UNIX's missing `loop` command"
  homepage "https://github.com/Miserlou/Loop"
  url "https://github.com/Miserlou/Loop/archive/master.zip"
  sha256 "ae3faebac27fe4fd1f14de8f348e3df410f0b8fd02deea54ef4abcd1f7e08648"
  head "https://github.com/Miserlou/Loop.git"
  version "0.1.0"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--root", prefix
  end

  test do
    system "#{bin}/loop", "-V"
  end
end
