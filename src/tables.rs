use std::fmt;
use std::iter;


use basic::*;
use basic::UnifiedToken;
use basic::FuzzyValue::*;
use self::TableValue::*;

#[derive(PartialEq, Debug,Clone, Copy)]
pub enum TableValue {
    Phi,
    E(FuzzyValue),
}

#[derive(PartialEq, Debug,Clone, Copy)]
pub enum Operator {
    Plus, Minus, Mult, Div, NoOp
}
const  EPS: f32 =  0.00001;

impl Operator {

    pub fn exists(&self) -> bool {
        match *self {
            Operator::NoOp => false,
            _ => true,
        }
    }

    pub fn calc( &self, fi: &UnifiedToken, se: &UnifiedToken) -> Option<f32> {

        match (fi, se) {
            (&UnifiedToken::Phi, &UnifiedToken::Phi) => None,
            (&UnifiedToken::Phi, &UnifiedToken::Exist(ref v)) => Some(*v),
            (&UnifiedToken::Exist(ref v), &UnifiedToken::Phi) => Some(*v),
            (&UnifiedToken::Exist(ref fi_ref), &UnifiedToken::Exist(ref se_ref)) => {
                let fi_val = *fi_ref;
                let se_val = *se_ref;
                match *self {
                    Operator::Plus => Some(fi_val +se_val),
                    Operator::Minus => Some(fi_val - se_val),
                    Operator::Mult => Some(fi_val *se_val),
                    Operator::Div => Some(
                        if se_val > EPS  { fi_val / se_val } else {fi_val / EPS} ),
                    Operator::NoOp => unreachable!(),
                }
            }

        }


    }

}

impl TableValue {
    pub fn index(&self) -> usize {
        match *self {
            Phi => 5,
            E(fv) => fv.index(),
        }
    }

    fn map_on_value<F: FnOnce(FuzzyValue) -> () > (&self, f : F )  {
        match *self {
            Phi => {/* none */ }
            E(fv) => {f(fv);}
        }
    }
}

pub trait ExecutableFuzzyTable {
    fn is_executable(&self, inps: &Vec<FuzzyToken> ) -> bool;
    fn execute(&self, inps: Vec<FuzzyToken> ) -> Vec<FuzzyToken>;
    fn possibly_executable(&self, inps: &Vec<bool> ) -> bool;
}

pub trait ExecutableUnifiedTable {
    fn is_executable(&self, inps: &Vec<UnifiedToken>, fuz: &Vec<&Fuzzyfier>) -> bool;
    fn execute(&self, inps: Vec<UnifiedToken>,fuz: &Vec<&Fuzzyfier>, defuz: &Vec<&Defuzzyfier> ) -> Vec<UnifiedToken>;
}

#[derive(Debug)]
pub struct UnifiedOneXOneTable {
    fuzzy_table: OneXOneTable,
}

impl UnifiedOneXOneTable {
    pub fn from_arr(values: [TableValue; 6]) -> UnifiedOneXOneTable {
        UnifiedOneXOneTable{fuzzy_table: OneXOneTable::from_arr(values)}
    }

    pub fn default_table() -> UnifiedOneXOneTable {
        UnifiedOneXOneTable{fuzzy_table: OneXOneTable::default_table()}
    }
}

impl ExecutableUnifiedTable for UnifiedOneXOneTable {
    fn is_executable(&self, inps: &Vec<UnifiedToken>, fuz: &Vec<&Fuzzyfier> ) -> bool{
        let ft = fuz[0].fuzzyfy(inps[0].as_option());
        self.fuzzy_table.is_executable(&vec![ft])
    }

    fn execute(&self, inps: Vec<UnifiedToken>,fuz: &Vec<&Fuzzyfier>, defuz: &Vec<&Defuzzyfier> ) -> Vec<UnifiedToken>{
        let ft = fuz[0].fuzzyfy(inps[0].as_option());
        let mut fuzzy_out = self.fuzzy_table.execute(vec![ft]);
        let option = defuz[0].defuzzyfy(fuzzy_out.pop().expect("Impossible"));
        vec![UnifiedToken::from_option(option)]
    }
}

#[derive(Debug)]
pub struct OneXOneTable {
    values: [TableValue; 6],
}

impl OneXOneTable {
    pub fn from_arr(values: [TableValue; 6]) -> OneXOneTable {
        OneXOneTable{values : values}
    }

    pub fn default_table() -> OneXOneTable {
        OneXOneTable{values : [E(NL), E(NM), E(ZR), E(PM), E(PL), Phi]}
    }
}

macro_rules! oxo_get {
    ($self_:ident, $val: ident) =>  (
        $self_.values[$val.index()]
                                     )
}

impl ExecutableFuzzyTable for OneXOneTable {
    fn is_executable(&self, inps: &Vec<FuzzyToken> ) -> bool{
        let ref inp = inps[0];
        match *inp {
            FuzzyToken::Phi => {
                if oxo_get!(self, Phi) == Phi {false} else {true}
            }
            FuzzyToken::Exist(_) => {
                for fv in inp.nonzero_values() {
                    if oxo_get!(self, fv) != Phi {
                        return true
                    }
                }
                false

            }
        }
    }

    fn execute(&self, inps: Vec<FuzzyToken> ) -> Vec<FuzzyToken> {
        let ref inp = inps[0];
        let mut to_ret =  FuzzyToken::Phi;
        match *inp {
            FuzzyToken::Phi => {
                let ref rule = oxo_get!(self, Phi);
                rule.map_on_value(|fv| { to_ret.add_to_val(fv, 1.0)});
            }
            FuzzyToken::Exist(_) => {
                for fv in inp.nonzero_values() {
                    let ref rule =oxo_get!(self, fv);
                    rule.map_on_value(|conculsion_fv|
                                   { to_ret.add_to_val(conculsion_fv, inp.get_val(*fv))});
                }
                to_ret.normailze();
            }
        }
        vec![to_ret]
    }

    fn possibly_executable(&self, inps: &Vec<bool> ) -> bool{
        if inps[0] {
            (0..5).any(|x| self.values[x] != Phi)
        } else {
            self.values[5] != Phi
        }
    }

}


#[derive(Debug)]
pub struct  UnifiedOneXTwoTable {
    fuzzy_table : OneXTwoTable,
}

impl UnifiedOneXTwoTable {
    pub fn from_arr(values: [TableValue; 12]) -> UnifiedOneXTwoTable {
        UnifiedOneXTwoTable{fuzzy_table: OneXTwoTable::from_arr(values)}
    }

    pub fn default_table() -> UnifiedOneXTwoTable {
        UnifiedOneXTwoTable{fuzzy_table: OneXTwoTable::default_table()}
    }
}

impl ExecutableUnifiedTable for UnifiedOneXTwoTable {
    fn is_executable(&self, inps: &Vec<UnifiedToken>, fuz: &Vec<&Fuzzyfier> ) -> bool{
        let ft_one = fuz[0].fuzzyfy(inps[0].as_option());
        self.fuzzy_table.is_executable(&vec![ft_one])
    }

    fn execute(&self, inps: Vec<UnifiedToken>,fuz: &Vec<&Fuzzyfier>, defuz: &Vec<&Defuzzyfier> ) -> Vec<UnifiedToken>{
        let ft = fuz[0].fuzzyfy(inps[0].as_option());
        let mut fuzzy_out = self.fuzzy_table.execute(vec![ft]);
        let option_two = defuz[1].defuzzyfy(fuzzy_out.pop().expect("Impossible"));
        let option_one = defuz[0].defuzzyfy(fuzzy_out.pop().expect("Impossible"));
        vec![UnifiedToken::from_option(option_one), UnifiedToken::from_option(option_two)]
    }
}

