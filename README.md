# Rust Wasm Poker Evaluatior

Equity and Draw Analysis.  

To use: 

Add the flop cards, optionally a turn & river card, then for each player, each choose a range or specific cards.

Created the 'draw' analysis classes to calculate if player has a draw or not

## Tech Stack

* ![Rust](http://rust-lang.org/logos/rust-logo-32x32.png) Rust
* <img src="dev/v-logo.svg"  width=32/> Vue3

## Screenshot

[Github pages](https://eric7237cire.github.io/)

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