# Mechanism notes

These commands verify the release and claim sequence by hand on the two Macs. Task 0 in the plan requires this before the app is trusted.

## Find the trackpad address

On the Mac that has the trackpad:

```sh
blueutil --paired --format json
```

Find the entry named Magic Trackpad and copy its address. Save a sample of this output to `src-tauri/tests/fixtures/paired.json` if it differs from the fixture already there.

## Release on Mac A

```sh
blueutil --unpair <address>
blueutil --power 0
sleep 2
blueutil --power 1
```

## Claim on Mac B

```sh
blueutil --inquiry 10
blueutil --pair <address>
blueutil --connect <address>
```

Then move the cursor to confirm Mac B has the trackpad. Repeat in reverse to switch back.

## Sleep case

Release on Mac A, then put Mac A to sleep. Wake Mac B and run only the claim commands on Mac B. The claim must succeed with Mac A asleep.

## Go or no-go

The app depends on one fact. `blueutil --pair` on Mac B must succeed over the air, with no cable and no press of the trackpad power button.

If it fails, the fallback is the Wake on Demand note model. Mac B wakes Mac A over the network to read a shared note. This is not implemented.

## Notes

- `--unpair` is marked experimental in blueutil. Note any error it prints.
- Record the exact JSON field names from `blueutil --paired --format json` here. The parser in `src-tauri/src/blueutil.rs` reads `address`, `name`, `paired`, and `connected`.
