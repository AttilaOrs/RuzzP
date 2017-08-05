extern crate ruzz_p;
extern crate time;
extern crate timer;
extern crate chrono;
extern crate sysfs_gpio;

use sysfs_gpio::{Direction, Pin};


use timer::Timer;
use chrono::Duration;
use std::thread;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

use ruzz_p::read_petri::*;
use ruzz_p::unified_petri_net::*;
use ruzz_p::basic::*;

use time::precise_time_ns;

struct MyConsumer {
    i : Pin,
    l: bool,
}

impl UnifiedTokenConsumer for MyConsumer {
    fn consume(&mut self, ft: UnifiedToken){
        if self.l {
		self.i.set_value(0).unwrap();
	   self.l = false;
        } else {
                self.i.set_value(1).unwrap();
	   self.l = true;
        }
        println!("{:?}", self.l);
    }
}


fn main() {
    let ww = my_file_read("unified_nets/blink.json");
    let rez = deseralize(&ww);
    let bld = rez.unwrap();

    let mut my_led = Pin::new(68); // number depends on chip, etc.
    my_led.export();
    my_led.set_direction(Direction::Out);

    let (net, mut manager) = bld.build();
    manager.add(0, Box::new(MyConsumer{i:my_led, l:true}));


    let mut exec = AsynchronousThreadedUnifiedPetriExecutor::
        new(net, manager, Duration::milliseconds(100));
    let guard = exec.start();

    let ten = std::time::Duration::from_millis(10*1000);
    thread::sleep(ten);
    let stop = precise_time_ns();
}
