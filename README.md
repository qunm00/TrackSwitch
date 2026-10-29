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

TrackSwitch is not signed by Apple. macOS blocks an unsigned app that carries the quarantine flag, and a browser download adds that flag. Two routes avoid the block. Pick one route for each Mac.

### Build on your own Macs

Use this route when you own both Macs. The app is compiled on the Mac, so macOS does not block it.

```sh
git clone https://github.com/qunm00/TrackSwitch.git
cd TrackSwitch
cargo install tauri-cli --version "^2"
cargo tauri build
cp -R "src-tauri/target/release/bundle/macos/TrackSwitch.app" /Applications/
```

### Install with the script

`scripts/install.sh` downloads the release zip with `curl`. Files that `curl` downloads do not get the quarantine flag. The script unpacks the app into `~/Applications` and adds a local signature, so Apple Silicon runs it.

```sh
curl -fsSL https://raw.githubusercontent.com/qunm00/TrackSwitch/main/scripts/install.sh | sh
```

To install a given version, pass the tag:

```sh
curl -fsSL https://raw.githubusercontent.com/qunm00/TrackSwitch/main/scripts/install.sh | sh -s -- v0.1.0
```

The script checks the SHA-256 value when the release includes a `.sha256` file. It stops if the value does not match.

The app goes to `~/Applications` by default, so an update needs no admin password. To use another folder, set `INSTALL_DIR`:

```sh
curl -fsSL https://raw.githubusercontent.com/qunm00/TrackSwitch/main/scripts/install.sh | INSTALL_DIR="/Applications" sh
```

The login item follows the app path, so both folders work.

## First run

1. Open TrackSwitch on both Macs.
2. The setup window opens on first run.
3. Pick the Magic Trackpad from the list.
4. Type the other Mac's name, or wait for the app to find it on the network.
5. Click Save.

TrackSwitch starts at login on both Macs.

## Switch

Click the menu bar icon. Choose `Switch to <other Mac>` to hand the trackpad over. Choose `This Mac (take trackpad)` to pull it back.

## Updates

Choose `Check for updates` in the menu bar menu. The app reads `latest.json` from the newest GitHub release. It downloads the newer app, checks a signature over the download, installs the app in place, and restarts.

That signature comes from your own key. It does not need an Apple certificate. Make the key pair one time:

```sh
cargo tauri signer generate -w ~/.tauri/trackswitch.key
```

The command prints a public key. Paste it into `plugins.updater.pubkey` in `src-tauri/tauri.conf.json`. Create an environment named `release` in the repository settings, then add the private key as `TAURI_SIGNING_PRIVATE_KEY` and its password as `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Add a required reviewer to that environment to hold every release until you approve it. The test in the workflow stops the release while the placeholder text is still in the configuration.

The updater verifies the signature before it changes anything. A download that fails the check leaves the installed app alone.

macOS asks for an admin password when the app sits in `/Applications`, because a normal process cannot write that folder. Install to `~/Applications` to skip the prompt. The login item follows the app path, so both folders work.

A fork can run its own pipeline. Make your own key pair with `cargo tauri signer generate`, paste the new public key into `plugins.updater.pubkey`, and point `plugins.updater.endpoints` at your own `latest.json`. Then add the private key and its password as the two secrets of a `release` environment in your repository. A fork that skips these changes keeps following the releases from `qunm00/TrackSwitch`, and it cannot publish an update that an existing install accepts.

## Build from source

```sh
cargo tauri build
```

Build on macOS. The unit tests run anywhere:

```sh
cd src-tauri && cargo test
```

## Release

`.github/workflows/release.yml` builds the app on a macOS runner. It runs the tests, builds the `.app`, adds an ad-hoc signature, and attaches these files to the release:

- `TrackSwitch.app.zip` and `TrackSwitch.app.zip.sha256`, for a manual install.
- `TrackSwitch.app.tar.gz` and `TrackSwitch.app.tar.gz.sig`, for the updater.
- `latest.json`, the manifest that the app reads to find an update.

The workflow needs one secret, `TAURI_SIGNING_PRIVATE_KEY`, and its password as `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Both live on the `release` environment, which the job declares, so an approval rule on that environment also gates the signing step. It stops with an error when the secret is missing. It does not sign with an Apple certificate and does not notarize the app.

The build reads `src-tauri/tauri.release.conf.json`, which turns on `createUpdaterArtifacts`. Local builds skip that file, so a build on your own Mac needs no key.

Push a tag to start the workflow. The tag must start with `v`:

```sh
git tag -a v0.1.0 -m "v0.1.0"
git push origin v0.1.0
```

You can also run the workflow by hand from the Actions tab and give it an existing tag.

The runner uses Apple Silicon, so the app is arm64 only. For Intel Macs, add `macos-13` to a job matrix, or add the `x86_64-apple-darwin` target and build with `--target universal-apple-darwin`.

To build by hand on your Mac instead, run the same steps:

```sh
cargo tauri build
ditto -c -k --keepParent "src-tauri/target/release/bundle/macos/TrackSwitch.app" TrackSwitch.app.zip
shasum -a 256 TrackSwitch.app.zip > TrackSwitch.app.zip.sha256
gh release create v0.1.0 TrackSwitch.app.zip TrackSwitch.app.zip.sha256 --title v0.1.0
```

`scripts/install.sh` expects the asset named `TrackSwitch.app.zip` on a `vX.Y.Z` tag. Keep the name.

## Notes

The release and claim commands come from `blueutil`, which uses Apple private Bluetooth APIs. The over-the-air pairing was verified by hand. See `docs/mechanism-notes.md`.
