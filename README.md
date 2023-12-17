# Rust Wasm Poker Evaluatior

Equity and Draw Analysis.  Add the flop cards, then for each player, each choose a range or specific cards.

## Tech Stack

* ![Rust](http://rust-lang.org/logos/rust-logo-32x32.png) Rust
* ![Vue](dev/v-logo.svg =32x32)

## Screenshot

[Github pages](https://eric7237cire.github.io/)

![Screenshot](dev/screenshot.png)

## Credits

### Wasm Postflop

Used UI components and the Range class of https://github.com/b-inary/wasm-postflop

### Rust Poker 

Used core classes and ranking from https://github.com/elliottneilclark/rs-poker

## Dev

### Prereqs

Install rust & wasm pack.

For example:

``` 
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
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