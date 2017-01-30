
use petri_net::petri_net_builder::{FuzzyPetriNet, EventManager};
use basic::*;
use std::mem;



struct BasicFuzzyPetriExecutor<'a> {
    net: &'a FuzzyPetriNet,
    event_manager: EventManager,
    place_state: Vec<FuzzyToken>,
    trans_state: Vec<i32>,
    trans_holds: Vec<Vec<FuzzyToken>>,
    trans_order: Vec<usize>,
}

impl<'a> BasicFuzzyPetriExecutor<'a> {

    pub fn new(net : &FuzzyPetriNet, men: EventManager) -> BasicFuzzyPetriExecutor {
        BasicFuzzyPetriExecutor{
            net: net,
            event_manager: men,
            place_state: init_place_state(net),
            trans_state: vec![0; net.get_trans_nr()],
            trans_holds: vec![vec![]; net.get_trans_nr()],
            trans_order: order_of_transitions(net),
        }
    }



    pub fn put_tokens_to_inp_places(&mut self, inps: Vec<(usize, FuzzyToken)>) {
        for (pl_id, tk) in inps.into_iter() {
            self.place_state[pl_id].unite(tk);
        }
    }

    pub fn execute_firable_transitions(&mut self) {
        let mut loop_cntr= 0;
        let max_loop = 40;
        let mut heappened_something = true;
        while heappened_something && loop_cntr < max_loop {
            heappened_something = false;
            loop_cntr += 1;
            for index in 0.. self.trans_order.len(){
                let tr_id = self.trans_order[index];
                match self.is_fireable(tr_id){
                    None => {/*does nothing */},
                    Some(inps) => {
                        heappened_something = true;
                        self.start_fire(tr_id, inps);
                    }
                }
            }
        }
    }

    pub fn update_delay_state(&mut self) {
        for tr_id in 0..self.net.get_trans_nr() {
            if self.trans_state[tr_id] > 0 {
                if self.trans_state[tr_id] == 1{
                    self.finish_fire(tr_id);
                }
                self.trans_state[tr_id] -= 1;
            }
        }
    }

    fn is_fireable(&self, tr_id: usize) -> Option<Vec<FuzzyToken>>{
        if self.trans_state[tr_id] != 0{
            return None;
        }
        let inp_tokens = self.get_inp_token(tr_id);
        if self.net.table_for_trans(tr_id).is_executable(&inp_tokens) {
            Some(inp_tokens)
        } else {None}
    }

    fn start_fire(&mut self, tr_id: usize, inp_tokens: Vec<FuzzyToken> ) {
        self.clear_inp_tokens(tr_id);
        let rez = self.net.table_for_trans(tr_id).execute(inp_tokens) ;
        self.trans_holds[tr_id] = rez;
        let delay = self.net.get_delay(tr_id);
        if delay == 0 {
            self.finish_fire(tr_id);
        } else {
            self.trans_state[tr_id] = delay;
        }

    }

    fn finish_fire(&mut self, tr_id: usize) {
        let mut rez = mem::replace(&mut self.trans_holds[tr_id], vec![]);
        if self.net.is_trans_out(tr_id) {
            self.event_manager.execute_handler(tr_id,
                                               mem::replace(&mut rez[0], FuzzyToken::Phi));
            return;
        }
        let out_places = self.net.get_places_after_trans(tr_id);
        if rez.len() != out_places.len() {
            panic!("wrong number of ouputs fot tr{}, it has {} from table but {} out places",
                   tr_id, rez.len(), out_places.len());
        }
        for i in 0..rez.len() {
             self.place_state[out_places[i]].unite(mem::replace(&mut rez[i], FuzzyToken::Phi));
        }
    }


    fn clear_inp_tokens(&mut self, tr_id: usize) {
        let inp_places = self.net.get_places_befor_trans(tr_id);
        for place in inp_places {
            self.place_state[place] = FuzzyToken::Phi;
        }
    }

    fn get_inp_token(&self, tr_id: usize) -> Vec<FuzzyToken> {
        let mut to_ret = vec![];
        let inp_places = self.net.get_places_befor_trans(tr_id);
        for place in inp_places {
            let ft = self.place_state[place].clone();
            let modified  = match ft {
                FuzzyToken::Phi => FuzzyToken::Phi,
                _ => {
                    let weight = self.net.get_weigth_for_arc(place, tr_id);
                    let val = TriangleFuzzyfier::default().defuzzyfy(ft)
                        .expect("Phi token after multiplication");
                    TriangleFuzzyfier::default().fuzzyfy(Some(val*weight))
                }
            };
            to_ret.push(modified);
        }
        to_ret
    }
}

fn init_place_state(net: &FuzzyPetriNet) -> Vec<FuzzyToken>{
    let mut to_ret = vec![];
    for place_id in 0..net.get_place_nr(){
        to_ret.push(net.get_initial_marking(place_id));
    }
    to_ret
}

