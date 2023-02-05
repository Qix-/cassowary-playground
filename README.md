# Cassowary Playground

A small playground for checking out Cassowary constraints.

Please see the demo site for instructions: https://junon.me/cassowary-playground

## Disclaimer

Please do not judge me for the code written here. It was done in ~3 hours or so.
It was purposefully hacked together to get it work as quickly as possible, and
there is a lot of _cursed nonsense_ across all of the languages used (Rust,
Javascript and PEG.js).

The error handling on this is also abysmal. I didn't optimize for a great DX,
as I wanted to play with some ideas as fast as possible (this is a tool I'm
using to research automated circuit design tooling I'm interested in creating).

So, if you're a recruiter or otherwise looking at my profile for reasons to judge,
feel free, but please know this code was a wreck from the first minute I started
on it and I do not have any interest in improving it. Sorry about that!

The interface between the WASM binary and Javascript is embarassingly crude as I
didn't want to get WASI involved (I don't particularly approve of WASI's design
decisions so tend to avoid it) and didn't want the hastle of another build tool
(wasm-pack) so I just went without any string or complex type support in the FFI
end of things. Worked out okay, just needed to use a stack-based stateful protocol.
I should probably mention that you should avoid stateful stack-based protocols
when you can.

Also, please ignore the globals in Rust. I'm... aware. I know.

**tl;dr** please don't take any code here as "good" or "I should copy this" material.
It's all really bad.

## Building

You'll need `npm`, `node` and `cargo` with a suitably recent toolchain and the `wasm32-unknown-unknown`
target installed.

Then run `make` and serve the `/www/` directory that is generated.

## License

Aside from the dependencies' respective licenses, all code under this repository
is released as either Public Domain, CC0, WTFPL, or MIT license - whatever
floats your boat. If you want a Copyright, you can use `Copyright (c) 2023, Josh Junon`.
I will not be enforcing it, though. Go nuts.
