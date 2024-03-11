use std::{
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom},
    os::unix::fs::FileExt,
    path::Path,
    str::FromStr,
};

pub struct Edge {
    from: Vertex,
    to: Vertex,
    weight: u32,
}

#[non_exhaustive]
#[derive(Debug, PartialEq, Eq)]
pub enum GraphErrorKind {
    NoDataRow,
    InvalidFormat,
    InvalidValue,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseEdgeError {
    kind: GraphErrorKind,
    line: String,
}

impl FromStr for Edge {
    type Err = ParseEdgeError;

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

pub type Vertex = u32;

pub type ParseVertexError = ParseEdgeError;

pub struct VertexCoord {
    vertex: Vertex,
    x: i64,
    y: i64,
}

impl FromStr for VertexCoord {
    type Err = ParseVertexError;

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
                    (Ok(vertex), Ok(x), Ok(y)) => Ok(VertexCoord { vertex, x, y }),
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

fn load_edges(path: &Path) -> impl Iterator<Item = Edge> {
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

pub fn load_coordinates(path: &Path) -> impl Iterator<Item = VertexCoord> {
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

pub fn load_vertices(path: &Path) -> impl Iterator<Item = Vertex> {
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
    let max_vertex = match VertexCoord::from_str(&buf) {
        Ok(v) => v.vertex + 1,
        Err(err) => panic!(
            "couldn't parse line:\n{}\nbecause of: {:#?}",
            err.line, err.kind
        ),
    };
    return 1..max_vertex;
}

fn main() {
    for e in load_edges(Path::new("./data/W-d.gr")) {
        let _ = e.from + e.to + e.weight;
    }
    for v in load_vertices(Path::new("./data/W.co")) {
        let _ = v;
    }
    for c in load_coordinates(Path::new("./data/W.co")) {
        let _ = c.vertex as i64 + c.x + c.y;
    }
}
