<div align="center">

# RustSpot 🕹️


**Experimenting Rust gamedev on [ODROID-GO Advance](https://www.odroid.co.uk/ODROID-GO-Advance).**

</div>

This project started as a relatively small experiment targeting a handheld-console, but it has grown while following a [community-driven OpenGL learning effort](https://forum.gameloop.it/d/729-learnopengl-imparare-le-basi-della-computer-graphics).


<table><tr><td>
<div align="center">

![RustSpot](https://raw.githubusercontent.com/wiki/Fahien/rustspot/image/cover.png)

_Shader variants in action_

</div>
</td></tr></table>


## Features

- Data-driven design
- Scene graph
- Shadow mapping
- glTF loading
- Physically Based Rendering
- Normal mapping

## Articles

Have a look at the [wiki](https://github.com/Fahien/rustspot/wiki) to find a list of technical articles written during development.

- [Renderer](https://github.com/Fahien/rustspot/wiki/Renderer)
- [Sky shader](https://github.com/Fahien/rustspot/wiki/Sky)
- [Grass shader](https://github.com/Fahien/rustspot/wiki/Grass)
- [Shader variants](https://github.com/Fahien/rustspot/wiki/Shader-Variants)

## Build

RustSpot works on Linux, MacOS, and Windows, but it will need Rust and SDL2.
It should not be difficult to setup the required dependencies and run the demos, but do not hesitate to contact me if you need any help.

### Windows

To link SDL2 in Windows, you can:

1. Compile SDL2 sources:
   ```sh
   cmake -S. -Bbuild && cmake --build build --config Release
   ```

2. Specify its library path:
   ```sh
   RUSTFLAGS='-L <SDL2>/build/Release' cargo build
   ```

## Screenshots

<div align="center">

<table>
<tr>
<td>

![Sponza screenshot](https://raw.githubusercontent.com/wiki/Fahien/rustspot/image/sponza.png)

_First render of Sponza_

</td>
</tr>

<tr>
<td>

![SciFi Helmet](https://raw.githubusercontent.com/wiki/Fahien/rustspot/image/scifi.png)

_Physically Based Rendering_

</td>
</tr>

<tr>
<td>

![Normal mapping](https://raw.githubusercontent.com/wiki/Fahien/rustspot/image/lion.png)

_Normal mapping in action_

</td>
</tr>

</div>