#[derive(Debug)]
pub struct OneXTwoTable {
    values: [TableValue; 12],
}

impl OneXTwoTable {
    pub fn from_arr(values: [TableValue; 12]) -> OneXTwoTable {
        OneXTwoTable{values : values}
    }

    pub fn default_table() -> OneXTwoTable {
        OneXTwoTable{values :
            [E(NL),E(NL), E(NM),E(NM),E(ZR), E(ZR),E(PM), E(PM), E(PL), E(PL), Phi, Phi]}
    }
}

macro_rules! oxt_get_f {
    ($self_:ident, $val: ident) =>  (
        $self_.values[$val.index()*2]
                                     )
}

macro_rules! oxt_get_s {
    ($self_:ident, $val: ident) =>  (
        $self_.values[$val.index()*2 + 1]
                                     )
}

impl ExecutableFuzzyTable for OneXTwoTable {
    fn is_executable(&self, inps: &Vec<FuzzyToken> ) -> bool{
        let ref inp = inps[0];
        match *inp {
            FuzzyToken::Phi => {
                if oxt_get_f!(self, Phi) == Phi && oxt_get_s!(self, Phi) == Phi
                {false} else {true}
            }
            FuzzyToken::Exist(_) => {
                for fv in inp.nonzero_values() {
                    if oxt_get_f!(self, fv) != Phi || oxt_get_s!(self, fv) != Phi {
                        return true
                    }
                }
                false
            }
        }
    }

    fn execute(&self, inps: Vec<FuzzyToken> ) -> Vec<FuzzyToken> {
        let ref inp = inps[0];
        let mut to_ret_f =  FuzzyToken::Phi;
        let mut to_ret_s =  FuzzyToken::Phi;

        match *inp {
            FuzzyToken::Phi => {
                let ref f_rule = oxt_get_f!(self, Phi);
                f_rule.map_on_value(|fv| { to_ret_f.add_to_val(fv, 1.0)});
                let ref s_rule = oxt_get_s!(self, Phi);
                s_rule.map_on_value(|fv| { to_ret_s.add_to_val(fv, 1.0)});
            }
            FuzzyToken::Exist(_) => {
                for fv in inp.nonzero_values() {
                    let ref f_rule =oxt_get_f!(self, fv);
                    f_rule.map_on_value(|conculsion_fv|
                                   { to_ret_f.add_to_val(conculsion_fv, inp.get_val(*fv))});
                    let ref s_rule =oxt_get_s!(self, fv);
                    s_rule.map_on_value(|conculsion_fv|
                                   { to_ret_s.add_to_val(conculsion_fv, inp.get_val(*fv))});

                }
                to_ret_f.normailze();
                to_ret_s.normailze();
            }
        }
        vec![to_ret_f, to_ret_s]
    }

    fn possibly_executable(&self, inps: &Vec<bool> ) -> bool{
        if inps[0] {
            (0..10).any(|x| self.values[x] != Phi)
        } else {
            oxt_get_f!(self,Phi) != Phi || oxt_get_s!(self, Phi) != Phi
        }
    }
}

#[derive( Debug)]
pub struct UnifiedTwoXOneTable{
    fuzzy_table : TwoXOneTable,
    op : Operator,
}


impl UnifiedTwoXOneTable {
    pub fn from_arr(values: [TableValue; 36], op : Operator) -> UnifiedTwoXOneTable {
        UnifiedTwoXOneTable{fuzzy_table: TwoXOneTable::from_arr(values), op : op}
    }

    pub fn default_table() -> UnifiedTwoXOneTable {
        UnifiedTwoXOneTable{fuzzy_table: TwoXOneTable::default_table(), op : Operator::NoOp}
    }

    pub fn all_pl(op: Operator) -> UnifiedTwoXOneTable {
        let t = TwoXOneTable{ values :
            [E(PL), E(PL), E(PL), E(PL), E(PL),  Phi,
             E(PL), E(PL), E(PL), E(PL), E(PL),  Phi,
             E(PL), E(PL), E(PL), E(PL), E(PL),  Phi,
             E(PL), E(PL), E(PL), E(PL), E(PL),  Phi,
             E(PL), E(PL), E(PL), E(PL), E(PL),  Phi,
             Phi,   Phi,   Phi,   Phi,   Phi,    Phi, ]};
        UnifiedTwoXOneTable{fuzzy_table: t, op: op}
    }
}

impl ExecutableUnifiedTable for UnifiedTwoXOneTable {
    fn is_executable(&self, inps: &Vec<UnifiedToken>, fuz: &Vec<&Fuzzyfier> ) -> bool{
        let ft_one = fuz[0].fuzzyfy(inps[0].as_option());
        let ft_two = fuz[1].fuzzyfy(inps[1].as_option());
        self.fuzzy_table.is_executable(&vec![ft_one, ft_two])
    }

    fn execute(&self, inps: Vec<UnifiedToken>,fuz: &Vec<&Fuzzyfier>, defuz: &Vec<&Defuzzyfier> ) -> Vec<UnifiedToken>{
        let op_rez = if self.op.exists() {
            self.op.calc(&inps[0], &inps[1])
        } else {
            None
        };

        let ft_one = fuz[0].fuzzyfy(inps[0].as_option());
        let ft_two = fuz[1].fuzzyfy(inps[1].as_option());

        let mut fuzzy_out = self.fuzzy_table.execute(vec![ft_one, ft_two]);
        let rez = fuzzy_out.pop().expect("Impossible") ;
        if op_rez.is_none() {
           let option = defuz[0].defuzzyfy(rez);
           vec![UnifiedToken::from_option(option)]
       } else {
           let defult_driver = TriangleFuzzyfier::with_min_max(-1.0, 1.0);
           let r = op_rez.expect("Impossible") *defult_driver.defuzzyfy(rez).expect("Impossible");
           vec![UnifiedToken::from_val( defuz[0].limit( r ))]

       }
    }
}


pub struct TwoXOneTable  {
    values: [TableValue; 36],
}

impl fmt::Debug for TwoXOneTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ll = String::new();
        for fv in 0..self.values.len() {
            match self.values[fv] {
                TableValue::Phi => {ll.push_str("FF,")},
                TableValue::E(val) => {
                    match val {
                        FuzzyValue::NL => {ll.push_str("NL,")},
                        FuzzyValue::NM => {ll.push_str("NM,")},
                        FuzzyValue::ZR => {ll.push_str("ZR,")},
                        FuzzyValue::PM => {ll.push_str("PM,")},
                        FuzzyValue::PL => {ll.push_str("PL,")},
                    }
                }
            }

        }
        write!(f, "TwoXOneTable {}", ll)
    }
}
impl TwoXOneTable {
    pub fn from_arr(values: [TableValue; 36]) -> TwoXOneTable {
        TwoXOneTable{values : values}
    }

    pub fn default_table() -> TwoXOneTable {
        TwoXOneTable{ values :
            [E(NL), E(NL), E(NM), E(NM), E(ZR),  Phi,
             E(NL), E(NM), E(NM), E(ZR), E(PM),  Phi,
             E(NM), E(NM), E(ZR), E(PM), E(PM),  Phi,
             E(NM), E(ZR), E(PM), E(PM), E(PL),  Phi,
             E(ZR), E(PM), E(PM), E(PL), E(PL),  Phi,
             Phi,   Phi,   Phi,   Phi,   Phi,    Phi, ]}

    }
}

macro_rules! txo_get {
    ($self_:ident, $val_r: ident, $val_c: ident) =>  (
        $self_.values[$val_r.index()*6 + $val_c.index()]
                                   )
}

