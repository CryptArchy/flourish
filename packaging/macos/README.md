# Packaging Flourish for macOS

```sh
scripts/bundle-macos.sh --universal
```

That produces `target/bundle/Flourish.app`, ad-hoc signed, containing a
universal binary and a procedurally rendered icon. Drag it to `/Applications`.

Drop `--universal` for a faster host-architecture-only build during development.

## What is in the bundle

```
Flourish.app/Contents/
├── Info.plist            generated from packaging/macos/Info.plist
├── MacOS/flourish        the release binary
├── PkgInfo
└── Resources/Flourish.icns
```

The version in `Info.plist` is substituted from `Cargo.toml` at build time, so
`Cargo.toml` stays the only place a version number is written.

`LSUIElement` is the key that matters. It makes Flourish an agent: no Dock
tile, no entry in the app switcher, and — the point — no stealing focus from
the deck when it launches. The binary also sets the accessory activation
policy at runtime so `cargo run` behaves identically, but only the plist key
gets it right from the very first frame.

## The icon

There is no image checked into this repository. `src/icon.rs` draws the mark
analytically, `examples/iconset.rs` renders it at all ten sizes macOS asks
for, and `iconutil` packs those into the `.icns`. The same code draws the
menu-bar icon, so the two can never drift apart, and adding a size means
adding one line rather than re-exporting a set of images.

To look at the icon without building the whole bundle:

```sh
cargo run --example iconset -- target/Flourish.iconset
```

## Shipping it to other people

Ad-hoc signing is enough to run the app on the machine that built it. It is
**not** enough for anyone else: Gatekeeper will refuse a downloaded bundle
that is not signed with a Developer ID and notarized, and the user gets a
"damaged and can't be opened" dialog rather than anything actionable.

Doing it properly needs a paid Apple Developer account, and then:

1. **Sign** with a Developer ID Application certificate. The script already
   passes `--options runtime`, which enables the hardened runtime that
   notarization requires:

   ```sh
   CODESIGN_IDENTITY="Developer ID Application: Your Name (TEAMID)" \
     scripts/bundle-macos.sh --universal
   ```

2. **Notarize** — upload to Apple, wait for the automated scan:

   ```sh
   ditto -c -k --keepParent target/bundle/Flourish.app target/Flourish.zip
   ```

   ```sh
   xcrun notarytool submit target/Flourish.zip --keychain-profile "AC_PASSWORD" --wait
   ```

3. **Staple** the ticket so it validates without a network round trip:

   ```sh
   xcrun stapler staple target/bundle/Flourish.app
   ```

Until that happens, anyone you hand the app to can get past Gatekeeper with
right-click → Open, or:

```sh
xattr -dr com.apple.quarantine /Applications/Flourish.app
```

CI builds and uploads an ad-hoc-signed bundle on every run, which is useful
for testing but is not a distributable artifact for the reason above.
