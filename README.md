# TrackSwitch

TrackSwitch moves one Apple Magic Trackpad between two Macs. You click a menu bar item to switch. The app re-pairs the trackpad over Bluetooth, so the trackpad follows you from one Mac to the other.

## How it works

Each Mac runs TrackSwitch. The two Macs use different Apple accounts, so Apple's cloud pairing does not apply. A click on Mac A releases the trackpad. Mac A unpairs the trackpad and turns Bluetooth off for a moment. The trackpad is then free. When Mac B wakes, or within a few seconds when it is awake, Mac B claims the trackpad. No message passes between the two Macs. The release is the signal.

The Mac that releases yields. It will not grab the trackpad back. It re-arms only after it sees the other Mac took the trackpad.

## What you need

- Two Macs on macOS 13 or newer.
- One Apple Magic Trackpad (2 or 3).
- `blueutil`, installed by Homebrew.

## Install

The app ships as a Homebrew cask. Publish `packaging/brew/Casks/trackswitch.rb` in a `homebrew-trackswitch` tap, then users run:

```sh
brew tap qunm00/homebrew-trackswitch
brew install --cask trackswitch
```

The cask depends on the `blueutil` formula. Fill the cask's `sha256` after building the `.app` on macOS.

## First run

1. Open TrackSwitch on both Macs.
2. The setup window opens on first run.
3. Pick the Magic Trackpad from the list.
4. Type the other Mac's name, or wait for the app to find it on the network.
5. Click Save.

TrackSwitch starts at login on both Macs.

## Switch

Click the menu bar icon. Choose `Switch to <other Mac>` to hand the trackpad over. Choose `This Mac (take trackpad)` to pull it back.

## Build from source

```sh
cargo tauri build
```

Build on macOS. The unit tests run anywhere:

```sh
cd src-tauri && cargo test
```

## Notes

The release and claim commands come from `blueutil`, which uses Apple private Bluetooth APIs. The over-the-air pairing was verified by hand. See `docs/mechanism-notes.md`.
