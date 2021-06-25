Web visualisation of (un)structured netCDF data.

# Installation
The visualisation can be deployed natively or within a docker container.

## Native build

### Build for netcdf-rust with Emscripten

1. Ensure emsdk is setup and installed
    - `./emsdk install 1.39.20`
    - `./emsdk activate 1.39.20`
    - `source emsdk_env.sh`
2. Ensure rust + rustup are installed
3. Clone this repository with recursive submodules
    - `git clone --recurse-submodules <url>`
4. Run build command
    - `./build.sh`

To (re)build only the wasm library, run the following:
    - `./build.sh fast`

#### Run visualization with custom data

0. Make sure nodejs (and npm) is installed
1. Install NPM dependencies
    - `cd earth`
    - `npm ci`
2. Run dev-server
    - `node dev-server.js 8001`
3. Copy data files into `public/data/weather/current`
4. Open browser with url: `http://localhost:8001/#current/wind/surface/level/filename=<filename>`
    - `<filename>` is the name of the data file you want to visualize without `.nc` extension (e.g. `REG_PL_0001` for `REG_PL_0001.nc`)

## Docker container

Build container
```
docker build --tag climate-v/nre:latest .
```

Run/deploy container
```
docker run -p 80:80 climate-v/nre:latest
```

# Usage

- Simply drag and drop netcdf files on browser window
- Use HTTP link in request `|file=<link>`
