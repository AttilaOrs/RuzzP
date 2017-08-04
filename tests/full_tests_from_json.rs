use std::sync::*;

extern crate ruzz_p;
use ruzz_p::read_petri::{my_file_read, deseralize};
use ruzz_p::unified_petri_net::*;
use ruzz_p::basic::*;

struct History{
    rez : Vec<(usize,UnifiedToken)>,
}

struct MyConsumer {
    hist: Arc<RwLock<History>>,
    tr_id : usize,
}

impl UnifiedTokenConsumer for MyConsumer {
    fn consume(&mut self, ft: UnifiedToken){
        self.hist.write().unwrap().rez.push((self.tr_id, ft));
    }
}

pub struct ConsumerFactory {
    hist: Arc<RwLock<History>>,
}

impl ConsumerFactory {
    fn new()-> ConsumerFactory{
        let hist = History{rez: Vec::new()};
        ConsumerFactory{
            hist: Arc::new(RwLock::new(hist)),
        }
    }

    fn create_handler_for(&mut self, tr_id :usize ) -> Box<MyConsumer> {
        Box::new(MyConsumer{hist : self.hist.clone(), tr_id: tr_id})
    }

    pub fn get_current_hist(&self) -> Vec<(usize, UnifiedToken)> {
        let mut to_ret = Vec::new();
        for i in &self.hist.read().unwrap().rez {
            to_ret.push(i.clone());
        }
        to_ret
    }

    pub fn clear_history(&self)  {
        self.hist.write().unwrap().rez.clear();
    }

    pub fn create_handler_for_all_outs(&mut self,
                                   net: &UnifiedPetriNet, manager: &mut EventManager ){
        for tr_id in 0..net.get_trans_nr() {
            if net.is_trans_out(tr_id) {
                manager.add(tr_id, self.create_handler_for(tr_id));
            }
        }

    }

}

#[test]
fn controller_net_test(){
    let ww = my_file_read("unified_nets/controller.json");
    let rez = deseralize(&ww);
    let bld = rez.unwrap();


    let (net, mut man) = bld.build();

    let mut consumer_fact = ConsumerFactory::new();
    consumer_fact.create_handler_for_all_outs(&net, &mut man );

    let mut exec = SynchronousUnifiedPetriExecutor::new(&net, man);
    let inp = vec![
        (2, UnifiedToken::from_val(0.0)),
        (4, UnifiedToken::from_val(30.0)),
        (3, UnifiedToken::from_val(20.0)),
        (1, UnifiedToken::from_val(10.0)),
        (0, UnifiedToken::from_val(40.0)),
    ];
    exec.run_tick(inp);
    let current_hist =consumer_fact.get_current_hist();
    assert_eq!(vec![(0, UnifiedToken::Exist(10.000001)), (1, UnifiedToken::Exist(20.0))],
               current_hist);
}

#[test]
fn lane() {
    let ww = my_file_read("unified_nets/lane.json");
    let rez = deseralize(&ww);
    let bld = rez.unwrap();


    let (net, mut man) = bld.build();

    let mut consumer_fact = ConsumerFactory::new();
    consumer_fact.create_handler_for_all_outs(&net, &mut man );

    let mut exec = SynchronousUnifiedPetriExecutor::new(&net, man);
    let inp = vec![
        (2, UnifiedToken::from_val(0.0)),
    ];
    exec.run_tick(inp);
    let current_hist =consumer_fact.get_current_hist();
    assert_eq!(vec![(0, UnifiedToken::Exist(0.0))], current_hist);
    consumer_fact.clear_history();

    let inp = vec![
        (1, UnifiedToken::from_val(10.0)),
    ];
    exec.run_tick(inp);
    let current_hist =consumer_fact.get_current_hist();
    assert!( current_hist.is_empty());

    let inp = vec![
        (2, UnifiedToken::from_val(0.0)),
    ];
    exec.run_tick(inp);
    let current_hist =consumer_fact.get_current_hist();
    assert_eq!(vec![(0, UnifiedToken::Exist(10.0))], current_hist);
    consumer_fact.clear_history();

    let inp = vec![
        (0, UnifiedToken::from_val(5.0)),
    ];
    exec.run_tick(inp);
    let current_hist =consumer_fact.get_current_hist();
    assert!( current_hist.is_empty());

    let inp = vec![
        (2, UnifiedToken::from_val(0.0)),
    ];
    exec.run_tick(inp);
    let current_hist =consumer_fact.get_current_hist();
    assert_eq!(vec![(0, UnifiedToken::Exist(5.0))], current_hist);



}

#[test]
fn max_finder() {
    let ww = my_file_read("unified_nets/maxTableTryOut.json");
    let rez = deseralize(&ww);
    let bld = rez.unwrap();


    let (net, mut man) = bld.build();

    let mut consumer_fact = ConsumerFactory::new();
    consumer_fact.create_handler_for_all_outs(&net, &mut man );

    let mut exec = SynchronousUnifiedPetriExecutor::new(&net, man);
    let inp = vec![
        (0, UnifiedToken::from_val(0.0)),
        (1, UnifiedToken::from_val(0.3)),
    ];
    exec.run_tick(inp);
    let current_hist =consumer_fact.get_current_hist();
    assert_eq!(vec![(0, UnifiedToken::Exist(0.3))], current_hist);

    consumer_fact.clear_history();

    let inp = vec![
        (0, UnifiedToken::from_val(0.2)),
        (1, UnifiedToken::from_val(-0.3)),
    ];
    exec.run_tick(inp);
    let current_hist =consumer_fact.get_current_hist();
    assert_eq!(vec![(0, UnifiedToken::Exist(0.2))], current_hist);

}
