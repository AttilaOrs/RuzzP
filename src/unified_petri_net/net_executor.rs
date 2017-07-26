
extern crate fnv;




use unified_petri_net::net_builder::{UnifiedPetriNet, EventManager};
use basic::*;
use std::mem;
use std::collections::HashMap;
use self::fnv::FnvHasher;
use std::hash::BuildHasherDefault;



type MyHasher = BuildHasherDefault<FnvHasher>;

struct BasicUnifiedPetriExecutor<'a> {
    net: &'a UnifiedPetriNet,
    event_manager: EventManager,
    place_state: Vec<UnifiedToken>,
    trans_state: Vec<i32>,
    trans_holds: Vec<Vec<UnifiedToken>>,
    trans_order: Vec<usize>,
    scales : Vec<TriangleFuzzyfier>,
    cached_possibly_exec : HashMap<Vec<bool>, Vec<usize>,MyHasher>,
}

impl<'a> BasicUnifiedPetriExecutor<'a> {

    pub fn new(net : &UnifiedPetriNet, men: EventManager) -> BasicUnifiedPetriExecutor {
        BasicUnifiedPetriExecutor{
            net: net,
            event_manager: men,
            place_state: init_place_state(net),
            trans_state: vec![0; net.get_trans_nr()],
            trans_holds: vec![vec![]; net.get_trans_nr()],
            trans_order: order_of_transitions(net),
            scales :init_scales(net),
            cached_possibly_exec: HashMap::default(),
        }
    }

