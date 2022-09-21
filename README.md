<h2 align="center"><b>KittenBoard</b></h2>
<h4 align="center">the perfect keyboard for all you little meow meows</h4>
<p align="center"><img src='ic_launcher-playstore.png' height='128'></p>
<p align="center">An extra comfy open source fork of the AOSP keyboard with added features.</p>

## Features

* Material You themes (from the LineageOS upstream)
* Improved emoji support
* Emoji suggestions while typing
* Emoji search (coming soon)
* Plus much more to come

## Installation

For now there are no prebuilt releases of KittenBoard available, but you can build it yourself!

## Building

### Requirements

- Android Studio
- Rust
- Python 3

### Instructions

- Verify that `rustc`, `rustup` and `python` are available in your PATH.
  - Python on Ubuntu: you can install `python-is-python3` with apt to route the `python` command to `python3`.
- Install the Android ARM and ARM64 Rust targets

```
rustup target add armv7-linux-androideabi 
rustup target add aarch64-linux-android
```
- Open the project in Android Studio and run it or build it.

## Contributions

Yes please!
