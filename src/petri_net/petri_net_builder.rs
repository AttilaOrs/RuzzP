use std::mem;
use std::collections::HashMap;

use tables::*;
use basic::*;

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum FuzzyTableE{
    oxo(OneXOneTable),
    oxt(OneXTwoTable),
    txo(TwoXOneTable),
    txt(TwoXTwoTable),
}

impl FuzzyTableE {
    fn get_table(&self) -> &ExecutableFuzzyTable {
        match *self {
            FuzzyTableE::oxo(ref t) =>  t,
            FuzzyTableE::oxt(ref t) =>  t,
            FuzzyTableE::txo(ref t) =>  t,
            FuzzyTableE::txt(ref t) =>  t,
        }
    }
}

pub struct ArcFromTransToPlace{
    trans_id: usize,
    place_id: usize,
}

pub struct ArcFromPlaceToTrans{
    place_id: usize,
    trans_id: usize,
    weigth: f32,
}

pub struct Trans {
    delay: i32,
    table: FuzzyTableE,
    is_out: bool,
}

struct Place {
    inital_marking: FuzzyToken,
    is_inp: bool,
}

pub struct FuzzyPetriNetBuilder {
    arcs_from_trans: Vec<ArcFromTransToPlace>,
    arcs_from_place: Vec<ArcFromPlaceToTrans>,
    transs : Vec<Trans>,
    places: Vec<Place>,
    trans_cntr: usize,
    place_cntr: usize,
    event_manager: EventManager
}

impl FuzzyPetriNetBuilder {
    pub fn new() -> FuzzyPetriNetBuilder {
        FuzzyPetriNetBuilder{
            arcs_from_place : vec![],
            arcs_from_trans : vec![],
            transs : vec![],
            places : vec![],
            trans_cntr: 0,
            place_cntr: 0,
            event_manager : EventManager::new(),
        }
    }

    pub fn add_trans(&mut self,delay: i32, table: FuzzyTableE) -> usize {
        let id = self.trans_cntr;
        self.trans_cntr += 1 ;
        let tr = Trans {
            delay: delay,
            table: table,
            is_out : false,
        };
        self.transs.push(tr);
        return id
    }

    pub fn add_out_trans(&mut self, table: FuzzyTableE) -> usize {
        match table {
            FuzzyTableE::oxo(_) =>{/**/},
            _ =>  unreachable!(),
        };

        let id = self.trans_cntr;
        self.trans_cntr += 1 ;
        let tr = Trans {
            delay: 0,
            table: table,
            is_out: true,
        };
        self.transs.push(tr);
        return id
    }

    pub fn add_action_for_out_trans(&mut self, tr_id: usize, f : Box<FuzzyTokenConsumer>) {
        self.event_manager.add(tr_id, f);
    }

    pub fn add_place(&mut self) -> usize {
        let id = self.place_cntr;
        self.place_cntr += 1;
        let pl=  Place {
            inital_marking: FuzzyToken::Phi,
            is_inp: false,
        };
        self.places.push(pl);
        id
    }

    pub fn add_inp_place(&mut self) -> usize {
        let id = self.place_cntr;
        self.place_cntr += 1;
        let pl=  Place {
            inital_marking: FuzzyToken::Phi,
            is_inp: true,
        };
        self.places.push(pl);
        id
    }

    pub fn set_initial_token(&mut self, place_id: usize ,token: FuzzyToken) {
        mem::replace(&mut self.places[place_id].inital_marking, token);
    }

    pub fn add_arc_from_trans_to_place(&mut self, trans_id: usize, place_id: usize ) {
        self.arcs_from_trans.push(ArcFromTransToPlace{
            trans_id : trans_id,
            place_id : place_id,
        })
    }

    pub fn add_arc_from_place_to_trans(&mut self, place_id: usize, trans_id: usize, weigth:f32) {
        self.arcs_from_place.push(ArcFromPlaceToTrans{
            trans_id : trans_id,
            place_id : place_id,
            weigth : weigth,
        })
    }

    pub fn build(self) -> (FuzzyPetriNet, EventManager) {
        let pn = FuzzyPetriNet {
            arcs_from_trans: self.arcs_from_trans,
            arcs_from_place: self.arcs_from_place,
            transs : self.transs,
            places: self.places,
            trans_cntr: self.trans_cntr,
            place_cntr: self.place_cntr,
        };
        (pn, self.event_manager)
    }
}