    pub fn put_tokens_to_inp_places(&mut self, inps: Vec<(usize, UnifiedToken)>) {
        for (pl_id, tk) in inps.into_iter() {
            self.place_state[pl_id].unite(tk);
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


    pub fn execute_firable_transitions(&mut self) {
        let mut loop_cntr= 0;
        let max_loop = 40;
        let mut heappened_something = true;
        while heappened_something && loop_cntr < max_loop {
            heappened_something = false;
            loop_cntr += 1;
            let pos ={self.get_possible_executable_trans()};
            for tr_id in pos{
                match self.is_fireable(tr_id){
                    None => {/*does notin*/},
                    Some(inps) => {
                        heappened_something = true;
                        self.start_fire(tr_id, inps);
                        break;
                    }
                }
            }
        }

    }

    fn finish_fire(&mut self, tr_id: usize) {
        let mut rez = mem::replace(&mut self.trans_holds[tr_id], vec![]);
        if self.net.is_trans_out(tr_id) {
            self.event_manager.execute_handler(tr_id,
                                               mem::replace(&mut rez[0], UnifiedToken::Phi));
        } else {
            let out_places = self.net.get_places_after_trans(tr_id);
            if rez.len() != out_places.len() {
                panic!("wrong number of ouputs fot tr{},
                       it has {} from table but {} out places",
                       tr_id, rez.len(), out_places.len());
            }
            for i in 0..rez.len() {
                 self.place_state[out_places[i]].unite(
                     mem::replace(&mut rez[i], UnifiedToken::Phi));
            }
        }


    }

    fn start_fire(&mut self, tr_id: usize, inp_tokens: Vec<UnifiedToken>) {

        self.clear_inp_tokens(tr_id);
        let rez ;
        {
            let defuzz = self.get_out_scales(tr_id);
            let fuzz = self.get_inp_scales(tr_id);
            rez = self.net.table_for_trans(tr_id).execute(inp_tokens, &fuzz, &defuzz) ;
        }
        self.trans_holds[tr_id] = rez;
        let delay = self.net.get_delay(tr_id);
        if delay == 0 {
            self.finish_fire(tr_id);
        } else {
            self.trans_state[tr_id] = delay;
        }
    }


    fn get_inp_token(&self, tr_id: usize) -> Vec<UnifiedToken> {
        let mut to_ret = vec![];
        let inp_places = self.net.get_places_befor_trans(tr_id);
        for place in inp_places {
            let ft = self.place_state[*place].clone();
            to_ret.push(ft);
        }
        to_ret
    }

    fn clear_inp_tokens(&mut self, tr_id: usize)  {
        let inp_places = self.net.get_places_befor_trans(tr_id);
        for place in inp_places {
            self.place_state[*place] = UnifiedToken::Phi;
        }
    }

    fn is_fireable(&self, tr_id: usize) -> Option<Vec<UnifiedToken>>{
        if self.trans_state[tr_id] != 0{
            return None;
        }
        let inp_tokens = self.get_inp_token(tr_id);
        let fuzz = self.get_inp_scales(tr_id);
        if self.net.table_for_trans(tr_id).is_executable(&inp_tokens, &fuzz) {
            Some(inp_tokens)
        } else {None}
    }

    fn get_inp_scales(&self, tr_id: usize) -> Vec<&Fuzzyfier>{
        let mut to_ret  : Vec<&Fuzzyfier> = vec![];
        for pl_id  in self.net.get_places_befor_trans(tr_id) {
            to_ret.push(&self.scales[*pl_id]);
        }
        to_ret
    }

    fn get_out_scales(&self, tr_id: usize) -> Vec<&Defuzzyfier>{
        let mut to_ret  : Vec<&Defuzzyfier> = vec![];
        if !self.net.is_trans_out(tr_id) {
            for pl_id  in self.net.get_places_after_trans(tr_id) {
                to_ret.push(&self.scales[*pl_id]);
            }
        } else {
            for pl_id  in self.net.get_places_befor_trans(tr_id) {
                to_ret.push(&self.scales[*pl_id]);
            }
        }
        to_ret
    }
    fn get_possible_executable_trans(&mut self) -> Vec<usize>{
        let sm = self.simplyfied_marking();
        if !self.cached_possibly_exec.contains_key(&sm) {
            let v = self.possibly_executable_trans(&sm);
            self.cached_possibly_exec.insert(sm.clone(), v);
        }
        return self.cached_possibly_exec.get(&sm).unwrap().clone()

    }

    fn simplyfied_marking(&self) -> Vec<bool>{
        self.place_state.iter().map(|x| x.not_phi()).collect()
    }

    fn possibly_executable_trans(&self, simpl_mark : &Vec<bool>) -> Vec<usize> {
        self.trans_order.iter()
            .map(|x|  (x,self.net.get_places_befor_trans(*x).iter().map(|pl| simpl_mark[*pl]).collect::<Vec<bool>>()))
            .filter(|&(ref tr_nr, ref inp)| self.net.table_for_trans(**tr_nr).possibly_executable(&inp))
            .map(|(tr_nr, _)| (*tr_nr).clone())
            .collect()
    }

}

fn init_scales(net: &UnifiedPetriNet) ->Vec<TriangleFuzzyfier> {
    let mut to_ret =vec![];
    for place_id in 0..net.get_place_nr() {
        let scale = net.get_place_scale(place_id);
        to_ret.push(TriangleFuzzyfier::with_min_max(-1.0*scale, scale));
    }
    to_ret
}
fn init_place_state(net: &UnifiedPetriNet) -> Vec<UnifiedToken>{
    let mut to_ret = vec![];
    for place_id in 0..net.get_place_nr(){
        to_ret.push(net.get_initial_marking(place_id));
    }
    to_ret
}

fn order_of_transitions(net: &UnifiedPetriNet) -> Vec<usize> {
    let mut inp_trs  = vec![];
    let mut out_trs  = vec![];
    let mut nondelay_trs = vec![];
    let mut delays_trs = vec![];
    for tr_id in 0..net.get_trans_nr() {
        let places_needed = net.get_places_befor_trans(tr_id);
        let mut found = false ;
        for pl_id in places_needed {
            if net.is_place_inp(*pl_id) {
                inp_trs.push(tr_id);
                found = true;
                break;
            }
        }
        if ! found {
            if net.is_trans_out(tr_id) {
                out_trs.push(tr_id);
                continue;
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


pub struct SynchronousUnifiedPetriExecutor<'a>{
    basic : BasicUnifiedPetriExecutor<'a>,
}

impl<'a> SynchronousUnifiedPetriExecutor<'a> {
    pub fn new(net: &UnifiedPetriNet, men: EventManager) -> SynchronousUnifiedPetriExecutor {
        SynchronousUnifiedPetriExecutor {
            basic: BasicUnifiedPetriExecutor::new(net, men),
        }
    }

    pub fn run_tick(&mut self, inps: Vec<(usize, UnifiedToken)>) {
        self.basic.put_tokens_to_inp_places(inps);
        self.basic.update_delay_state();
        self.basic.execute_firable_transitions();
    }
}


#[cfg(test)]
mod tests {

    use super::*;
    use tables::*;
    use unified_petri_net::net_builder::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct History{
        rez : Vec<(usize,UnifiedToken)>,
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

    }

    pub fn simple_delay_net() -> (UnifiedPetriNet, EventManager, ConsumerFactory) {
        let mut bld = UnifiedPetriNetBuilder::new();

        let ip0 = bld.add_inp_place(1.0);
        let p0 = bld.add_place(1.0);
        bld.set_innitial_marking(p0, UnifiedToken::from_val(0.0));
        let t0 = bld.add_transition(1,UnifiedTableE::txt(UnifiedTwoXTwoTable::default_table()));
        bld.connect_place_with_transition(p0, t0);
        bld.connect_place_with_transition(ip0, t0);

        let p1 = bld.add_place(1.0);
        bld.connect_transition_with_place(t0,p1);

        let p2 = bld.add_place(1.0);
        bld.connect_transition_with_place(t0,p2);

        let ot2 = bld.add_out_transition(
            UnifiedTableE::oxo(UnifiedOneXOneTable::default_table()));
        bld.connect_place_with_transition(p1, ot2);

        let t1 = bld.add_transition(0,
                                    UnifiedTableE::oxo(UnifiedOneXOneTable::default_table()));
        bld.connect_place_with_transition(p2,t1);
        bld.connect_transition_with_place(t1, p0);


        let mut consumer_factory = ConsumerFactory::new();
        let (net, mut event_manager) = bld.build();
        event_manager.add(ot2, consumer_factory.create_handler_for(ot2));

        (net, event_manager, consumer_factory)

    }

    #[test]
    fn simple_delay_net_test(){
       let (net, event_manager, cons_fact) =simple_delay_net();
       let mut exec = SynchronousUnifiedPetriExecutor::new(&net, event_manager);

       let inp = vec![(0, UnifiedToken::from_val(0.0))];
       exec.run_tick(inp);

       let rez = cons_fact.get_current_hist();
       assert!(rez.len() == 0);

       let inp = vec![];
       exec.run_tick(inp);

       let rez = cons_fact.get_current_hist();
       assert_eq!(vec![(1,UnifiedToken::Exist(0.0))],rez);
    }


}
