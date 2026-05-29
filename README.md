# mensa-rs

A terminal tool that shows today's menu for [Mensa am Schloss](https://www.stw-ma.de/essen-trinken/speiseplaene/mensa-am-schloss/) in Mannheim.

```
🍽  Mensa am Schloss — Mittwoch 18.03.
  Pasta.......... Spaghetti Bolognese (Rind, Ei, Weizen)
                  2,40 € / Portion
  Vegan.......... Linsensuppe mit Brot (Sellerie, Senf)
                  1,90 € / Portion
```

Meals containing milk (`Mi`) are highlighted in red.

## Installation

### Pre-built binary (recommended)

```bash
curl -sSL https://raw.githubusercontent.com/ivbeck/mensa-rs/refs/heads/master/install.sh | bash
```

### Build from source

```bash
cargo install --path .
```

## Usage

```bash
mensa
```

Runs once and prints today's menu. The response is cached under `$XDG_CACHE_HOME/mensa/` (or `~/.cache/mensa/`) so repeated calls within the same day do not hit the network.

Useful options:

```bash
mensa --tomorrow
mensa --date 2026-05-27
mensa --week
mensa --lang en
mensa --allergen Ei --hide-allergens
mensa --favorite curry
```

The optional config file lives at `$XDG_CONFIG_HOME/mensa/config.toml` (or `~/.config/mensa/config.toml`):

```toml
language = "en"
no_cache = false
allergens = ["Mi", "Ei"]
hide_allergens = false
favorites = ["curry", "pommes", "vegan"]
```

## Android app

An Android shell app lives in `android/`. It reuses the Rust menu fetcher and parser through a JNI library (`libmensa.so`) and renders the result with a small native Java UI.

Prerequisites:

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
cargo install cargo-ndk
```

Install Android SDK API 36, Android Build Tools 36, and NDK 28.2+ in Android Studio, then build:

```bash
cd android
./gradlew :app:assembleDebug
```

The Gradle build calls `cargo ndk` and writes the generated native libraries into `android/app/src/main/jniLibs/`.

GitHub Actions builds a signed `mensa-android.apk` and attaches it to the GitHub Release when pushing to `master`, `release`, or a `v*` tag.
