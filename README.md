# RustSpot

Experimenting Rust gamedev on [ODROID-GO Advance](https://www.odroid.co.uk/ODROID-GO-Advance) with [libgo2-rs](https://github.com/Fahien/libgo2-rs).

## Windows

To link SDL2 in Windows, you can:

1. compile SDL2 sources
   ```sh
   cmake -S. -Bbuild && cmake --build build --config Release
   ```

2. Specify its library path
   ```sh
   RUSTFLAGS='-L <SDL2>/build/Release' cargo build
   ```
