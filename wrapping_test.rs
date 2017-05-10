#![feature(inclusive_range_syntax)]
use std::u16;

#[derive(Clone,Copy)]
struct Rel {
	seq: u16,
}

struct S {
	rx_rel_seq: u16,
}

#[derive(Debug,PartialEq)]
enum E {
	Future,
	Current,
	Past,
}

impl S {
    fn fn2(&self, rel: Rel) -> E {
        if rel.seq == self.rx_rel_seq {
            E::Current
        } else {
            let cur = self.rx_rel_seq;
            let new = rel.seq;
            let future = ((new > cur) && ((new - cur) < (u16::MAX / 2))) || ((new < cur) && ((cur - new) > (u16::MAX / 2)));
            if future {
                E::Future
            } else {
                E::Past
            }
        }
    }

    fn fn3(&self, rel: Rel) -> E {
        if rel.seq == self.rx_rel_seq {
            E::Current
        } else if rel.seq.wrapping_sub(self.rx_rel_seq) < u16::MAX/2 {
            E::Future
        } else {
            E::Past
        }
    }
}

/// test that two methods have identical results on all input values
/// result: they are not :)
/// I do not want to find error in the fn2 because it will be replaced by fn3 anyway (f3 is mach faster)
fn main() {
    let mut err_total = 0;
    //let self_seq = u16::MAX;
    for self_seq in 0...u16::MAX {
        let state = S {rx_rel_seq: self_seq};
        for recvd_seq in 0...u16::MAX {
            let r = Rel{seq: recvd_seq};
            let e1 = state.fn2(r);
            let e2 = state.fn3(r);
            if e1 != e2 {
                err_total += 1;
                //println!("{:?} != {:?}, seq={}, rx_seq={}, seq-rx_seq={}", e1, e2, self_seq, recvd_seq, recvd_seq.wrapping_sub(self_seq));
            }
        }
    }
    println!("total: {}", err_total);

    //TODO add testbenches of two methods

    let now = std::time::Instant::now();
    for self_seq in 0...u16::MAX {
        let state = S {rx_rel_seq: self_seq};
        for recvd_seq in 0...u16::MAX {
            let r = Rel{seq: recvd_seq};
            let _ = state.fn2(r);
        }
    }
    let elapsed = now.elapsed();
    println!("fn2: {:?}", elapsed);

    let now = std::time::Instant::now();
    for self_seq in 0...u16::MAX {
        let state = S {rx_rel_seq: self_seq};
        for recvd_seq in 0...u16::MAX {
            let r = Rel{seq: recvd_seq};
            let _ = state.fn3(r);
        }
    }
    let elapsed = now.elapsed();
    println!("fn3: {:?}", elapsed);
}
