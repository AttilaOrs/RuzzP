use std::io::prelude::*;
use std::fs::File;
use std::collections::HashMap;

use basic::*;
use petri_net::petri_net_builder::*;


pub struct DotStringBuilder {
    pub dot_string : String,
    place_ids  : HashMap<usize, String>,
    trans_ids  : HashMap<usize, String>,
}

use std;

impl DotStringBuilder{
    pub fn build(net: &FuzzyPetriNet) -> DotStringBuilder {
        let mut builder = DotStringBuilder {
            dot_string : String::from(""),
            place_ids: HashMap::new(),
            trans_ids: HashMap::new(),
        };
        builder.init();
        builder.add_places(net);
        builder.add_transs(net);
        builder.add_arcs_from_places(net);
        builder.add_arcs_from_transs(net);
        builder.finish();
        builder
    }

    pub fn write_to_file(&self, filne_name: &str) ->  std::io::Result<()>{
        let mut f = try!(File::create(filne_name));
        try!(f.write_all(&self.dot_string.as_bytes()));
        Ok(())
    }



    fn init(&mut self){
        self.dot_string.push_str("digraph G{ \n rankdir=LR; ");
    }

    fn finish(&mut self) {
        self.dot_string.push_str("\n}");
    }

    fn add_arcs_from_places(&mut self, net : &FuzzyPetriNet) {
        for place_id in 0..net.get_place_nr() {
            let trans_ids = net.get_tanss_after_place(place_id);
            for trans_id in trans_ids {
                self.dot_string.push('\"');
                self.dot_string.push_str(&self.place_ids[&place_id]);
                self.dot_string.push_str("\"->");
                self.dot_string.push_str(&self.trans_ids[&trans_id]);

                let weigth =net.get_weigth_for_arc(place_id, trans_id);
                if weigth != 1.0 {
                    self.dot_string.push_str("[ label=\" ");
                    self.dot_string.push_str(&weigth.to_string());
                    self.dot_string.push_str("\"] ");
                }
                self.dot_string.push_str(";\n");
            }
        }
    }

    fn add_arcs_from_transs(&mut self, net : &FuzzyPetriNet) {
        for trans_id in 0..net.get_trans_nr() {
            let place_ids = net.get_places_after_trans(trans_id);
            for place_id in place_ids {
                self.dot_string.push_str(&self.trans_ids[&trans_id]);
                self.dot_string.push_str("->\"");
                self.dot_string.push_str(&self.place_ids[&place_id]);
                self.dot_string.push_str("\";\n");
            }
        }
    }


    fn add_transs(&mut self, net: &FuzzyPetriNet){
        self.dot_string.push_str("subgraph trans {
        node [style=filled fillcolor=black shape=rect height=1 width=0.05];\n");
        for trans_id in 0..net.get_trans_nr()  {
            let tr_dot_id = DotStringBuilder::create_trans_id(trans_id);
            let tr_dot_label = DotStringBuilder::create_trans_label(net, trans_id);
            self.trans_ids.insert(trans_id, tr_dot_id.clone());

            self.dot_string.push_str(&tr_dot_id);
            self.dot_string.push_str("[label=\"\"xlabel=<<FONT POINT-SIZE='15'> ");
            self.dot_string.push_str(&tr_dot_label);
            self.dot_string.push_str("</FONT>>];\n");
        }
        self.dot_string.push_str("}\n");

    }

    fn create_trans_label(net: &FuzzyPetriNet, trans_id : usize) -> String {
        let mut to_ret = String::from("");
        if net.is_trans_out(trans_id){
            to_ret.push('o');
        };
        to_ret.push('T');
        to_ret.push_str(&trans_id.to_string());

        let delay =net.get_delay(trans_id) ;
        if delay != 0 {
            to_ret.push('[');
            to_ret.push_str(&delay.to_string());
            to_ret.push(']');
        };
        to_ret
    }

    fn create_trans_id(trans_id : usize) -> String {
        let mut to_ret = String::from("");
        to_ret.push('t');
        to_ret.push_str(&trans_id.to_string());
        to_ret
    }

    fn add_places(&mut self, net: &FuzzyPetriNet) {
        self.dot_string.push_str("subgraph palce {
        graph [shape=circle,color=gray];node [shape=circle,fixedsize=true,width=0.4];");
        for place_id in 0..net.get_place_nr() {
            let place_str_id = DotStringBuilder::create_place_id(net, place_id);
            self.place_ids.insert(place_id, place_str_id.clone());

            self.dot_string.push('"');
            self.dot_string.push_str(&place_str_id);
            self.dot_string.push('"');
            self.dot_string.push(';');

        }
        self.dot_string.push_str("}\n");

    }

    fn create_place_id(net: &FuzzyPetriNet, place_id : usize) -> String {
        let mut to_ret = String::from("");
        if net.is_place_inp(place_id) {
            to_ret.push('i');
        };
        to_ret.push('P');
        to_ret.push_str(&place_id.to_string());

        if net.get_initial_marking(place_id) != FuzzyToken::Phi {
            to_ret.push('●');
        };
        to_ret
    }

}

#[cfg(test)]
mod tests {

    #![allow(non_snake_case)]
    use super::*;
    use tables::*;
    use basic::*;
    use petri_net::petri_net_builder::{FuzzyTableE, FuzzyPetriNetBuilder};

    #[test]
    fn dot_builder_test() {
        let mut bld = FuzzyPetriNetBuilder::new();
        let i_p0 = bld.add_inp_place();
        let t0 = bld.add_trans(0, FuzzyTableE::oxo(OneXOneTable::default_table()));
        let p1 = bld.add_place();
        bld.set_initial_token(p1, FuzzyToken::zero_token());
        bld.add_arc_from_place_to_trans(i_p0, t0, 0.5);
        bld.add_arc_from_trans_to_place(t0, p1);
        let t1 = bld.add_trans(2, FuzzyTableE::oxo(OneXOneTable::default_table()));
        bld.add_arc_from_place_to_trans(p1, t1, 0.375);
        bld.add_arc_from_trans_to_place(t1, i_p0);
        let oT2 = bld.add_out_trans(FuzzyTableE::oxo(OneXOneTable::default_table()));
        bld.add_arc_from_place_to_trans(p1, oT2, 1.0);

        let (net,_) = bld.build();
        let dot_bld = DotStringBuilder::build(&net);
        assert!(dot_bld.dot_string.contains("iP0"));
        assert!(dot_bld.dot_string.contains("P1●"));
        assert!(dot_bld.dot_string.contains("T0"));
        assert!(dot_bld.dot_string.contains("T1[2]"));
        assert!(dot_bld.dot_string.contains("oT2"));
        assert!(dot_bld.dot_string.contains("\"iP0\"->t0"));
        assert!(dot_bld.dot_string.contains("\"P1●\"->t1") );
        assert!(dot_bld.dot_string.contains("\"P1●\"->t2") );
        assert!(dot_bld.dot_string.contains("t0->\"P1●\"") );
        assert!(dot_bld.dot_string.contains("t1->\"iP0\"") );
    }
}
