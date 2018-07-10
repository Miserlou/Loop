class Loop < Formula
  desc "UNIX's missing `loop` command"
  homepage "https://github.com/Miserlou/Loop"
  url "https://github.com/Miserlou/Loop/archive/master.zip"
  sha256 "56f351200bfddf72136aaf5051cd97ab04e0c42810ef312dbbd809125c4798ec"
  head "https://github.com/Miserlou/Loop.git"
  version "0.3.3"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--root", prefix
  end

  test do
    system "#{bin}/loop", "-V"
  end
end
