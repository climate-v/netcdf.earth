# Visualize

This is a wrapper around the [head-scratcher] that provides the WASM specific functionality 
as well as exporting functionality to use from the javascript application.

## Building

To build, make sure rust is on a relatively recent version (anything newer than 1.50 should 
work just fine). Then make sure that the `wasm32-unknown-unknown` target is installed, 
otherwise it can be installed using:

```shell
rustup target add wasm32-unknown-unknown
```

To verify, running 

```shell
cargo build --target wasm32-unknown-unknown
```

should succeed.

Next, make sure that `wasm-pack` is installed or install it using:

```shell
cargo install wasm-pack
```

To finally build the application, run

```shell
wasm-pack build --target no-modules
```

and it should leave you with a new `/pkg` directory containing the wasm output and js 
wrappers. We need `no-modules` as target because we cannot load modules in a web worker,
where this will be used.