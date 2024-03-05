use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
    str::FromStr,
};

struct Edge {
    from: u16,
    to: u16,
    weight: u32,
}

#[non_exhaustive]
#[derive(Debug, PartialEq, Eq)]
pub enum EdgeErrorKind {
    NoDataRow,
    InvalidFormat,
    InvalidValue,
    Zero,
}

#[derive(Debug, PartialEq, Eq)]
struct ParseEdgeError {
    kind: EdgeErrorKind,
}

impl FromStr for Edge {
    type Err = ParseEdgeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut fields = s.split_whitespace();
        fields.next();
        let from = fields.next();
        let to = fields.next();
        let weight = fields.next();
        match (from, to, weight) {
            (Some(f), Some(t), Some(w)) => {
                match (u16::from_str(f), u16::from_str(t), u32::from_str(w)) {
                    (Ok(from), Ok(to), Ok(weight)) => Ok(Edge { from, to, weight }),
                    _ => Err(ParseEdgeError {
                        kind: EdgeErrorKind::InvalidValue,
                    }),
                }
            }
            _ => Err(ParseEdgeError {
                kind: EdgeErrorKind::InvalidFormat,
            }),
        }
    }
}

impl TryFrom<Result<String, io::Error>> for Edge {
    type Error = ParseEdgeError;

    fn try_from(item: Result<String, io::Error>) -> Result<Self, Self::Error> {
        match item {
            Ok(s) => s.parse::<Edge>(),
            _ => Err(ParseEdgeError {
                kind: EdgeErrorKind::InvalidFormat,
            }),
        }
    }
}

fn load_graph(path: &Path) -> Vec<Edge> {
    let display = path.display();
    // Open the path in read-only mode, returns `io::Result<File>`
    let file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };
    // Read the file contents into a string, returns `io::Result<usize>`
    BufReader::new(file)
        .lines()
        .filter_map(|line| Edge::try_from(line).ok())
        .collect()
}

fn main() {
    let graph = load_graph(Path::new("./data/W-d.gr"));
    for e in graph {
        println!("from: {},\tto: {},\tdistance: {}\n", e.from, e.to, e.weight)
    }
}
