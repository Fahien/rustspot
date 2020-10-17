# RustSpot

Experimenting Rust gamedev on [ODROID-GO Advance](https://www.odroid.co.uk/ODROID-GO-Advance).

## Windows

To link SDL2 in Windows, you can:

1. Compile SDL2 sources:
   ```sh
   cmake -S. -Bbuild && cmake --build build --config Release
   ```

2. Specify its library path:
   ```sh
   RUSTFLAGS='-L <SDL2>/build/Release' cargo build
   ```
