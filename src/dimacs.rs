use std::fmt::Display;
use std::io;
use std::num::TryFromIntError;
use std::{
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom},
    num::ParseIntError,
    os::unix::fs::FileExt,
    path::Path,
    str::FromStr,
    vec::Vec,
};

/// Represents a vertex in the graph.
#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vertex(pub u32);

/// Represents a route in the graph.
#[derive(Debug)]
pub struct Route(pub Vec<Vertex>);

impl Route {
    /// Reverses the route.
    pub fn reverse(&mut self) {
        self.0.pop();
        self.0.reverse();
    }
    /// Joins another route to this one.
    pub fn join(&mut self, other: &mut Route) {
        self.0.append(&mut other.0);
    }
}

/// Represents a cost matrix.
pub struct CostMatrix {
    inner: File,
    size: usize,
}

impl CostMatrix {
    /// Constructs a new `CostMatrix`.
    ///
    /// # Arguments
    ///
    /// * `path` - A `Path` to the file containing the cost matrix.
    /// * `size` - The size N of the matrix NxN.
    pub fn new(path: &Path, size: usize) -> Result<Self, io::Error> {
        Ok(Self {
            inner: File::open(path)?,
            size,
        })
    }

    /// Gets the cost between two vertices.
    pub fn get(&self, source: Vertex, target: Vertex) -> Result<u32, io::Error> {
        let ref mut bytes = [0u8; std::mem::size_of::<u32>()];
        let offset = usize::from(target) * std::mem::size_of::<u32>()
            + usize::from(source) * self.size * std::mem::size_of::<u32>();
        self.inner.read_exact_at(bytes, offset as u64)?;
        Ok(u32::from_le_bytes(*bytes))
    }
}

/// Represents an undefined vertex.
pub const UNDEFINED: Vertex = Vertex(0);

impl FromStr for Vertex {
    type Err = ParseIntError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match u32::from_str(s) {
            Ok(v) => Ok(Vertex(v)),
            Err(e) => Err(e),
        }
    }
}

impl TryFrom<usize> for Vertex {
    type Error = TryFromIntError;

    #[inline]
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match u32::try_from(value) {
            Ok(index) => Ok(Vertex(index + 1)),
            Err(e) => Err(e),
        }
    }
}

impl From<Vertex> for usize {
    #[inline]
    fn from(value: Vertex) -> Self {
        (value.0 as usize) - 1
    }
}

impl Display for Vertex {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.0)
    }
}

impl Display for Route {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.0.iter().rev();
        write!(f, "{}", iter.next().unwrap())?;
        for vertex in iter {
            write!(f, "->{}", vertex)?;
        }
        write!(f, "")
    }
}

/// Represents an edge in the graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edge {
    pub from: Vertex,
    pub to: Vertex,
    pub weight: u32,
}

/// Enumerates different kinds of graph errors.
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq)]
pub enum GraphErrorKind {
    NoDataRow,
    InvalidFormat,
    InvalidValue,
}

/// Represents an error when parsing edges.
#[derive(Debug, PartialEq, Eq)]
pub struct ParseEdgeError {
    kind: GraphErrorKind,
    line: String,
}

impl FromStr for Edge {
    type Err = ParseEdgeError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut fields = s.split_whitespace();
        let Some("a") = fields.next() else {
            return Err(ParseEdgeError {
                kind: GraphErrorKind::NoDataRow,
                line: String::from_str(&s).unwrap(),
            });
        };
        let from = fields.next();
        let to = fields.next();
        let weight = fields.next();
        match (from, to, weight) {
            (Some(f), Some(t), Some(w)) => {
                match (Vertex::from_str(f), Vertex::from_str(t), u32::from_str(w)) {
                    (Ok(from), Ok(to), Ok(weight)) => Ok(Edge { from, to, weight }),
                    _ => Err(ParseEdgeError {
                        kind: GraphErrorKind::InvalidValue,
                        line: String::from_str(&s).unwrap(),
                    }),
                }
            }
            _ => Err(ParseEdgeError {
                kind: GraphErrorKind::InvalidFormat,
                line: String::from_str(&s).unwrap(),
            }),
        }
    }
}

/// Represents an error when parsing vertices.
pub type ParseVertexError = ParseEdgeError;

#[derive(Debug, PartialEq, Eq)]
pub struct Coordinates {
    x: i64,
    y: i64,
}

/// Represents coordinates.
pub struct VertexCoord {
    vertex: Vertex,
    coordinates: Coordinates,
}

