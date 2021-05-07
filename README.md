# Build for netcdf-rust with Emscripten

1. Ensure emsdk is setup and installed
    - `./emsdk install 1.39.20`
    - `./emsdk activate 1.39.20`
    - `source emsdk_env.sh`
2. Ensure rust + rustup are installed
3. Clone this repository with recursive submodules
    - `git clone --recurse-submodules <url>`
4. Run build command
    - `./build.sh`