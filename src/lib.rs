
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Node {
    Leaf(Vec<u8>),
    Inner(Vec<Node>)
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Leaf(bytes) => {
                write!(f, "0x")?;
                for (i, b) in bytes.iter().enumerate() {
                    if i > 0 {
                        write!(f, "_")?;
                    }
                    write!(f, "{:02x}", b)?;
                }
            },
            Self::Inner(nodes) => {
                write!(f, "(")?;
                for (i, node) in nodes.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    node.fmt(f)?;
                }
                write!(f, ")")?;
            }
        }
        Ok(())
    }
}

impl Node {
    pub fn new_leaf(bytes: Vec<u8>) -> Node {
        Node::Leaf(bytes)
    }

    pub fn new_inner(nodes: Vec<Node>) -> Node {
        Node::Inner(nodes)
    }

    /// Returns `true` if the node is [`Leaf`].
    ///
    /// [`Leaf`]: enum.Node.html#variant.Leaf
    #[must_use = "if you intended to assert that this is a leaf, consider `.unwrap_leaf()` instead"]
    #[inline]
    pub fn is_leaf(&self) -> bool {
        matches!(*self, Node::Leaf(_))
    }

    /// Returns `true` if the node is [`Inner`].
    ///
    /// [`Inner`]: enum.Node.html#variant.Inner
    #[must_use = "if you intended to assert that this is an inner node, consider `.unwrap_inner()` instead"]
    #[inline]
    pub fn is_inner(&self) -> bool {
        matches!(*self, Node::Inner(_))
    }

    pub fn serialize(&self) -> Vec<u8> {
        // include magic number
        let mut res = "BAUM1".as_bytes().to_vec();
        self._serialize(&mut res);
        res
    }

    fn _serialize(&self, w: &mut Vec<u8>) {
        match self {
            Node::Leaf(bytes) => {
                w.push(0);
                w.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
                w.extend_from_slice(&bytes);
            },
            Node::Inner(nodes) => {
                w.push(1);
                w.extend_from_slice(&(nodes.len() as u64).to_le_bytes());
                for node in nodes {
                    node._serialize(w);
                }
            }
        }
    }

    pub fn serialize_into<W>(&self, writer: &mut W) -> std::io::Result<()> 
    where 
        W: std::io::Write 
    {
        writer.write("BAUM1".as_bytes())?;
        self._serialize_into(writer)
    }

    fn _serialize_into<W>(&self, writer: &mut W) -> std::io::Result<()> 
    where 
        W: std::io::Write 
    {
        match self {
            Node::Leaf(b) => {
                writer.write(&[0])?;
                writer.write(&(b.len() as u64).to_le_bytes())?;
                writer.write(&b)?;
                Ok(())
            }
            Node::Inner(nodes) => {
                writer.write(&[1])?;
                writer.write(&(nodes.len() as u64).to_le_bytes())?;
                for node in nodes {
                    node._serialize_into(writer)?;
                }
                Ok(())
            }
        }
    }

    pub fn deserialize<'a>(bytes: &'a [u8]) -> Result<Self, Error> {
        Self::deserialize_from(bytes)
    }

    pub fn deserialize_from<R>(mut reader: R) -> Result<Self, Error> 
    where
        R: std::io::Read
    {
        let mut magic_num = vec!(0; 5);
        reader.read_exact(&mut magic_num)?;
        if magic_num != "BAUM1".as_bytes() {
            return Err(Error::InvalidMagicNumber)
        }
        
        let res = Self::_deserialize_from(&mut reader)?;
        
        // check if whole input has been processed
        let mut buf = [0];
        if reader.read(&mut buf)? != 0 {
            return Err(Error::AdditionalBytes);
        }

        Ok(res)
    }

    fn _deserialize_from<R>(reader: &mut R) -> Result<Self, Error> 
    where
        R: std::io::Read
    {
        let type_byte = read_u8(reader)?;
        match type_byte {
            // leaf
            0 => {
                let len = read_u64(reader)?;
                let mut bytes = vec!(0; len as usize);
                reader.read_exact(&mut bytes)?;
                Ok(Node::Leaf(bytes))
            }
            // inner
            1 => {
                let len = read_u64(reader)?;
                let mut nodes = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    nodes.push(Node::_deserialize_from(reader)?);
                }
                Ok(Node::Inner(nodes))
            }
            // error
            _ => {
                Err(Error::InvalidNodeType)
            }
        }
    }

}

