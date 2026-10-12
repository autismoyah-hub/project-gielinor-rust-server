use gielinor_world::CoordGrid;
use std::io::{self, BufRead};

fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.expect("read fixture");
        let args: Vec<&str> = line.split_whitespace().collect();
        let values: Vec<i32> = args[1..]
            .iter()
            .map(|v| v.parse().expect("fixture integer"))
            .collect();
        let coord = match args[0] {
            "new" => CoordGrid::new(values[0], values[1], values[2]),
            "raw" => Ok(CoordGrid::from_packed(values[0])),
            "move" => CoordGrid::from_packed(values[0]).translate(values[1], values[2], values[3]),
            "distance" => {
                println!(
                    "{}",
                    CoordGrid::from_packed(values[0])
                        .chebyshev_distance(CoordGrid::from_packed(values[1]))
                );
                continue;
            }
            op => panic!("unsupported fixture operation {op}"),
        };
        match coord {
            Ok(c) => println!(
                "{} {} {} {} {} {} {} {}",
                c.packed(),
                c.x(),
                c.z(),
                c.level(),
                c.mx(),
                c.mz(),
                c.lx(),
                c.lz()
            ),
            Err(_) => println!("invalid"),
        }
    }
}
