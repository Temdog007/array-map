# array-map
A fixed-sized, stack-allocated implementation of a hash map using arrays for Rust.

## Examples
```rust
#[macro_use]
use array_map::*;

// Creates a module named Map with the ArrayMap
// Keys are bytes
// Values are signed integers
// Max size is 64 (4 * 4 * 4)
make_map!(Map, u8, i32, 4, 4, 4);
type TestMap = Map::ArrayMap;

fn main() {
  let mut t: TestMap = Default::default();

  for i in 0..64 {
      t.insert(i, i as i32 * 32);
  }
  
  for (k,v) in t.iter() { 
    println!("{} => {}", k, v);
  }
}
```
