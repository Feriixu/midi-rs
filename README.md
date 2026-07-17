# midi-rs

[![Build Status](https://travis-ci.org/samdoshi/midi-rs.svg?branch=master)](https://travis-ci.org/samdoshi/midi-rs)

[Documentation](http://samdoshi.github.io/midi-rs/midi/index.html)

Common Midi types for Rust.

The crate supports `no_std` environments without an allocator. Variable-length
MIDI data uses fixed-capacity [`heapless`](https://crates.io/crates/heapless)
vectors. System Exclusive payloads can contain up to 1024 bytes, and callers
choose the output capacity when converting messages:

```rust
use midi::{Message, SysExData, ToRawMessages};
use midi::Manufacturer::OneByte;

let data = SysExData::from_slice(&[1, 2, 3]).unwrap();
let message = Message::SysEx(OneByte(100), data);
let raw = message.to_raw_messages::<8>().unwrap();
```

```toml
# Cargo.toml
[dependencies]
midi = "*"
```

Very much a work in progress.