pub struct FuzzyPetriNet {
    arcs_from_trans: Vec<ArcFromTransToPlace>,
    arcs_from_place: Vec<ArcFromPlaceToTrans>,
    transs : Vec<Trans>,
    places: Vec<Place>,
    trans_cntr: usize,
    place_cntr: usize,
}

impl FuzzyPetriNet {
    #[inline]
    pub fn get_place_nr(&self ) -> usize {
        self.place_cntr
    }

    #[inline]
    pub fn get_trans_nr(&self ) -> usize {
        self.trans_cntr
    }

    #[inline]
    pub fn is_trans_out(&self, tr_id: usize) -> bool {
        self.transs[tr_id].is_out
    }

    #[inline]
    pub fn is_place_inp(&self, pl_id: usize) -> bool {
        self.places[pl_id].is_inp
    }

    #[inline]
    pub fn table_for_trans(&self, tr_id: usize) ->&ExecutableFuzzyTable {
        self.transs[tr_id].table.get_table()
    }

    #[inline]
    pub fn typed_table_for_trans(&self, tr_id: usize) ->&FuzzyTableE {
        &self.transs[tr_id].table
    }

    pub fn get_places_after_trans(&self, tr_id: usize) -> Vec<usize>{
        let mut to_ret = Vec::new();
        for arc in &self.arcs_from_trans {
            if arc.trans_id == tr_id {
                to_ret.push(arc.place_id)
            }
        }
        to_ret
    }

    pub fn get_tanss_after_place(&self, pl_id: usize) -> Vec<usize>{
        let mut to_ret = Vec::new();
        for arc in &self.arcs_from_place {
            if arc.place_id == pl_id {
                to_ret.push(arc.trans_id)
            }
        }
        to_ret
    }

    pub fn get_places_befor_trans(&self, tr_id: usize) -> Vec<usize> {
        let mut to_ret = Vec::new();
        for arc in &self.arcs_from_place {
            if arc.trans_id == tr_id {
                to_ret.push(arc.place_id)
            }
        }
        to_ret
    }

    #[inline]
    pub fn get_initial_marking(&self, pl_id: usize) -> FuzzyToken {
        self.places[pl_id].inital_marking.clone()
    }

    #[inline]
    pub fn get_delay(&self, tr_id: usize) -> i32 {
        self.transs[tr_id].delay
    }

    pub fn get_weigth_for_arc(&self, pl_id: usize, tr_id: usize) -> f32 {
        for arc in &self.arcs_from_place {
            if arc.place_id == pl_id && arc.trans_id == tr_id{
                return arc.weigth
            }
        }
        panic!("arc does not exists")
    }

}


pub trait FuzzyTokenConsumer {
    fn consume(&mut self, ft: FuzzyToken);
}

pub struct EventManager {
    handlers: HashMap<usize, Vec<Box<FuzzyTokenConsumer>>>,
}


impl EventManager {
    pub fn new() -> EventManager {
        EventManager {
            handlers: HashMap::new(),
        }
    }

    pub fn add(&mut self, tr_id: usize, f :Box<FuzzyTokenConsumer>) {
        self.handlers.entry(tr_id).or_insert(vec![]).push(f);
    }

    pub fn execute_handler(&mut self, tr_id: usize, ft: FuzzyToken) {
        if let Some(my_handlers) = self.handlers.get_mut(&tr_id) {
            for f in (*my_handlers).iter_mut() {
                f.consume(ft.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #![allow(non_snake_case)]
    use super::{FuzzyPetriNetBuilder};
    use tables::{OneXOneTable};
    use petri_net::petri_net_builder::FuzzyTableE;

    #[test]
    fn test_petri_builder() {
        let mut bld = FuzzyPetriNetBuilder::new();
        let i_p0 = bld.add_inp_place();
        let t0 = bld.add_trans(0, FuzzyTableE::oxo(OneXOneTable::default_table()));
        let p1 = bld.add_place();
        bld.add_arc_from_place_to_trans(i_p0, t0, 0.5);
        bld.add_arc_from_trans_to_place(t0, p1);

        let (net, _) = bld.build();

        assert!(net.arcs_from_trans.len()== 1);
        assert!(net.arcs_from_place.len()== 1);
        assert!(net.transs.len()== 1);
        assert!(net.places.len()== 2);
    }
}