impl ExecutableFuzzyTable for TwoXOneTable {
    fn is_executable(&self, inps: &Vec<FuzzyToken> ) -> bool{
        if inps[0] == FuzzyToken::Phi && inps[1] == FuzzyToken::Phi {
            if txo_get!(self, Phi, Phi) == Phi { false  } else {true}
        }else if inps[0] == FuzzyToken::Phi {
            for fv in inps[1].nonzero_values() {
                if txo_get!(self,  Phi, fv) != Phi {return true}
            }
            false
        }else if inps[1] == FuzzyToken::Phi {
            for fv in inps[0].nonzero_values() {
                if txo_get!(self, fv, Phi) != Phi {return true}
            }
            false
        } else {
            for fv_f in inps[0].nonzero_values() {
                for fv_s  in inps[1].nonzero_values() {
                    if txo_get!(self, fv_f, fv_s ) != Phi {return true}

                }
            }
            false
        }
    }

    fn execute(&self, inps: Vec<FuzzyToken> ) -> Vec<FuzzyToken> {
             let mut to_ret = FuzzyToken::Phi;
             if inps[0] == FuzzyToken::Phi && inps[1] == FuzzyToken::Phi {
                 txo_get!(self,Phi, Phi).map_on_value(|fv| {to_ret.add_to_val(fv,1.0);});
             } else if inps[0] == FuzzyToken::Phi {
                 for fv in inps[1].nonzero_values() {
                     txo_get!(self,  Phi, fv).map_on_value(|conculsion_fv|
                         {to_ret.add_to_val(conculsion_fv, inps[1].get_val(*fv));});
                 }
                 to_ret.normailze();
             } else if inps[1] == FuzzyToken::Phi {
                 for fv in inps[0].nonzero_values() {
                     txo_get!(self, fv, Phi).map_on_value(|conculsion_fv|
                         {to_ret.add_to_val(conculsion_fv, inps[0].get_val(*fv));});
                 }
                 to_ret.normailze();

             } else {
                 for fv_f in inps[0].nonzero_values() {
                     for fv_s  in inps[1].nonzero_values() {
                         txo_get!(self, fv_f, fv_s).map_on_value(|conculsion_fv| {
                                let val = inps[0].get_val(*fv_f) * inps[1].get_val(*fv_s);
                                to_ret.add_to_val(conculsion_fv, val);
                            }
                        );
                     }
                 }
                 to_ret.normailze();
             }
             vec![to_ret]

    }

    fn possibly_executable(&self, inps: &Vec<bool> ) -> bool{
        for fi in index_for(inps[0]) {
            for se in index_for(inps[1]) {
                if txo_get!(self,fi,se) != Phi {
                    return true
                }
            }
        }
        false
    }
}

fn index_for(i : bool) -> Vec<TableValue> {
    if i {
        FuzzyValue::iter().map(|x| E(*x)).collect()
    } else {
        vec![Phi]
    }
}

#[derive( Debug)]
pub struct UnifiedTwoXTwoTable{
    fuzzy_table : TwoXTwoTable,
    op : Operator,
}


impl UnifiedTwoXTwoTable {

    pub fn from_arr(values: [TableValue; 72], op: Operator) -> UnifiedTwoXTwoTable {
        UnifiedTwoXTwoTable{fuzzy_table : TwoXTwoTable::from_arr(values), op: op}
    }

    pub fn default_table() -> UnifiedTwoXTwoTable {
        UnifiedTwoXTwoTable{fuzzy_table : TwoXTwoTable::default_table(), op: Operator::NoOp}
    }

    pub fn all_pl(op: Operator) -> UnifiedTwoXTwoTable {
        let t = TwoXTwoTable{ values :
            [E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL),  Phi,  Phi,
             E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL),  Phi,  Phi,
             E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL),  Phi,  Phi,
             E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL),  Phi,  Phi,
             E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL),  Phi,  Phi,
             Phi,   Phi,   Phi,   Phi,   Phi,    Phi, Phi,   Phi,   Phi,   Phi,   Phi,    Phi, ]};
        UnifiedTwoXTwoTable{fuzzy_table : t, op: op}
    }
}

impl ExecutableUnifiedTable for UnifiedTwoXTwoTable {

    fn is_executable(&self, inps: &Vec<UnifiedToken>, fuz: &Vec<&Fuzzyfier> ) -> bool{
        let ft_one = fuz[0].fuzzyfy(inps[0].as_option());
        let ft_two = fuz[1].fuzzyfy(inps[1].as_option());
        self.fuzzy_table.is_executable(&vec![ft_one, ft_two])
    }

    fn execute(&self, inps: Vec<UnifiedToken>,fuz: &Vec<&Fuzzyfier>, defuz: &Vec<&Defuzzyfier> ) -> Vec<UnifiedToken>{
        let op_rez = if self.op.exists() {
            self.op.calc(&inps[0], &inps[1])
        } else {
            None
        };

        let ft_one = fuz[0].fuzzyfy(inps[0].as_option());
        let ft_two = fuz[1].fuzzyfy(inps[1].as_option());

        let mut fuzzy_out = self.fuzzy_table.execute(vec![ft_one, ft_two]);
        let second_rez = fuzzy_out.pop().expect("Impossible") ;
        let first_rez = fuzzy_out.pop().expect("Impossible") ;
        if op_rez.is_none() {
           let option_one = defuz[0].defuzzyfy(first_rez);
           let option_two = defuz[1].defuzzyfy(second_rez);
           vec![UnifiedToken::from_option(option_one), UnifiedToken::from_option(option_two)]
       } else {
           let defult_driver = TriangleFuzzyfier::with_min_max(-1.0, 1.0);
           let r = op_rez.expect("Impossible") ;
           let first = defult_driver.defuzzyfy(first_rez).map(|v| defuz[0].limit(r*v));
           let second = defult_driver.defuzzyfy(second_rez).map(|v| defuz[1].limit(r*v));

           vec![UnifiedToken::from_option( first),
               UnifiedToken::from_option(second)]

       }
    }
}

pub struct TwoXTwoTable  {
    values: [TableValue; 72],
}


impl fmt::Debug for TwoXTwoTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ll = String::new();
        for fv in 0..self.values.len() {
            match self.values[fv] {
                TableValue::Phi => {ll.push_str("FF,")},
                TableValue::E(val) => {
                    match val {
                        FuzzyValue::NL => {ll.push_str("NL,")},
                        FuzzyValue::NM => {ll.push_str("NM,")},
                        FuzzyValue::ZR => {ll.push_str("ZR,")},
                        FuzzyValue::PM => {ll.push_str("PM,")},
                        FuzzyValue::PL => {ll.push_str("PL,")},
                    }
                }
            }

        }
        write!(f, "TwoXOneTable {}", ll)
    }
}
impl TwoXTwoTable {
    pub fn from_arr(values: [TableValue; 72]) -> TwoXTwoTable {
        TwoXTwoTable{values : values}
    }

    pub fn default_table() -> TwoXTwoTable {
        TwoXTwoTable{ values :
            [E(NL), E(NL), E(NL), E(NL), E(NM), E(NM), E(NM), E(NM), E(ZR), E(ZR),  Phi,  Phi,
             E(NL), E(NL), E(NM), E(NM), E(NM), E(NM), E(ZR), E(ZR), E(PM), E(PM),  Phi,  Phi,
             E(NM), E(NM), E(NM), E(NM), E(ZR), E(ZR), E(PM), E(PM), E(PM), E(PM),  Phi,  Phi,
             E(NM), E(NM), E(ZR), E(ZR), E(PM), E(PM), E(PM), E(PM), E(PL), E(PL),  Phi,  Phi,
             E(ZR), E(ZR), E(PM), E(PM), E(PM), E(PM), E(PL), E(PL), E(PL), E(PL),  Phi,  Phi,
             Phi,   Phi,   Phi,   Phi,   Phi,    Phi, Phi,   Phi,   Phi,   Phi,   Phi,    Phi, ]}
    }
}
macro_rules! txt_get_f {
    ($self_:ident, $val_r: ident, $val_c: ident) =>  (
        $self_.values[($val_r.index()*6 + $val_c.index())*2]
                                   )
}

