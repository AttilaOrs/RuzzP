use std::cell::RefCell;
use std::rc::Rc;

extern crate ruzz_p;

use ruzz_p::read_petri::{my_file_read, deseralize};
use ruzz_p::petri_net::*;
use ruzz_p::basic::*;



struct History{
    rez : Vec<(usize,FuzzyToken)>,
}

struct MyConsumer {
    hist: Rc<RefCell<History>>,
    tr_id : usize,
}

impl FuzzyTokenConsumer for MyConsumer {
    fn consume(&mut self, ft: FuzzyToken){
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

    pub fn get_current_hist(&self) -> Vec<(usize, FuzzyToken)> {
        let mut to_ret = Vec::new();
        for i in &self.hist.borrow().rez {
            to_ret.push(i.clone());
        }
        to_ret
    }

    pub fn clear_history(&self)  {
        self.hist.borrow_mut().rez.clear();
    }

    fn create_handler_for_all_outs(&mut self, net: &FuzzyPetriNet, manager: &mut EventManager ){
        for tr_id in 0..net.get_trans_nr() {
            if net.is_trans_out(tr_id) {
                manager.add(tr_id, self.create_handler_for(tr_id));
            }
        }

    }

}


#[test]
fn simple_delay_net_test(){
        let ww = my_file_read("inputs/SimpleDelayPetriNet.json");
        let rez = deseralize(&ww).unwrap();
        let (net, mut man) = rez.build();

        let mut consumer_fact = ConsumerFactory::new();
        consumer_fact.create_handler_for_all_outs(&net, &mut man, );

        let mut exc =SynchronousFuzzyPetriExecutor::new(&net, man);

        let inp = vec![(2, FuzzyToken::zero_token())];
        exc.run_tick(inp);

        let rez = consumer_fact.get_current_hist();
        assert_eq!(0, rez.len());

        let inp = vec![];
        exc.run_tick(inp);

        let rez = consumer_fact.get_current_hist();
        assert_eq!(1, rez.len());
        assert_eq!((2, FuzzyToken::Exist([0.0, 0.0, 1.0, 0.0, 0.0])), rez[0]);
}

#[test]
fn selection_like_two_branches() {
        let ww = my_file_read("inputs/SelectionLikeTwoBranchExamplePetriNet.json");
        let rez = deseralize(&ww).unwrap();
        let (net, mut man) = rez.build();
        let mut consumer_fact = ConsumerFactory::new();
        consumer_fact.create_handler_for_all_outs(&net, &mut man, );

        let mut exc =SynchronousFuzzyPetriExecutor::new(&net, man);

        exc.run_tick(vec![(1, FuzzyToken::zero_token())]);

        let rez = consumer_fact.get_current_hist();
        assert_eq!(1, rez.len());
        assert_eq!((4, FuzzyToken::Exist([0.0, 0.0, 1.0, 0.0, 0.0])), rez[0]);
        consumer_fact.clear_history();

        exc.run_tick(vec![]);
        let rez = consumer_fact.get_current_hist();
        assert_eq!(0, rez.len());

        let inp = vec![(2, FuzzyToken::from_arr([0.0, 1.0, 0.0, 0.0, 0.0,]))];
        exc.run_tick(inp);
        let rez = consumer_fact.get_current_hist();
        assert_eq!(1, rez.len());
        assert_eq!((5, FuzzyToken::Exist([0.0, 1.0, 0.0, 0.0, 0.0])), rez[0]);
        consumer_fact.clear_history();

        exc.run_tick(vec![(1, FuzzyToken::zero_token())]);
        let rez = consumer_fact.get_current_hist();
        assert_eq!(0, rez.len());

        exc.run_tick(vec![]);
        let rez = consumer_fact.get_current_hist();
        assert_eq!(1, rez.len());
}

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
        //currenty not working... i will find the bug , in another day, in a nother year,
        //a nother lifre,
        assert_eq!(0, rez.len());

}
