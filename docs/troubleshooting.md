# Troubleshooting

## Trackpad not found

The setup window shows no Magic Trackpad. Check that Bluetooth is on and the trackpad is powered. Run `blueutil --paired --format json` in a terminal. If the trackpad is missing there, pair it to the Mac once through System Settings.

## blueutil missing

TrackSwitch needs `blueutil` on `PATH`. Install it:

```sh
brew install blueutil
```

## Bluetooth off

`blueutil` cannot pair when Bluetooth is off. The app turns Bluetooth off and on as part of a release. If a switch fails, check Bluetooth in the menu bar and turn it on.

## Trackpad bounces back

The release turns Bluetooth off for two seconds so the old Mac does not grab the trackpad back. If the trackpad returns to the old Mac, the two seconds may be too short on that machine. Increase the sleep in `switch::release` in `src-tauri/src/switch.rs`.

## Over-the-air pairing fails

The whole design depends on `blueutil --pair` grabbing the trackpad over the air, with no cable and no press of the power button. If that fails, see the fallback note in `docs/mechanism-notes.md`. The fallback needs Apple's Wake on Demand and is not implemented.