fn order_of_transitions(net: &FuzzyPetriNet) -> Vec<usize> {
    let mut inp_trs  = vec![];
    let mut out_trs  = vec![];
    let mut nondelay_trs = vec![];
    let mut delays_trs = vec![];
    for tr_id in 0..net.get_trans_nr() {
        let places_needed = net.get_places_befor_trans(tr_id);
        let mut found = false ;
        for pl_id in places_needed {
            if net.is_place_inp(pl_id) {
                inp_trs.push(tr_id);
                found = true;
                break;
            }
        }
        if ! found {
            if net.is_trans_out(tr_id) {
                out_trs.push(tr_id);
                break;
            }

            if net.get_delay(tr_id) == 0{
                nondelay_trs.push(tr_id);
            } else {
                delays_trs.push(tr_id);

            }
        }
    }
    inp_trs.append(&mut out_trs);
    inp_trs.append(&mut nondelay_trs);
    inp_trs.append(&mut delays_trs);
    inp_trs
}


pub struct SynchronousFuzzyPetriExecutor<'a>{
    basic : BasicFuzzyPetriExecutor<'a>,
}

impl<'a> SynchronousFuzzyPetriExecutor<'a> {
    pub fn new(net: &FuzzyPetriNet, men: EventManager) -> SynchronousFuzzyPetriExecutor {
        SynchronousFuzzyPetriExecutor {
            basic: BasicFuzzyPetriExecutor::new(net, men),
        }
    }

    pub fn run_tick(&mut self, inps: Vec<(usize, FuzzyToken)>) {
        self.basic.put_tokens_to_inp_places(inps);
        self.basic.update_delay_state();
        self.basic.execute_firable_transitions();
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;
    use basic::*;
    use tables::*;
    use petri_net::petri_net_builder::*;
    use std::cell::RefCell;
    use std::rc::Rc;



    #[test]
    fn order_of_transitions_test() {
        let (net, _, _) = simple_delay_net();
        let rez = super::order_of_transitions(&net);
        assert_eq!(rez, vec![0,2,1]);
    }


    #[test]
    fn SynchronousFuzzyPetriExecutor_simple_delay_net() {
        let  (net, man, cons_fact) = simple_delay_net();

        let mut exc =SynchronousFuzzyPetriExecutor::new(&net, man);
        let inp = vec![(2, FuzzyToken::zero_token())];
        exc.run_tick(inp);

        let rez = cons_fact.get_current_hist();
        assert_eq!(0, rez.len());

        let inp = vec![];
        exc.run_tick(inp);

        let rez = cons_fact.get_current_hist();
        assert_eq!(1, rez.len());
        assert_eq!((2, FuzzyToken::Exist([0.0, 0.0, 1.0, 0.0, 0.0])), rez[0]);

        cons_fact.clear_history();
        let inp = vec![(2, FuzzyToken::from_arr([1.0, 0.0, 0.0, 0.0, 0.0]))];
        exc.run_tick(inp);
        let rez = cons_fact.get_current_hist();
        assert_eq!(0, rez.len());

        let inp = vec![];
        exc.run_tick(inp);

        let rez = cons_fact.get_current_hist();
        assert_eq!(1, rez.len());
        assert_eq!((2, FuzzyToken::Exist([0.0, 1.0, 0.0, 0.0, 0.0])), rez[0]);
    }

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

    }

    pub fn simple_delay_net() -> (FuzzyPetriNet, EventManager, ConsumerFactory) {
        let mut bld = FuzzyPetriNetBuilder::new();

        let p0 = bld.add_place();
        let p1 = bld.add_place();
        let p2_inp = bld.add_inp_place();
        let p3 = bld.add_place();
        let t0 = bld.add_trans(0, FuzzyTableE::txo(TwoXOneTable::default_table()));
        let t1 = bld.add_trans(1, FuzzyTableE::oxt(OneXTwoTable::default_table()));
        let t2_out = bld.add_out_trans(FuzzyTableE::oxo(OneXOneTable::default_table()));

        bld.add_arc_from_place_to_trans(p2_inp, t0, 1.0);
        bld.add_arc_from_place_to_trans(p0, t0, 1.0);
        bld.add_arc_from_trans_to_place(t0, p1);
        bld.add_arc_from_place_to_trans(p1, t1, 1.0);
        bld.add_arc_from_trans_to_place(t1, p0);
        bld.add_arc_from_trans_to_place(t1, p3);
        bld.add_arc_from_place_to_trans(p3, t2_out, 1.0);
        bld.set_initial_token(p0, FuzzyToken::zero_token());

        let mut consumer_factory = ConsumerFactory::new();
        let (net, mut event_manager) = bld.build();
        event_manager.add(t2_out, consumer_factory.create_handler_for(t2_out));

        (net, event_manager, consumer_factory)
    }
}
