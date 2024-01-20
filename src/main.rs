pub mod game;

use hex2d::{Coordinate, Direction, Spin};

fn main() {
    println!("Hello, world!");

    let origin = Coordinate::new(0, 0);

    let mut total = 0;
    for r in 0..=40 {
        println!();
        println!("r = {r}:");
        let ring = origin.ring_iter(r, Spin::CW(Direction::XY));

        let ring_count = ring.count();
        println!("num tiles in ring:  {ring_count}");

        total += ring_count;
        println!("total tiles:  {total}");

        // for (i, tile) in ring.enumerate() {
        //     println!("{:4}: {tile:?}", i + 1);
        // }
    }

    dbg!((3.0_f64).sqrt());
}
