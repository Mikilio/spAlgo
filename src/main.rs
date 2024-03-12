use rand::{thread_rng, Rng};
use std::num::TryFromIntError;
use std::{
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom},
    num::ParseIntError,
    os::unix::fs::FileExt,
    path::Path,
    str::FromStr,
};

const UNDEFINED: Vertex = Vertex(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vertex(u32);

impl FromStr for Vertex {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match u32::from_str(s) {
            Ok(v) => Ok(Vertex(v)),
            Err(e) => Err(e),
        }
    }
}

impl TryFrom<usize> for Vertex {
    type Error = TryFromIntError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match u32::try_from(value) {
            Ok(index) => Ok(Vertex(index + 1)),
            Err(e) => Err(e),
        }
    }
}

impl From<Vertex> for usize {
    fn from(value: Vertex) -> Self {
        (value.0 as usize) - 1
    }
}

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

pub type ParseVertexError = ParseEdgeError;

#[derive(Debug, PartialEq, Eq)]
pub struct Coordinates {
    x: i64,
    y: i64,
}

pub struct VertexCoord {
    vertex: Vertex,
    coordinates: Coordinates,
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

fn main() {
    let mut rng = thread_rng();

    let n: usize = load_max_vertex(Path::new("./data/NY.co")).into();

    let source: Vertex = rng.gen_range(0..n).try_into().unwrap();
    let mut edges: Vec<Edge> = load_edges(Path::new("./data/NY-d.gr")).collect();

    let mut queue: Vec<Vertex> = Vec::with_capacity(n);
    let mut dist: Vec<u32> = Vec::with_capacity(n);
    let mut prev: Vec<Vertex> = Vec::with_capacity(n);

    for i in 0..n {
        let v = Vertex::try_from(i).unwrap();
        queue.push(v);
        dist.push(if v == source { 0 } else { u32::MAX });
        prev.push(UNDEFINED);
    }

    let mut count = 0.0;
    let mut ratio = 0.0;

    while !queue.is_empty() {
        //feedback
        count += 1.0;
        let tmp = count / n as f64;
        if (tmp - ratio) > 0.001 {
            ratio = tmp;
            println!("ratio {:.3}", ratio);
        }

        //choose next vector
        let i = queue
            .iter()
            .enumerate()
            .min_by(|(_, &a), (_, &b)| (&dist[usize::from(a)]).cmp(&dist[usize::from(b)]))
            .map(|(index, _)| index)
            .unwrap();
        let u = queue.swap_remove(i);

        // get neighbors of u
        let (nbi, nbe): (Vec<usize>, Vec<&Edge>) = edges
            .iter()
            .enumerate()
            .filter_map(|(i, e)| {
                if e.from == u && queue.contains(&e.to) {
                    Some((i, e))
                } else {
                    None
                }
            })
            .unzip();
        // update neighbors
        for e in nbe {
            let alt = dist[usize::from(u)] + e.weight;
            if alt < dist[usize::from(e.to)] {
                dist[usize::from(e.to)] = alt;
                prev[usize::from(e.to)] = u;
            }
        }
        //discard used edges
        for i in nbi.iter().rev() {
            let _ = edges.swap_remove(*i);
        }
    }
}