fn read_u8<R: std::io::Read>(input: &mut R) -> Result<u8, Error> {
    let mut buf = [0];
    input.read_exact(&mut buf)?;
    Ok(buf[0])
}
fn read_u64<R: std::io::Read>(input: &mut R) -> Result<u64, Error> {
    let mut buf: [u8; 8] = [0; 8];
    input.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    InvalidMagicNumber,
    InvalidNodeType,
    AdditionalBytes
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IOError(err)
    }
}


#[test]
fn fmt() {
    let baum = Node::Inner(vec!(
        Node::Leaf(vec!(1, 2, 3)),
        Node::Inner(vec!(
            Node::Leaf(vec!()),
            Node::Leaf(vec!(0x1)),
            Node::Leaf(vec!(0x23, 16, 10, 11*16+12)),
        ))
    ));

    let s = format!("{}", baum);
    let exp = "(0x01_02_03 (0x 0x01 0x23_10_0a_bc))";
    assert_eq!(s, exp);
}

#[test]
fn test_basic() {
    let node = Node::Inner(vec!(
        Node::Leaf(vec!(1)),
        Node::Leaf(vec!(2)),
        Node::Leaf(vec!(3, 4)),
    ));
    
    // encoded:
    let exp = vec!(
        // magic numbers
        66, 65, 85, 77, 49,
        // inner node
        1,
        // length 3
        3, 0, 0, 0, 0, 0, 0, 0,
        // first leaf
        0,
        // length 1
        1, 0, 0, 0, 0, 0, 0, 0,
        // payload
        1,
        // second leaf
        0,
        // length 1
        1, 0, 0, 0, 0, 0, 0, 0,
        // payload
        2,
        // third leaf
        0,
        // length 1
        2, 0, 0, 0, 0, 0, 0, 0,
        // payload
        3, 4,
    );
    
    let mut v = vec!();
    node.serialize_into(&mut v).unwrap();
    assert_eq!(v, exp);
    
    
    // test decode
    let mut bytes_slice = &exp[..];
    assert_eq!(node, Node::deserialize_from(&mut bytes_slice).unwrap());
}

#[test]
fn test_leaf() {
    let node = Node::Leaf(vec!(1, 2, 3, 4));
    
    // encoded:
    let exp = vec!(
        // magic numbers
        66, 65, 85, 77, 49,
        // inner node
        0,
        // length 3
        4, 0, 0, 0, 0, 0, 0, 0,
        // payload
        1, 2, 3, 4,
    );
    
    let mut v = vec!();
    node.serialize_into(&mut v).unwrap();
    assert_eq!(v, exp);
    
    // test decode
    let mut bytes_slice = &exp[..];
    assert_eq!(node, Node::deserialize_from(&mut bytes_slice).unwrap());
}

#[test]
fn test_recursive() {
    let node = Node::Inner(vec!(
        Node::Leaf(vec!(1)),
        Node::Inner(vec!(
            Node::Leaf(vec!(2)),
            Node::Leaf(vec!(2)),
        )),
        Node::Leaf(vec!(3, 4)),
    ));
    
    // encoded:
    let exp = vec!(
        // magic numbers
        66, 65, 85, 77, 49,
        // inner node
        1,
        // length 3
        3, 0, 0, 0, 0, 0, 0, 0,
        // first leaf
        0,
        // length 1
        1, 0, 0, 0, 0, 0, 0, 0,
        // payload
        1,
        
        // resursive inner node
        1,
        // length 2
        2, 0, 0, 0, 0, 0, 0, 0,
        // payload
        
        // leaf 2
        0,
        // length 1
        1, 0, 0, 0, 0, 0, 0, 0,
        // payload
        2,
        // leaf 2
        0,
        // length 1
        1, 0, 0, 0, 0, 0, 0, 0,
        // payload
        2,
        
        // third leaf
        0,
        // length 1
        2, 0, 0, 0, 0, 0, 0, 0,
        // payload
        3, 4,
    );
    
    let mut v = vec!();
    node.serialize_into(&mut v).unwrap();
    assert_eq!(v, exp);
    
    // test decode
    let mut bytes_slice = &exp[..];
    assert_eq!(node, Node::deserialize_from(&mut bytes_slice).unwrap());
}