impl FromStr for VertexCoord {
    type Err = ParseVertexError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut fields = s.split_whitespace();
        let Some("v") = fields.next() else {
            return Err(ParseVertexError {
                kind: GraphErrorKind::NoDataRow,
                line: String::from_str(&s).unwrap(),
            });
        };
        let vertex = fields.next();
        let x = fields.next();
        let y = fields.next();
        match (vertex, x, y) {
            (Some(v), Some(x), Some(y)) => {
                match (Vertex::from_str(v), i64::from_str(x), i64::from_str(y)) {
                    (Ok(vertex), Ok(x), Ok(y)) => Ok(VertexCoord {
                        vertex,
                        coordinates: Coordinates { x, y },
                    }),
                    _ => Err(ParseVertexError {
                        kind: GraphErrorKind::InvalidValue,
                        line: String::from_str(&s).unwrap(),
                    }),
                }
            }
            _ => Err(ParseEdgeError {
                kind: GraphErrorKind::InvalidFormat,
                line: String::from_str(&s).unwrap(),
            }),
        }
    }
}

/// Loads edges from a file downloaded from https://www.diag.uniroma1.it/challenge9/download.shtml.
#[inline]
pub fn load_edges(path: &Path) -> impl Iterator<Item = Edge> {
    let display = path.display();
    // Open the path in read-only mode, returns `io::Result<File>`
    let file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };
    // Read the file contents into a string, returns `io::Result<usize>`
    BufReader::new(file)
        .lines()
        .filter_map(|line| match Edge::from_str(&line.ok()?) {
            Ok(e) => Some(e),
            Err(err) => {
                if err.kind != GraphErrorKind::NoDataRow {
                    panic!(
                        "couldn't parse line:\n{}\nbecause of: {:#?}",
                        err.line, err.kind
                    )
                } else {
                    return None;
                }
            }
        })
}

/// Loads coordinates from a file downloaded from https://www.diag.uniroma1.it/challenge9/download.shtml.
#[inline]
pub fn load_coordinates(path: &Path) -> impl Iterator<Item = Coordinates> {
    let display = path.display();
    // Open the path in read-only mode, returns `io::Result<File>`
    let file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };
    // Read the file contents into a string, returns `io::Result<usize>`
    BufReader::new(file)
        .lines()
        .filter_map(|line| match VertexCoord::from_str(&line.ok()?) {
            Ok(v) => Some(v.coordinates),
            Err(err) => {
                if err.kind != GraphErrorKind::NoDataRow {
                    panic!(
                        "couldn't parse line:\n{}\nbecause of: {:#?}",
                        err.line, err.kind
                    )
                } else {
                    return None;
                }
            }
        })
}

/// Loads the maximum vertex from a file downloaded from https://www.diag.uniroma1.it/challenge9/download.shtml.
#[inline]
pub fn load_max_vertex(path: &Path) -> Vertex {
    let display = path.display();
    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    let mut char = [0u8];
    let mut end = file.seek(SeekFrom::End(-1)).unwrap();
    while {
        end = end - 1;
        file.read_at(&mut char, end).ok();
        char != [0x0a]
    } {}
    let mut buf = String::new();
    file.seek(SeekFrom::Start(end + 1)).ok();
    BufReader::new(file).read_line(&mut buf).ok();
    match VertexCoord::from_str(&buf) {
        Ok(v) => v.vertex,
        Err(err) => panic!(
            "couldn't parse line:\n{}\nbecause of: {:#?}",
            err.line, err.kind
        ),
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, os::unix::fs::FileExt, path::Path};

    use crate::dimacs::Vertex;

    use super::CostMatrix;

    #[test]
    fn cost_matrix_test() {
        let path = Path::new(&"./test/costmatrix_test.cost");
        {
            let matrix: Vec<u32> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
            let file = File::create(path).unwrap();
            let ptr: *const [u8; 9 * 4] = matrix.as_ptr().cast();
            let bytes = unsafe { *ptr };
            let rows = bytes.splitn(3, |_| false);
            for (i, row) in rows.enumerate() {
                file.write_all_at(row, i as u64 * (4u64 * 9u64)).unwrap();
            }
        }
        let cost = CostMatrix::new(path, 3).unwrap();
        for x in 1..4 {
            for y in 1..4 {
                assert_eq!(cost.get(Vertex(x), Vertex(y)).unwrap(), y + 3 * (x - 1));
            }
        }
    }
}
