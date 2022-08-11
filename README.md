# Baum

A tree format as simple as it gets.

## Motivation

A lot of data programmers work with has some kind of hierarchical structure. There are many file formats that support hierarchical tree-like data, e.g. XML (HTML, SVG, ...), Json, Yaml, Toml and many more. What all of these formats have in common is that they are textual format, meaning that hierarchy is encoded on top of a text encoding (e.g. UTF-8). This encoding 
- Hierarchical data
- XML, Json, Yaml, Toml, ... encode hierarchy on top of a text encoding which requires escaping, special syntax and complex parsers to get it right.

### Goals

- Conceptually simple: It should be easy to build `Baum` support for new programming languages. Being conceptually simple should allow many formats to be build on top of the `Baum` format.

### Non-Goals

- Performance / Space efficiency: This format isn't optimized for space efficiency or read / write performance. 


## Data model

A `Baum` tree consists of nodes. A node can either be a leaf or an inner node. Leaves contain a sequence of bytes, inner nodes a sequence of nodes. This is the actual implementation:

```rust
pub enum Node {
    Leaf(Vec<u8>),
    Inner(Vec<Node>)
}
```

It's simple as that!

## Serialization format

`Baum` trees are stored as binary files. A `Baum` file always starts with a five-byte magic number `"BAUM1"` (`0x42_41_55_4d_31`). This magic number is followed by the serialization of the root node.

A node is represented as `<type><len><data>`.

- `<type>` is a single byte that indicates whether the node is a leaf (`0x00`) or an inner node (`0x01`). Other values are invalid.
- `<len>` is a 64-bit unsigned [little-endian](https://en.wikipedia.org/wiki/Endianness) integer. For leaf nodes, it contains the number of bytes the node contains. For inner nodes, it contains the number of children the node has.
- `<data>` can be an arbitrary number of bytes. For leaf nodes, it contains exactly `<len>` bytes. For inner nodes, it contains the concatenation of all serialized child nodes. This leads to a pre-order traversal of the `Baum`.

### Example

The following example shows how a `Baum` is serialized using the following simple tree structure:

```
Node
- 0x01
- Node
  - 0x02
  - 0x03
- 0x04_05
```

The `Baum` consists of a root node that has three children. The first and third child are leaf nodes containing bytes `0x01` and `0x04_05` respectively. The second child is another inner node containing two leaf nodes (`0x02` and `0x03`). The `Baum` can be created using the following rust code:

```rust
use baum::Node::{Leaf, Inner};

let node = Inner(vec!(
    Leaf(vec!(1)),
    Inner(vec!(
        Leaf(vec!(2)),
        Leaf(vec!(3)),
    )),
    Leaf(vec!(4, 5)),
));
```

Serializing the `Baum` described above yields the following bytes. Take a look at the comments for a description of each part of the serialization:

```rust
let exp = vec!(
    0x42, 0x41, 0x55, 0x4d, 0x31,   // magic numbers
    1,                              // inner node
    3, 0, 0, 0, 0, 0, 0, 0,         // length 3
    0,                              // - first leaf
    1, 0, 0, 0, 0, 0, 0, 0,         //   length 1
    1,                              //   payload
    1,                              // - recursive inner node
    2, 0, 0, 0, 0, 0, 0, 0,         //   length 2
    0,                              //   - leaf 2
    1, 0, 0, 0, 0, 0, 0, 0,         //     length 1
    2,                              //     payload
    0,                              //   - leaf 3
    1, 0, 0, 0, 0, 0, 0, 0,         //     length 1
    3,                              //     payload
    0,                              // - third leaf
    2, 0, 0, 0, 0, 0, 0, 0,         //   length 2
    4, 5,                           //   payload
);
assert_eq!(node.serialize(), exp);
```

### File extension

By convention, `Baum` files use the file extension `.baum`.

