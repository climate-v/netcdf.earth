Web visualisation of (un)structured netCDF data.

# Installation
The visualisation can be deployed natively or within a docker container.

## Building

### Building WASM
To build the wasm module, just run the build script:
    - `./build.nre.sh`

This already makes sure all the necessary dependencies are installed.

### Run & build web frontend

0. Make sure nodejs (and npm) is installed
1. Install NPM dependencies
    - `cd earth`
    - `npm ci`
2. Run dev-server
    - `./node_modules/.bin/vite`
3. Open browser at `http://localhost:3000` to use the application

Alternatively, to build a production artifact, instead of running the dev server, you can bundle the application:
    - `./node_modules/.bin/vite build`

## Docker container

Build container
```
docker build --tag climate-v/netcdf.earth:latest --file Dockerfile .
```

Run/deploy container
```
docker run -p 80:80 climate-v/netcdf.earth:latest
```

# Usage

- Simply drag and drop netcdf files on browser window
- Use HTTP link in request `|file=<link>`
