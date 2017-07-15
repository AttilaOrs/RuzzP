use std::cell::RefCell;
use std::rc::Rc;

extern crate ruzz_p;
use ruzz_p::read_petri::{my_file_read, deseralize};
use ruzz_p::unified_petri_net::*;
use ruzz_p::basic::*;



struct History{
    rez : Vec<(usize, UnifiedToken)>,
}

struct MyConsumer {
    hist: Rc<RefCell<History>>,
    tr_id : usize,
}

impl UnifiedTokenConsumer for MyConsumer {
    fn consume(&mut self, ft: UnifiedToken){
        self.hist.borrow_mut().rez.push((self.tr_id, ft));
    }
}

pub struct ConsumerFactory {
    hist: Rc<RefCell<History>>,
}

impl ConsumerFactory {
    fn new()-> ConsumerFactory{
        let hist = History{rez: Vec::new()};
        ConsumerFactory{
            hist: Rc::new(RefCell::new(hist)),
        }
    }

    fn create_handler_for(&mut self, tr_id :usize ) -> Box<MyConsumer> {
        Box::new(MyConsumer{hist : self.hist.clone(), tr_id: tr_id})
    }

    pub fn get_current_hist(&self) -> Vec<(usize, UnifiedToken)> {
        let mut to_ret = Vec::new();
        for i in &self.hist.borrow().rez {
            to_ret.push(i.clone());
        }
        to_ret
    }

    pub fn clear_history(&self)  {
        self.hist.borrow_mut().rez.clear();
    }

