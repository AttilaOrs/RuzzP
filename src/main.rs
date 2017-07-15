extern crate ruzz_p;
extern crate time;

use ruzz_p::read_petri::*;
use ruzz_p::unified_petri_net::*;
use ruzz_p::basic::*;

use time::precise_time_ns;


fn main() {
    let ww = my_file_read("unified_nets/controller.json");
    let rez = deseralize(&ww);
    let bld = rez.unwrap();

    let start = precise_time_ns();

    let (net, manaeger) = bld.build();
    let mut exec = SynchronousUnifiedPetriExecutor::new(&net, manaeger);
    for _ in 0..10 {
        let inp = vec![
            (2, UnifiedToken::from_val(0.0)),
            (4, UnifiedToken::from_val(30.0)),
            (3, UnifiedToken::from_val(20.0)),
            (1, UnifiedToken::from_val(10.0)),
            (0, UnifiedToken::from_val(40.0)),
        ];
        exec.run_tick(inp);
    }
    let stop = precise_time_ns();

    println!(">>> done {:?}", stop-start);
}
