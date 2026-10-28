cask "trackswitch" do
  version "0.1.0"
  sha256 "REPLACE_WITH_SHA256_AFTER_TAURI_BUILD"

  url "https://github.com/qunm00/TrackSwitch/releases/download/v#{version}/TrackSwitch.app.zip",
      verified: "github.com/qunm00/TrackSwitch/"

  name "TrackSwitch"
  desc "Move an Apple Magic Trackpad between two Macs with Bluetooth re-pairing"
  homepage "https://github.com/qunm00/TrackSwitch"

  depends_on formula: "blueutil"

  app "TrackSwitch.app"

  zap trash: [
    "~/Library/LaunchAgents/com.qunm00.trackswitch.plist",
    "~/.config/trackswitch",
  ]
end