    fn create_handler_for_all_outs(&mut self,
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

    println!(">>>>>>>>>>>>>>><<<<<<<<<<<<<<<<<");
    let inp = vec![
        (1, UnifiedToken::from_val(10.0)),
    ];
    exec.run_tick(inp);
    let current_hist =consumer_fact.get_current_hist();
    assert!( current_hist.is_empty());
    println!(">>>>>>>>>>>>>>><<<<<<<<<<<<<<<<<");

    let inp = vec![
        (2, UnifiedToken::from_val(0.0)),
    ];
    exec.run_tick(inp);
    let current_hist =consumer_fact.get_current_hist();
    assert_eq!(vec![(0, UnifiedToken::Exist(10.0))], current_hist);
    consumer_fact.clear_history();



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

/*
#[test]
fn concurent_petri() {
        let ww = my_file_read("inputs/ConcurentPetriNet.json");
        let rez = deseralize(&ww).unwrap();
        let (net, mut man) = rez.build();
        let mut consumer_fact = ConsumerFactory::new();
        consumer_fact.create_handler_for_all_outs(&net, &mut man, );

        let mut exc =SynchronousFuzzyPetriExecutor::new(&net, man);

        exc.run_tick(vec![]);
        let rez = consumer_fact.get_current_hist();
        assert_eq!(0, rez.len());

        exc.run_tick(vec![(3, FuzzyToken::zero_token())]);
        let rez = consumer_fact.get_current_hist();
        assert_eq!(1, rez.len());
        assert_eq!((2, FuzzyToken::Exist([0.0, 0.0, 1.0, 0.0, 0.0])), rez[0]);
        consumer_fact.clear_history();
}

#[test]
fn maximum_finder() {
        let ww = my_file_read("inputs/MaximumFinder.json");
        let rez = deseralize(&ww).unwrap();
        let (net, mut man) = rez.build();
        let mut consumer_fact = ConsumerFactory::new();
        consumer_fact.create_handler_for_all_outs(&net, &mut man, );

        let mut exc =SynchronousFuzzyPetriExecutor::new(&net, man);
        let fuzz = TriangleFuzzyfier::default();

        let inp = vec![
            (0, fuzz.fuzzyfy(Some(-0.5))),
            (1, fuzz.fuzzyfy(Some(0.0))),
            (2, fuzz.fuzzyfy(Some(0.5))),
        ];

        exc.run_tick(inp);
        let rez = consumer_fact.get_current_hist();
        assert_eq!(1, rez.len());
        assert_eq!((2, FuzzyToken::Exist([0.0, 0.0, 1.0, 0.0, 0.0])), rez[0]);
        consumer_fact.clear_history();

        let inp = vec![
            (0, fuzz.fuzzyfy(Some(0.5))),
            (1, fuzz.fuzzyfy(Some(0.0))),
            (2, fuzz.fuzzyfy(Some(0.2))),
        ];

        exc.run_tick(inp);
        let rez = consumer_fact.get_current_hist();
        assert_eq!(1, rez.len());
        assert_eq!((0, FuzzyToken::Exist([0.0, 0.0, 1.0, 0.0, 0.0])), rez[0]);
        consumer_fact.clear_history();

        let inp = vec![
            (0, fuzz.fuzzyfy(Some(0.5))),
            (1, fuzz.fuzzyfy(Some(1.0))),
            (2, fuzz.fuzzyfy(Some(0.2))),
        ];

        exc.run_tick(inp);
        let rez = consumer_fact.get_current_hist();
        assert_eq!(1, rez.len());
        assert_eq!((1, FuzzyToken::Exist([0.0, 0.0, 1.0, 0.0, 0.0])), rez[0]);
        consumer_fact.clear_history();
}

#[test]
fn comprator(){
        let ww = my_file_read("inputs/Comparator.json");
        let rez = deseralize(&ww).unwrap();
        let (net, mut man) = rez.build();

        let mut consumer_fact = ConsumerFactory::new();
        consumer_fact.create_handler_for_all_outs(&net, &mut man, );

        let mut exc =SynchronousFuzzyPetriExecutor::new(&net, man);
        let fuzz = TriangleFuzzyfier::default();

        let inp = vec![
            (0, fuzz.fuzzyfy(Some(-0.5))),
            (1, fuzz.fuzzyfy(Some(0.0))),
        ];

        exc.run_tick(inp);
        let rez = consumer_fact.get_current_hist();
        assert_eq!(1, rez.len());
        assert_eq!((2, FuzzyToken::Exist([0.0, 0.0, 1.0, 0.0, 0.0])), rez[0]);
        consumer_fact.clear_history();


        let inp = vec![
            (0, fuzz.fuzzyfy(Some(0.5))),
            (1, fuzz.fuzzyfy(Some(0.0))),
        ];
        exc.run_tick(inp);
        let rez = consumer_fact.get_current_hist();
        assert_eq!(1, rez.len());
        assert_eq!((3, FuzzyToken::Exist([0.0, 0.0, 1.0, 0.0, 0.0])), rez[0]);
        consumer_fact.clear_history();
}

#[test]
fn loop_petri(){
        let ww = my_file_read("inputs/Loop.json");
        let rez = deseralize(&ww).unwrap();
        let (net, mut man) = rez.build();
        println!("{:?}", net.typed_table_for_trans(2));

        let mut consumer_fact = ConsumerFactory::new();
        consumer_fact.create_handler_for_all_outs(&net, &mut man, );

        let mut exc =SynchronousFuzzyPetriExecutor::new(&net, man);
        let fuzz = TriangleFuzzyfier::default();

        let inp = vec![
            (0, fuzz.fuzzyfy(Some(-0.25))),
        ];
        exc.run_tick(inp);
        let rez = consumer_fact.get_current_hist();
        assert_eq!(1, rez.len());
        assert_eq!((0, FuzzyToken::Exist([0.00,1.00,0.00,0.00,0.00])), rez[0]);
        consumer_fact.clear_history();

        let inp = vec![
            (0, fuzz.fuzzyfy(Some(0.78))),
        ];
        exc.run_tick(inp);
        let rez = consumer_fact.get_current_hist();
        assert_eq!(1, rez.len());
        assert_eq!((0, FuzzyToken::Exist([0.0, 0.0, 0.32000017, 0.6799998, 0.0])), rez[0]);
        consumer_fact.clear_history();


}
*/