macro_rules! txt_get_s {
    ($self_:ident, $val_r: ident, $val_c: ident) =>  (
        $self_.values[($val_r.index()*6 + $val_c.index())*2  + 1]
                                   )
}

 impl ExecutableFuzzyTable for TwoXTwoTable {
     fn is_executable(&self, inps: &Vec<FuzzyToken> ) -> bool {
         if inps[0] == FuzzyToken::Phi && inps[1] == FuzzyToken::Phi {
             if txt_get_f!(self, Phi, Phi) == Phi && txt_get_s!(self, Phi, Phi) == Phi
             { false  } else {true}
         }else if inps[0] == FuzzyToken::Phi {
             for fv in inps[1].nonzero_values() {
                 if txt_get_f!(self,  Phi, fv) != Phi {return true}
                 if txt_get_s!(self,  Phi, fv) != Phi {return true}
             }
             false
         }else if inps[1] == FuzzyToken::Phi {
             for fv in inps[0].nonzero_values() {
                 if txt_get_f!(self, fv, Phi) != Phi {return true}
                 if txt_get_s!(self, fv, Phi) != Phi {return true}
             }
             false
         } else {
             for fv_f in inps[0].nonzero_values() {
                 for fv_s  in inps[1].nonzero_values() {
                     if txt_get_f!(self, fv_f, fv_s ) != Phi {return true}
                     if txt_get_s!(self, fv_f, fv_s ) != Phi {return true}
                 }
             }
             false
         }
     }

     fn execute(&self, inps: Vec<FuzzyToken> ) -> Vec<FuzzyToken> {
         let mut to_ret_f = FuzzyToken::Phi;
         let mut to_ret_s = FuzzyToken::Phi;
         if inps[0] == FuzzyToken::Phi && inps[1] == FuzzyToken::Phi {
             txt_get_f!(self,Phi, Phi).map_on_value(|fv| {to_ret_f.add_to_val(fv,1.0);});
             txt_get_s!(self,Phi, Phi).map_on_value(|fv| {to_ret_s.add_to_val(fv,1.0);});
         } else if inps[0] == FuzzyToken::Phi {
             for fv in inps[1].nonzero_values() {
                 txt_get_f!(self,  Phi, fv).map_on_value(|conculsion_fv|
                     {to_ret_f.add_to_val(conculsion_fv, inps[1].get_val(*fv));});

                 txt_get_s!(self,  Phi, fv).map_on_value(|conculsion_fv|
                         {to_ret_s.add_to_val(conculsion_fv, inps[1].get_val(*fv));});

             }
             to_ret_f.normailze();
              to_ret_s.normailze();
         } else if inps[1] == FuzzyToken::Phi {
             for fv in inps[0].nonzero_values() {
                 txt_get_f!(self, fv, Phi).map_on_value(|conculsion_fv|
                     {to_ret_f.add_to_val(conculsion_fv, inps[0].get_val(*fv));});
                 txt_get_s!(self, fv, Phi).map_on_value(|conculsion_fv|
                      {to_ret_s.add_to_val(conculsion_fv, inps[0].get_val(*fv));});
             }
             to_ret_f.normailze();
             to_ret_s.normailze();

         } else {
             for fv_f in inps[0].nonzero_values() {
                 for fv_s  in inps[1].nonzero_values() {
                     let val = inps[0].get_val(*fv_f) * inps[1].get_val(*fv_s);
                     txt_get_f!(self, fv_f, fv_s).map_on_value(|conculsion_fv| {
                            to_ret_f.add_to_val(conculsion_fv, val);
                        });
                    txt_get_s!(self, fv_f, fv_s).map_on_value(|conculsion_fv| {
                            to_ret_s.add_to_val(conculsion_fv, val);
                        });
                 }
             }
             to_ret_f.normailze();
             to_ret_s.normailze();
         }
         vec![to_ret_f,to_ret_s]
     }

    fn possibly_executable(&self, inps: &Vec<bool> ) -> bool{
        for fi in index_for(inps[0]) {
            for se in index_for(inps[1]) {
                if (txt_get_f!(self,fi, se) != Phi) || (txt_get_s!(self, fi,se) != Phi)
                {
                    return true
                }
            }
        }
        false
    }
 }


