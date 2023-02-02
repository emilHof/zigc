Zigc aims to provide the basic functionality for compiling and linking [Zig](https://ziglang.org/)
libraries into your [Rust](https://www.rust-lang.org/) projects.

### Disclaimer

[zig](https://ziglang.org/download/) is a requirement to compile `.zig` files with this crate.

### Usage

Given the following function definition as an example:

```zig
// main.zig
const std = @import("std");

export fn add(a: c_int, b: c_int) callconv(.C) c_int {
    return a + b;
}
```

1. Import the `zigc` and `libc` crates:

```toml
[dependencies]
libc = "*"

[build-dependencies]
zigc = "*"
```

2. Specify the `.zig` source file in your build script and zigc automatically compiles it into the right
   directory and links the artifacts into your rust binary.

```rust
/* build.rs */
fn main() {
    zigc::Build::new()
        .file("./src/main.zig")
        .finish();
}
```

3. Import the functions in your Rust source code.

```rust
/* main.rs */
extern crate libc;
use libc::c_int;

#[link(name = "main", kind = "dylib")]
extern "C" {
    fn add(a: c_int, b: c_int) -> c_int;
}

fn main() {
    let res = unsafe { add(2, 2) };
    println!("{res}");
}
```

4. Build/run your crate.

```
$ cargo run
4
```

### Roadmap

- [x] Basic `.zig` compilation
- [x] MVP linking of `.so` files to cargo projects.
- [x] Add logging.
- [x] Automatic target specification using cargo's `TARGET` flag.
- [x] Allow compilation and linking of `static` Zig libraries.
- [ ] Add more options to `Build`
  - [ ] Additional flags (`-cflags`, `-target`, `-mcpu`, etc)
  - [x] Name output library file.
  - [ ] Specify additional `include` libraries
- [ ] Ability to compile and link multiple `.zig` files.

### Contribute

Any discovered issues, feature requests, and pull request are highly encouraged and appreciated! :)
