# Rust Wasm Poker Evaluatior 

![Github Actions](https://github.com/eric7237cire/poker_eval/actions/workflows/build.yml/badge.svg)

Equity and Draw Analysis.  Try it out on [Github pages](https://eric7237cire.github.io/poker_eval)

Card recognition using image detection & classification using [ultralytics](https://github.com/ultralytics/ultralytics)

## Tech Stack

* ![Rust](http://rust-lang.org/logos/rust-logo-32x32.png) Rust
* <img src="dev/v-logo.svg"  width=32/> Vue3

## Screenshot

![Screenshot](dev/screenshot.png)

## Credits

### Wasm Postflop

Used UI components and the Range class of https://github.com/b-inary/wasm-postflop

Used same worker/Wasm architecture

### Rust Poker 

Used core classes and ranking from https://github.com/elliottneilclark/rs-poker

### Comlink

Web Worker <=> App communication 

https://github.com/GoogleChromeLabs/comlink

## Dev

### Prereqs

Install node

```  
sudo npm install n -g
sudo n stable
sudo npm install -g npm@10.2.5
```

Install rust & wasm pack.

For example:

``` 
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

cargo install cargo-watch
```

This will watch rust & vue files
```
cd vue-poker
npm install
npm run r-dev
```

## Tests

```
cd rust
cargo test
```

## Formatting

```
cd rust
cargo fmt
cargo fix
```

```
cd vue-poker
npm run format
```

# Running prod static server

```
sudo npm install -g serve
serve -s dist
```

http://localhost:3000

# Jupyter

(Used WSL non root user)
```
python3 -m pip install scipy
python3 -m pip install matplotlib --force-reinstall
python3 -m pip install jupyterlab

jupyter lab
```