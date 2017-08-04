
use tables::*;
use basic::*;
use std::collections::HashMap;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum UnifiedTableE{
    oxo(UnifiedOneXOneTable),
    oxt(UnifiedOneXTwoTable),
    txo(UnifiedTwoXOneTable),
    txt(UnifiedTwoXTwoTable),
}

impl UnifiedTableE {
    pub fn get_table(&self) -> &ExecutableUnifiedTable {
        match *self {
          UnifiedTableE::oxo(ref t) => t,
          UnifiedTableE::oxt(ref t) => t,
          UnifiedTableE::txo(ref t) => t,
          UnifiedTableE::txt(ref t) => t,
        }
    }
}

pub struct Trans{
    delay: i32,
    table: UnifiedTableE,
    is_out: bool,
    before_places: Vec<usize>,
    after_places: Vec<usize>,
}

impl Trans {
    pub fn new(delay : i32, table: UnifiedTableE, out : bool) -> Trans{
         Trans{
            delay: delay,
            is_out: out,
            table: table,
            before_places : vec![],
            after_places : vec![],
        }

    }
}

pub struct Place{
    scale: f32,
    is_inp: bool,
    inital_marking: UnifiedToken,
    before_transition: Vec<usize>,
    after_transition: Vec<usize>,
}

impl Place {
    pub fn new(scale: f32, inp: bool) -> Place {
        Place {
            scale: scale,
            is_inp :inp,
            inital_marking : UnifiedToken::Phi,
            before_transition : vec![],
            after_transition : vec![],
        }
    }

}

pub trait UnifiedTokenConsumer : Send{
    fn consume(&mut self, ft: UnifiedToken);
}

pub struct EventManager {
    handlers: HashMap<usize, Vec<Box<UnifiedTokenConsumer>>>,
}

impl EventManager {
    pub fn new() -> EventManager {
        EventManager {
            handlers: HashMap::new(),
        }
    }

    pub fn add(&mut self, tr_id: usize, f :Box<UnifiedTokenConsumer>) {
        self.handlers.entry(tr_id).or_insert(vec![]).push(f);
    }

    pub fn execute_handler(&mut self, tr_id: usize, ft: UnifiedToken) {
        if let Some(my_handlers) = self.handlers.get_mut(&tr_id) {
            for f in (*my_handlers).iter_mut() {
                f.consume(ft.clone());
            }
        }
    }
}

pub struct UnifiedPetriNet {
    transitions : Vec<Trans>,
    places : Vec<Place>,
}

pub struct UnifiedPetriNetBuilder {
    transitions : Vec<Trans>,
    places : Vec<Place>,
    manager: EventManager,
}

impl UnifiedPetriNetBuilder {

    pub fn new() -> UnifiedPetriNetBuilder {
        UnifiedPetriNetBuilder {
            transitions : vec![],
            places: vec![],
            manager: EventManager::new(),
        }
    }

    pub fn add_transition(&mut self, delay : i32,  table: UnifiedTableE ) -> usize {
        self.transitions.push(Trans::new(delay, table, false ));
        self.transitions.len() - 1
    }

    pub fn add_out_transition(&mut self,  table: UnifiedTableE ) -> usize {
        match table {
            UnifiedTableE::oxo(_) => {/* nothing */},
            _ => panic!("wrong table type for out transition")
        }
        self.transitions.push(Trans::new(0, table, true));
        self.transitions.len() - 1
    }

    pub fn add_place(&mut self,  scale: f32 ) -> usize {
        self.places.push(Place::new(scale, false ));
        self.places.len() -1
    }

    pub fn add_inp_place(&mut self,  scale: f32 ) -> usize {
        self.places.push(Place::new(scale, true ));
        self.places.len() -1
    }

    pub fn connect_transition_with_place(&mut self, tr : usize, pl : usize)  {
        self.transitions[tr].after_places.push(pl);
        self.places[pl].before_transition.push(tr);
    }

    pub fn connect_place_with_transition(&mut self,  pl : usize, tr: usize)  {
        self.transitions[tr].before_places.push(pl);
        self.places[pl].after_transition.push(tr);
    }

    pub fn set_innitial_marking(&mut self, pl: usize, token : UnifiedToken ) {
        self.places[pl].inital_marking = token;
    }

    pub fn add_action_for_out_trans(&mut self, tr_id: usize, f : Box<UnifiedTokenConsumer>) {
        self.manager.add(tr_id, f);
    }

    pub fn build(self) ->(UnifiedPetriNet, EventManager) {
        let net = UnifiedPetriNet{
            places : self.places,
            transitions : self.transitions,
        };
        (net, self.manager)
    }
}

impl UnifiedPetriNet {

    #[inline]
    pub fn get_place_nr(&self ) -> usize {
        self.places.len()
    }

    #[inline]
    pub fn get_trans_nr(&self ) -> usize {
        self.transitions.len()
    }

    #[inline]
    pub fn is_trans_out(&self, tr_id: usize) -> bool {
        self.transitions[tr_id].is_out
    }

    #[inline]
    pub fn is_place_inp(&self, pl_id: usize) -> bool {
        self.places[pl_id].is_inp
    }

    #[inline]
    pub fn get_place_scale(&self, pl_id: usize) -> f32 {
        self.places[pl_id].scale
    }

    #[inline]
    pub fn table_for_trans(&self, tr_id: usize) ->&ExecutableUnifiedTable {
        self.transitions[tr_id].table.get_table()
    }

    #[inline]
    pub fn typed_table_for_trans(&self, tr_id: usize) ->&UnifiedTableE {
        &self.transitions[tr_id].table
    }

    #[inline]
    pub fn get_places_after_trans(&self, tr_id: usize) -> &Vec<usize>{
        &self.transitions[tr_id].after_places
    }

    #[inline]
    pub fn get_tanss_after_place(&self, pl_id: usize) -> &Vec<usize>{
        &self.places[pl_id].after_transition
    }

    #[inline]
    pub fn get_places_befor_trans(&self, tr_id: usize) -> &Vec<usize> {
        &self.transitions[tr_id].before_places
    }

    #[inline]
    pub fn get_initial_marking(&self, pl_id: usize) -> UnifiedToken {
        self.places[pl_id].inital_marking.clone()
    }

    #[inline]
    pub fn get_delay(&self, tr_id: usize) -> i32 {
        self.transitions[tr_id].delay
    }

}
