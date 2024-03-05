use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    str::FromStr,
};

struct Edge {
    from: u32,
    to: u32,
    weight: u32,
}

#[non_exhaustive]
#[derive(Debug, PartialEq, Eq)]
pub enum EdgeErrorKind {
    NoDataRow,
    InvalidFormat,
    InvalidValue,
}

#[derive(Debug, PartialEq, Eq)]
struct ParseEdgeError {
    kind: EdgeErrorKind,
    line: String,
}

impl FromStr for Edge {
    type Err = ParseEdgeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut fields = s.split_whitespace();
        let Some("a") = fields.next() else {
            return Err(ParseEdgeError {
                kind: EdgeErrorKind::NoDataRow,
                line: String::from_str(&s).unwrap(),
            });
        };
        let from = fields.next();
        let to = fields.next();
        let weight = fields.next();
        match (from, to, weight) {
            (Some(f), Some(t), Some(w)) => {
                match (u32::from_str(f), u32::from_str(t), u32::from_str(w)) {
                    (Ok(from), Ok(to), Ok(weight)) => Ok(Edge { from, to, weight }),
                    _ => Err(ParseEdgeError {
                        kind: EdgeErrorKind::InvalidValue,
                        line: String::from_str(&s).unwrap(),
                    }),
                }
            }
            _ => Err(ParseEdgeError {
                kind: EdgeErrorKind::InvalidFormat,
                line: String::from_str(&s).unwrap(),
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
        .filter_map(|line| match Edge::from_str(&line.ok()?) {
            Ok(e) => Some(e),
            Err(err) => {
                if err.kind != EdgeErrorKind::NoDataRow {
                    panic!(
                        "couldn't parse line:\n{}\nbecause of: {:#?}",
                        err.line, err.kind
                    )
                } else {
                    return None;
                }
            }
        })
        .collect()
}

fn main() {
    let graph = load_graph(Path::new("./data/W-d.gr"));
    for e in graph {
        println!("from: {},\tto: {},\tdistance: {}\n", e.from, e.to, e.weight)
    }
}
