extern crate ruzz_p;
extern crate time;
extern crate chrono;

use ruzz_p::read_petri::*;
use ruzz_p::unified_petri_net::*;
use ruzz_p::basic::*;

use time::precise_time_ns;
use chrono::Duration;

struct k;
impl UnifiedTokenConsumer for k{
    fn consume(&mut self, ft: UnifiedToken) {
        println!("i was born in the dark");
    }
}

fn main() {
    let ww = my_file_read("unified_nets/blink.json");
    let rez = deseralize(&ww);
    let bld = rez.unwrap();

    let start = precise_time_ns();

    let (net, mut man) = bld.build();
    man.add(0, Box::new(k{}));
    /*

    let mut exec = AsynchronousThreadedUnifiedPetriExecutor::
        new(&net, man, Duration::milliseconds(1000));
    exec.start();
    */

    let stop = precise_time_ns();

    println!(">>> done {:?}", stop-start);
}