#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::{ExecutableFuzzyTable, OneXOneTable,  OneXTwoTable, TwoXOneTable, TwoXTwoTable};
    use super::{ExecutableUnifiedTable, UnifiedOneXOneTable, UnifiedOneXTwoTable};
    use super::{ UnifiedTwoXOneTable, Operator, UnifiedTwoXTwoTable};
    use basic::{FuzzyToken, UnifiedToken, TriangleFuzzyfier};
    use super::TableValue::*;
    use basic::FuzzyValue::*;

    macro_rules! t {
        ($fv1:expr, $fv2:expr,$fv3:expr,$fv4:expr,$fv5:expr) =>  (
            FuzzyToken::from_arr([$fv1, $fv2, $fv3, $fv4, $fv5])
                                         )
    }

    macro_rules! ut {
        ($v:expr) => (
            UnifiedToken::from_val($v)
                     )
    }

    #[test]
    fn OneXOneTable_is_executable_test() {
        let table = OneXOneTable::default_table();
        assert!(table.is_executable(&vec![FuzzyToken::zero_token()]));
        assert!(! table.is_executable(&vec![FuzzyToken::Phi]));

        let table = OneXOneTable::from_arr([Phi, Phi, E(NL), Phi, Phi, E(PM)]);
        assert!(table.is_executable(&vec![FuzzyToken::zero_token()]));
        assert!(table.is_executable(&vec![FuzzyToken::Phi]));
        assert!(! table.is_executable( &vec![t!(1.0, 0.0, 0.0, 0.0, 0.0)]));
    }


    #[test]
    fn OneXOneTable_possibly_executable() {
        let table = OneXOneTable::default_table();
        assert!(table.possibly_executable( &vec![true]));
        assert!(! table.possibly_executable(&vec![false]));

        let table = OneXOneTable::from_arr([Phi, Phi, E(NL), Phi, Phi, E(PM)]);
        assert!(table.possibly_executable(&vec![true]));
        assert!( table.possibly_executable(&vec![false]));

        let table = OneXOneTable::from_arr([Phi, Phi, Phi, Phi, Phi, E(PM)]);
        assert!(! table.possibly_executable(&vec![true]));
        assert!( table.possibly_executable(&vec![false]));

    }

    #[test]
    fn UnifiedOneXOneTable_is_executbale_test(){
        let table = UnifiedOneXOneTable::default_table();
        let default_fuzzifier = TriangleFuzzyfier::with_min_max(-1.0,1.0);
        assert!(table.is_executable(&vec![ut!(0.0)], &vec![ & default_fuzzifier] ));
        assert!(! table.is_executable(&vec![UnifiedToken::Phi], &vec![ & default_fuzzifier] ));
    }

    #[test]
    fn OneXOneTable_execute_test() {
        let table = OneXOneTable::from_arr([E(PL), E(PM), E(NL), Phi , E(NL), Phi]);

        let rez = table.execute(vec![FuzzyToken::zero_token()]);
        assert_eq!(rez,  vec![t!(1.0, 0.0, 0.0, 0.0, 0.0)]);

        let rez = table.execute(vec![t!(1.0, 1.0, 0.0, 0.0, 0.0)]);
        assert_eq!(rez,  vec![t!(0.0, 0.0, 0.0, 0.5, 0.5)]);

        let rez = table.execute(vec![t!(0.0, 0.0, 0.0, 1.0, 0.0)]);
        assert_eq!(rez,  vec![FuzzyToken::Phi]);

        let rez = table.execute(vec![FuzzyToken::Phi]);
        assert_eq!(rez,  vec![FuzzyToken::Phi]);

        let rez = table.execute(vec![t!(1.0, 0.0, 2.0, 0.0, 2.0)]);
        assert_eq!(rez,  vec![t!(0.8, 0.0, 0.0, 0.0, 0.2)]);


        let table = OneXOneTable::from_arr([ Phi, Phi,Phi,Phi,Phi, E(ZR)]);

        let rez = table.execute(vec![FuzzyToken::zero_token()]);
        assert_eq!(rez,  vec![FuzzyToken::Phi]);

        let rez = table.execute(vec![FuzzyToken::Phi]);
        assert_eq!(rez,  vec![FuzzyToken::zero_token()]);
    }

    #[test]
    fn UnifiedOneXOneTable_execute_test(){
        let fuzz_defuzzs = TriangleFuzzyfier::with_min_max(-1.0,1.0);
        let table = UnifiedOneXOneTable::from_arr([E(PL), E(PM), E(NL), Phi , E(NL), Phi]);
        let rez = table.execute(vec![ut!(1.0)], &vec![& fuzz_defuzzs], &vec![&fuzz_defuzzs] );
        assert_eq!(rez, vec![ut!(-1.0)]);

    }

    #[test]
    fn map_on_value_test() {
        let rule = Phi;
        let mut i_running = false;
        rule.map_on_value(|_| {i_running = true});
        assert!(!i_running);
        let rule = E(NL);
        rule.map_on_value(|_| {i_running = true});
        assert!(i_running);
    }

    #[test]
    fn OneXTwoTable_is_executable_test() {
        let table = OneXTwoTable::default_table();
        assert!(table.is_executable(&vec![FuzzyToken::zero_token()]));
        assert!(! table.is_executable(&vec![FuzzyToken::Phi]));

        let table = OneXTwoTable::from_arr(
            [Phi, Phi, Phi, E(NM), E(NL),Phi, Phi, Phi, Phi, Phi, E(PM), Phi]);
        assert!(table.is_executable(&vec![FuzzyToken::zero_token()]));
        assert!(table.is_executable(&vec![FuzzyToken::Phi]));
        assert!(! table.is_executable( &vec![t!(1.0, 0.0, 0.0, 0.0, 0.0)]));
        assert!(table.is_executable( &vec![t!(0.0, 1.0, 0.0, 0.0, 0.0)]));
        assert!(! table.is_executable( &vec![t!(0.0, 0.0, 0.0, 1.0, 0.0)]));

        let table = OneXTwoTable::from_arr(
            [Phi, Phi, Phi, Phi, Phi, Phi, Phi, Phi, Phi, Phi, Phi, E(ZR)]);
        assert!(table.is_executable(&vec![FuzzyToken::Phi]));
        assert!(! table.is_executable( &vec![t!(0.0, 0.0, 0.0, 1.0, 0.0)]));
    }

    #[test]
    fn UnifiedOneXTwoTable_is_executable_test() {
        let table = UnifiedOneXTwoTable::default_table();
        let default_fuzzifier = TriangleFuzzyfier::with_min_max(-1.0, 1.0);
        assert!(table.is_executable(&vec![ut!(0.0)], &vec![ & default_fuzzifier]));
        assert!(! table.is_executable(&vec![UnifiedToken::Phi], &vec![ & default_fuzzifier]));
    }

    #[test]
    fn OneXTwoTable_execute_test() {
        let table = OneXTwoTable::from_arr(
            [E(PL), Phi,
             E(PM), Phi,
              Phi,  Phi,
              Phi, E(NM),
              Phi, E(NL),
              Phi,  Phi ]);

        let rez = table.execute(vec![FuzzyToken::zero_token()]);
        assert_eq!(rez,  vec![FuzzyToken::Phi, FuzzyToken::Phi]);

        let rez = table.execute(vec![t!(1.0, 0.0, 0.0, 0.0, 0.0)]);
        assert_eq!(rez,  vec![t!(0.0, 0.0, 0.0, 0.0, 1.0), FuzzyToken::Phi]);

        let rez = table.execute(vec![t!(0.0, 0.0, 0.0, 1.0, 0.0)]);
        assert_eq!(rez,  vec![FuzzyToken::Phi, t!(0.0, 1.0, 0.0, 0.0, 0.0)]);

        let rez = table.execute(vec![FuzzyToken::Phi]);
        assert_eq!(rez,  vec![FuzzyToken::Phi, FuzzyToken::Phi]);

        let table = OneXTwoTable::from_arr(
            [E(PL), E(NM),
             E(PM), E(NL),
              Phi,  Phi,
             E(PM), E(NM),
             E(PL), E(NL),
              Phi,  E(ZR) ]);

        let rez = table.execute(vec![FuzzyToken::Phi]);
        assert_eq!(rez,  vec![FuzzyToken::Phi, t!(0.0, 0.0, 1.0, 0.0, 0.0)]);

        let rez = table.execute(vec![t!(1.0, 1.0, 1.0, 1.0, 1.0)]);
        assert_eq!(rez,  vec![t!(0.0, 0.0, 0.0, 0.5, 0.5), t!(0.5, 0.5, 0.0, 0.0, 0.0)]);

    }

    fn OneXTwo_possible_executable(){
        let table = OneXTwoTable::default_table();
        assert!(table.possibly_executable(&vec![true]));
        assert!(! table.possibly_executable(&vec![false]));

        let table = OneXTwoTable::from_arr(
            [E(PL), E(NM),
             E(PM), E(NL),
              Phi,  Phi,
             E(PM), E(NM),
             E(PL), E(NL),
             Phi,  E(ZR) ]);

        assert!(table.possibly_executable(&vec![true]));
        assert!( table.possibly_executable(&vec![false]));

        let table = OneXTwoTable::from_arr(
            [Phi, Phi,
             Phi, Phi,
             Phi, Phi,
             Phi, Phi,
             Phi, Phi,
             Phi, Phi ]);

        assert!(!table.possibly_executable(&vec![true]));
        assert!(! table.possibly_executable(&vec![false]));
    }


    #[test]
    fn UnifiedOneXTwoTable_execute_test() {
        let table = UnifiedOneXTwoTable::from_arr(
            [E(PL), Phi,
             E(PM), Phi,
              Phi,  Phi,
              Phi, E(NM),
              Phi, E(NL),
              Phi,  Phi ]);

        let fuzz_one = TriangleFuzzyfier::with_min_max(-1.0, 1.0)  ;
        let fuzz_two = TriangleFuzzyfier::with_min_max(-2.0, 2.0)  ;

        let rez = table.execute(vec![UnifiedToken::zero_token()], &vec![&fuzz_one], &vec![&fuzz_one, &fuzz_two]);
        assert_eq!(rez,  vec![UnifiedToken::Phi, UnifiedToken::Phi]);

        let rez = table.execute(vec![ut!(1.0)], &vec![&fuzz_one], &vec![&fuzz_one, &fuzz_two]);
        assert_eq!(rez,  vec![UnifiedToken::Phi, ut!(-2.0)]);
        let rez = table.execute(vec![ut!(-0.75)], &vec![&fuzz_one], &vec![&fuzz_one, &fuzz_two]);
        assert_eq!(rez,  vec![ut!(0.75), UnifiedToken::Phi]);
    }

    #[test]
    fn UnifiedTwoXOneTable_executable_test(){
        let default_fuzzifier =TriangleFuzzyfier::with_min_max(-1.0,1.0);
        let table = UnifiedTwoXOneTable::default_table();
        assert!(table.is_executable(&vec![UnifiedToken::zero_token(),
                                    UnifiedToken::zero_token()],
            &vec![ &default_fuzzifier, &default_fuzzifier]));
        assert!(! table.is_executable(&vec![UnifiedToken::Phi,UnifiedToken::zero_token()],
            &vec![ &default_fuzzifier, &default_fuzzifier]));


        let complex_table = UnifiedTwoXOneTable::from_arr(
            [Phi,   Phi  , Phi  , Phi  , Phi  ,  Phi,
             E(PL), E(NM), E(NM), E(NM), E(PM),  Phi,
             E(NM), E(NM), E(NM), E(PM), E(PM), E(ZR),
             E(NM), E(NM), E(PM), E(PM), E(NM),  Phi,
             E(NM), E(PM), E(PM), E(NM), E(NM),  Phi,
             Phi,   E(ZR), Phi,   Phi,   Phi,    E(PL), ],
             Operator::Plus
        );
        assert!(complex_table.is_executable(&vec![ut!(-2.0), ut!(-2.0)],
            &vec![ & TriangleFuzzyfier::with_min_max(-5.0,5.0), & default_fuzzifier]));

        assert!(!complex_table.is_executable(&vec![ut!(-2.0), ut!(-2.0)],
                &vec![ & default_fuzzifier, & TriangleFuzzyfier::with_min_max(-5.0,5.0)]))

    }

    #[test]
    fn UnifiedTwoXOneTable_execute_test(){
        let plus_table = UnifiedTwoXOneTable::all_pl(Operator::Plus);
        let big_defuz = TriangleFuzzyfier::with_min_max(-10.0, 10.0);
        let rez = plus_table.execute(vec![ut!(1.0), ut!(1.0)], &vec![&big_defuz, &big_defuz],
                           &vec![&big_defuz, &big_defuz]);
        assert_eq!(rez, vec![ut!(2.0)]);
    }

    #[test]
    fn TwoXOne_is_executable_default_table_test() {
        let table = TwoXOneTable::default_table();
        assert!(!table.is_executable(&vec![FuzzyToken::Phi, FuzzyToken::Phi]));
        assert!(!table.is_executable(&vec![FuzzyToken::Phi, t!(0.0, 0.0, 1.0, 0.0, 0.0) ]));
        assert!(!table.is_executable(&vec![t!(0.0, 0.0, 1.0, 0.0, 0.0), FuzzyToken::Phi]));
        assert!(table.is_executable(&vec![t!(0.0, 0.0, 1.0, 0.0, 0.0),
                                    t!(1.0, 0.0, 0.0, 0.0, 0.0)]));
    }


    #[test]
    fn TwoXOne_is_executable_intersting_table_test() {
        let table = TwoXOneTable::from_arr(
            [E(PL), E(PL), E(NM), E(NM), Phi  ,  Phi,
             E(PL), E(NM), E(NM), Phi  , E(PM),  Phi,
             E(NM), E(NM), Phi  , E(PM), E(PM), E(ZR),
             E(NM), Phi  , E(PM), E(PM), E(NM),  Phi,
             Phi  , E(PM), E(PM), E(NM), E(NM),  Phi,
             Phi,   E(ZR), Phi,   Phi,   Phi,    E(PL), ]);

        assert!(table.is_executable(&vec![FuzzyToken::Phi, FuzzyToken::Phi]));
        assert!(table.is_executable(&vec![ t!(0.0, 0.0, 1.0, 0.0, 0.0),FuzzyToken::Phi ]));
        assert!(!table.is_executable(&vec![ FuzzyToken::Phi, t!(0.0, 0.0, 1.0, 0.0, 0.0)]));
        assert!(table.is_executable(&vec![FuzzyToken::Phi,t!(0.0, 1.0, 0.0, 0.0, 0.0)]));
        assert!(!table.is_executable(&vec![t!(0.0, 0.0, 1.0, 0.0, 0.0),
                                     t!(0.0, 0.0, 1.0, 0.0, 0.0)]));
        assert!(table.is_executable(&vec![t!(1.0, 0.0, 1.0, 0.0, 0.0),
                                    t!(1.0, 0.0, 1.0, 0.0, 0.0)]));
        assert!(table.is_executable(&vec![t!(0.0, 0.0, 1.0, 1.0, 0.0),
                                    t!(0.0, 0.0, 1.0, 0.0, 1.0)]));
    }

    #[test]
    fn TwoXOneTable_possibly_executable(){
        let table = TwoXOneTable::default_table();
        assert!(table.possibly_executable(&vec![true, true]));
        assert!(!table.possibly_executable(&vec![false, true]));
        assert!(!table.possibly_executable(&vec![true, false]));
        assert!(!table.possibly_executable(&vec![false, false]));

        let table = TwoXOneTable::from_arr(
            [E(PL), E(PL), E(NM), E(NM), Phi  ,  Phi,
             E(PL), E(NM), E(NM), Phi  , E(PM),  Phi,
             E(NM), E(NM), Phi  , E(PM), E(PM), E(ZR),
             E(NM), Phi  , E(PM), E(PM), E(NM),  Phi,
             Phi  , E(PM), E(PM), E(NM), E(NM),  Phi,
             Phi,   Phi  , Phi,   Phi,   Phi,    E(PL), ]);

        assert!(table.possibly_executable(&vec![true, true]));
        assert!(!table.possibly_executable(&vec![false, true]));
        assert!(table.possibly_executable(&vec![true, false]));
        assert!(table.possibly_executable(&vec![false, false]));

        let table = TwoXOneTable::from_arr(
            [Phi,Phi,Phi,Phi,Phi,Phi,
             Phi,Phi,Phi,Phi,Phi,Phi,
             Phi,Phi,Phi,Phi,Phi,Phi,
             Phi,Phi,Phi,Phi,Phi,Phi,
             Phi,Phi,Phi,Phi,Phi,Phi,
           E(NM),Phi,Phi,Phi,Phi, Phi, ]);


        assert!(!table.possibly_executable(&vec![true, true]));
        assert!(table.possibly_executable(&vec![false, true]));
        assert!(!table.possibly_executable(&vec![true, false]));
        assert!(!table.possibly_executable(&vec![false, false]));
    }

    #[test]
    fn TwoXOneTable_execute_default_table_test() {
        let table = TwoXOneTable::default_table();

        let rez = table.execute(vec![FuzzyToken::Phi, FuzzyToken::Phi]);
        assert_eq!(rez, vec![FuzzyToken::Phi]);

        let rez = table.execute(vec![FuzzyToken::Phi, FuzzyToken::zero_token()]);
        assert_eq!(rez, vec![FuzzyToken::Phi]);

        let rez = table.execute(vec![FuzzyToken::zero_token(),FuzzyToken::Phi]);
        assert_eq!(rez, vec![FuzzyToken::Phi]);

        let rez = table.execute(vec![t!(0.0, 0.0, 1.0, 0.0, 0.0),
                                t!(0.0, 1.0, 0.0, 0.0 , 0.0) ]);
        assert_eq!(rez, vec![t!(0.0, 1.0, 0.0, 0.0, 0.0)]);
        let rez = table.execute(vec![t!(0.0, 0.5, 0.5, 0.0, 0.0),
                                t!(0.5, 0.5, 0.0, 0.0 , 0.0) ]);
        assert_eq!(rez, vec![t!(0.25, 0.75, 0.0, 0.0, 0.0)]);
    }

    #[test]
    fn TwoXOneTable_execute_intreseting_table_test() {
        let table = TwoXOneTable::from_arr(
            [E(PL), E(PL), E(NM), E(NM), Phi  ,  Phi,
             E(PL), E(NM), E(NM), Phi  , E(PM),  Phi,
             E(NM), E(NM), Phi  , E(PM), E(PM), E(ZR),
             E(NM), Phi  , E(PM), E(PM), E(NM),  Phi,
             Phi  , E(PM), E(PM), E(NM), E(NM),  Phi,
             Phi,   E(PL), Phi,   Phi,   Phi,    E(PL), ]);

        let rez = table.execute(vec![FuzzyToken::Phi, FuzzyToken::Phi]);
        assert_eq!(rez, vec![t!(0.0, 0.0, 0.0, 0.0, 1.0)]);


        let rez = table.execute(vec![FuzzyToken::zero_token(),FuzzyToken::Phi]);
        assert_eq!(rez, vec![FuzzyToken::zero_token()]);

        let rez = table.execute(vec![FuzzyToken::Phi, FuzzyToken::zero_token()]);
        assert_eq!(rez, vec![FuzzyToken::Phi]);
        let rez = table.execute(vec![FuzzyToken::Phi, t!(0.0, 1.0, 0.0, 0.0, 0.0)]);
        assert_eq!(rez, vec![t!(0.0, 0.0, 0.0, 0.0, 1.0)]);

        let rez = table.execute(vec![t!(1.0, 1.0, 0.0, 0.0, 0.0), t!(1.0, 1.0, 0.0, 0.0, 0.0)]);
        assert_eq!(rez, vec![t!(0.0, 0.25, 0.0, 0.0, 0.75)]);

        let rez = table.execute(vec![t!(1.0, 1.0, 0.0, 0.0, 0.0), t!(0.0, 0.0, 0.0, 1.0, 1.0)]);
        assert_eq!(rez, vec![t!(0.0, 0.5, 0.0, 0.5, 0.0)]);

    }

    #[test]
    fn TwoXTwoTable_is_executable_default_table(){
        let table = TwoXTwoTable::default_table();
        assert!(! table.is_executable(&vec![FuzzyToken::Phi, FuzzyToken::Phi]));
        assert!(! table.is_executable(&vec![FuzzyToken::zero_token(), FuzzyToken::Phi]));
        assert!(! table.is_executable(&vec![FuzzyToken::Phi, FuzzyToken::zero_token()]));
        assert!( table.is_executable(&vec![FuzzyToken::zero_token(),
                                     FuzzyToken::zero_token()]));
    }
    #[test]
    fn TwoXTwoTable_is_executable_simple_table(){
    let table = TwoXTwoTable::from_arr(
        [Phi  , Phi  , Phi  , Phi  , Phi  , Phi  , Phi  , Phi  , Phi  , Phi  ,  Phi,  Phi,
         Phi  , Phi  , Phi  , Phi  , Phi  , Phi  , Phi  , Phi  , Phi  , Phi  ,  Phi  ,Phi,
         Phi  , Phi  , Phi  , Phi  , Phi  , Phi  , Phi  , Phi  , Phi  , Phi  ,  Phi,  Phi  ,
         Phi  , Phi  , Phi  , Phi  , Phi  , Phi  , Phi  , Phi  , Phi  , Phi  ,  Phi,  Phi,
         Phi  , Phi  , Phi  , Phi  , Phi  , Phi  , Phi  , Phi  , Phi  , Phi  ,  Phi,  Phi,
         Phi,   Phi,   Phi,   Phi,   Phi,   Phi,   Phi,   Phi  , Phi  , Phi  ,  Phi  ,E(PL),]);
        assert!( table.is_executable(&vec![FuzzyToken::Phi, FuzzyToken::Phi]));
        assert!(! table.is_executable(&vec![FuzzyToken::zero_token(), FuzzyToken::Phi]));
        assert!(! table.is_executable(&vec![FuzzyToken::Phi, FuzzyToken::zero_token()]));
        assert!(! table.is_executable(&vec![FuzzyToken::zero_token(),
                                     FuzzyToken::zero_token()]));
    }

    #[test]
    fn TwoXTwoTable_is_executable_interseting_table(){
    let table = TwoXTwoTable::from_arr(
        [E(NL), E(NL), E(NL), E(NL), E(NM), E(NM), E(NM), E(NM), Phi  , Phi  ,  Phi,  Phi,
         E(NL), E(NL), E(NM), E(NM), E(NM), E(NM), Phi  , Phi  , E(PM), E(PM),  E(PL),Phi,
         E(NM), E(NM), E(NM), E(NM), Phi  , Phi  , E(PM), E(PM), E(PM), E(PM),  Phi,  E(NM),
         E(NM), E(NM), Phi  , Phi  , E(PM), E(PM), E(PM), E(PM), E(PL), E(PL),  Phi,  Phi,
         Phi  , Phi  , E(PM), E(PM), E(PM), E(PM), E(PL), E(PL), E(PL), E(PL),  Phi,  Phi,
         Phi,   Phi,   Phi,   Phi,   Phi,   Phi,   Phi,   E(NM), E(PM),   Phi,   E(NM), Phi,]);

        assert!(table.is_executable(&vec![FuzzyToken::Phi, FuzzyToken::Phi]));
        assert!(table.is_executable(&vec![t!(0.0, 1.0, 0.0, 0.0, 0.0), FuzzyToken::Phi]));
        assert!(table.is_executable(&vec![t!(0.0, 0.0, 1.0, 0.0, 0.0), FuzzyToken::Phi]));
        assert!(!table.is_executable(&vec![t!(0.0, 0.0, 0.0, 1.0, 0.0), FuzzyToken::Phi]));

        assert!(table.is_executable(&vec![FuzzyToken::Phi,t!(0.0, 0.0, 0.0, 1.0, 0.0)]));
        assert!(table.is_executable(&vec![FuzzyToken::Phi,t!(0.0, 0.0, 0.0, 0.0, 1.0)]));
        assert!(!table.is_executable(&vec![t!(0.0, 0.0, 1.0, 0.0, 0.0),
                                     t!(0.0, 0.0, 1.0, 0.0, 0.0)]));
        assert!(table.is_executable(&vec![t!(0.0, 0.0, 0.0, 1.0, 0.0),
                                    t!(0.0, 0.0, 1.0, 0.0, 0.0)]));
    }

    #[test]
    fn TwoXTwoTable_possibly_executable(){
        let table = TwoXTwoTable::default_table();
        assert!(table.possibly_executable(&vec![true, true]));
        assert!(!table.possibly_executable(&vec![false, true]));
        assert!(!table.possibly_executable(&vec![true, false]));
        assert!(!table.possibly_executable(&vec![false, false]));

    }

    #[test]
    fn TwoXTwoTable_execute_default_table(){
        let table = TwoXTwoTable::default_table();

        let rez = table.execute(vec![FuzzyToken::Phi, FuzzyToken::Phi]);
        assert_eq!(rez, vec![FuzzyToken::Phi, FuzzyToken::Phi]);

        let rez = table.execute(vec![t!(0.0, 0.0, 1.0, 0.0, 0.0),
                                t!(0.0, 1.0, 0.0, 0.0 , 0.0) ]);
        assert_eq!(rez, vec![t!(0.0, 1.0, 0.0, 0.0, 0.0), t!(0.0, 1.0, 0.0, 0.0, 0.0)]);
        let rez = table.execute(vec![t!(0.0, 0.5, 0.5, 0.0, 0.0),
                                t!(0.5, 0.5, 0.0, 0.0 , 0.0) ]);
        assert_eq!(rez, vec![t!(0.25, 0.75, 0.0, 0.0, 0.0), t!(0.25, 0.75, 0.0, 0.0, 0.0)]);
    }

    #[test]
    fn TwoXTwoTable_execute_interseting_table(){
    let table = TwoXTwoTable::from_arr(
        [E(NL), E(NL), E(NL), E(NL), E(PL), E(PL), E(PL), E(PL), Phi  , Phi  ,  Phi,  Phi,
         E(NL), E(NL), E(PL), E(PL), E(PL), E(PL), Phi  , Phi  , E(PM), E(PM),  E(PL),Phi,
         E(PL), E(PL), E(PL), E(PL), Phi  , Phi  , E(PM), E(PM), E(PM), E(PM),  Phi,  E(PL),
         E(PL), E(PL), Phi  , Phi  , E(PM), E(PM), E(PM), E(PM), E(PL), E(PL),  Phi,  Phi,
         Phi  , Phi  , E(PM), E(PM), E(PM), E(PM), E(PL), E(PL), E(PL), E(PL),  Phi,  Phi,
         Phi,   Phi,   Phi,   Phi,   Phi,   Phi,   Phi,   E(PL), E(PM),   Phi,   E(PL), Phi,]);

        let rez = table.execute(vec![FuzzyToken::Phi, FuzzyToken::Phi]);
        assert_eq!(rez, vec![t!(0.0, 0.0, 0.0, 0.0, 1.0), FuzzyToken::Phi]);

        let rez = table.execute(vec![t!(0.0, 1.0, 0.0, 0.0, 0.0), FuzzyToken::Phi]);
        assert_eq!(rez, vec![t!(0.0, 0.0, 0.0, 0.0, 1.0), FuzzyToken::Phi]);

        let rez = table.execute(vec![t!(0.0, 0.0, 1.0, 0.0, 0.0), FuzzyToken::Phi]);
        assert_eq!(rez, vec![FuzzyToken::Phi, t!(0.0, 0.0, 0.0, 0.0, 1.0)]);

        let rez = table.execute(vec![FuzzyToken::Phi, t!(0.0, 0.0, 0.0, 0.0, 1.0)]);
        assert_eq!(rez, vec![t!(0.0, 0.0, 0.0, 1.0, 0.0), FuzzyToken::Phi]);

        let rez = table.execute(vec![FuzzyToken::Phi, t!(0.0, 0.0, 0.0, 1.0, 0.0)]);
        assert_eq!(rez, vec![FuzzyToken::Phi,t!(0.0, 0.0, 0.0, 0.0, 1.0)]);

        let rez = table.execute(vec![t!(0.6, 0.4, 0.0, 0.0, 0.0),
                                t!(0.0, 0.3, 0.0, 0.7, 0.0)]);
        assert_eq!(rez, vec![t!(0.25, 0.0, 0.0, 0.0, 0.75), t!(0.25, 0.0, 0.0, 0.0, 0.75)]);
    }

    #[test]
    fn UnifiedTwoXTwoTable_is_execuatble(){
        let default_fuzzyfier = TriangleFuzzyfier::with_min_max(-1.0, 1.0);
        let bigger_fuzzyfier = TriangleFuzzyfier::with_min_max(-10.0, 10.0);

        let default_table = UnifiedTwoXTwoTable::default_table();

        assert!(default_table.is_executable(&vec![ut!(0.0), ut!(0.0)],
            &vec![&default_fuzzyfier, &default_fuzzyfier]));

        assert!(!default_table.is_executable(&vec![UnifiedToken::Phi, ut!(0.0)],
            &vec![&default_fuzzyfier, &default_fuzzyfier]));

    let intersing_table = UnifiedTwoXTwoTable::from_arr(
        [E(NL), E(NL), E(NL), E(NL), E(NM), E(NM), E(NM), E(NM), Phi  , Phi  ,  Phi,  Phi,
         E(NL), E(NL), E(NM), E(NM), E(NM), E(NM), Phi  , Phi  , E(PM), E(PM),  E(PL),Phi,
         E(NM), E(NM), E(NM), E(NM), Phi  , Phi  , E(PM), E(PM), E(PM), E(PM),  Phi,  E(NM),
         E(NM), E(NM), Phi  , Phi  , E(PM), E(PM), E(PM), E(PM), E(PL), E(PL),  Phi,  Phi,
         Phi  , Phi  , E(PM), E(PM), E(PM), E(PM), E(PL), E(PL), E(PL), E(PL),  Phi,  Phi,
         Phi,   Phi,   Phi,   Phi,   Phi,   Phi,   Phi,   E(NM), E(PM),   Phi,   E(NM), Phi,],
             Operator::Mult);

        assert!(!intersing_table.is_executable(&vec![ut!(-10.0), ut!(2.0)],
            &vec![&bigger_fuzzyfier, &default_fuzzyfier]));

        assert!(intersing_table.is_executable(&vec![ut!(-7.0), ut!(2.0)],
            &vec![&bigger_fuzzyfier, &default_fuzzyfier]));

        assert!(intersing_table.is_executable(&vec![ut!(-7.0), UnifiedToken::Phi],
            &vec![&bigger_fuzzyfier, &default_fuzzyfier]));
    }

    #[test]
    fn UnifiedTwoXTwoTable_execute(){
        let default_fuzzyfier = TriangleFuzzyfier::with_min_max(-1.0, 1.0);
        let bigger_fuzzyfier = TriangleFuzzyfier::with_min_max(-10.0, 10.0);
        let medium_fuzzyfier = TriangleFuzzyfier::with_min_max(-5.0, 5.0);
        let small_fuzzyfier = TriangleFuzzyfier::with_min_max(-2.0, 2.0);

        let first_table = UnifiedTwoXTwoTable::all_pl(Operator::Mult);

        let rez = first_table.execute(vec![ut!(5.0), ut!(2.0)],
                                      &vec![&bigger_fuzzyfier, &medium_fuzzyfier],
                                      &vec![&medium_fuzzyfier, &bigger_fuzzyfier]);
        assert_eq!(rez, vec![ut!(5.0), ut!(10.0)]);

    let t = TwoXTwoTable{ values :
        [  Phi, E(ZR), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(ZR),   E(PL),  Phi,  Phi,
         E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL),  Phi,  Phi,
         E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL),  Phi,  Phi,
         E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL),  Phi,  Phi,
         E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL), E(PL),  Phi,  Phi,
         Phi,   Phi,   Phi,   Phi,   Phi,    Phi, Phi,   Phi,   Phi,   Phi,   Phi,    Phi, ]};
        let second_table = UnifiedTwoXTwoTable{fuzzy_table : t, op: Operator::Plus};

        let rez = second_table.execute(vec![ut!(-10.0), ut!(2.0)],
                                      &vec![&bigger_fuzzyfier, &small_fuzzyfier],
                                      &vec![&default_fuzzyfier, &bigger_fuzzyfier]);

        assert_eq!(rez, vec![ut!(0.0), ut!(-8.0)]);

        let rez = second_table.execute(vec![ut!(-10.0), ut!(-2.0)],
                                      &vec![&bigger_fuzzyfier, &small_fuzzyfier],
                                      &vec![&default_fuzzyfier, &bigger_fuzzyfier]);

        assert_eq!(rez, vec![UnifiedToken::Phi, ut!(0.0)]);
    }


}
