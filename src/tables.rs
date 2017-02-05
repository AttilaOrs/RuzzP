use std::fmt;


use basic::*;
use basic::FuzzyValue::*;
use self::TableValue::*;

#[derive(PartialEq, Debug,Clone, Copy)]
pub enum TableValue {
    Phi,
    E(FuzzyValue),
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

 }


#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::{ExecutableFuzzyTable, OneXOneTable,  OneXTwoTable, TwoXOneTable, TwoXTwoTable};
    use basic::{FuzzyToken};
    use super::TableValue::*;
    use basic::FuzzyValue::*;

    macro_rules! t {
        ($fv1:expr, $fv2:expr,$fv3:expr,$fv4:expr,$fv5:expr) =>  (
            FuzzyToken::from_arr([$fv1, $fv2, $fv3, $fv4, $fv5])
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

    #[test]
    fn TwoXOne_is_executable_default_table_test() {
        let table = TwoXOneTable::default_table();
        assert!(!table.is_executable(&vec![FuzzyToken::Phi, FuzzyToken::Phi]));
        assert!(!table.is_executable(&vec![FuzzyToken::Phi, t!(0.0, 0.0, 1.0, 0.0, 0.0) ]));
        assert!(!table.is_executable(&vec![t!(0.0, 0.0, 1.0, 0.0, 0.0), FuzzyToken::Phi]));
        assert!(table.is_executable(&vec![t!(0.0, 0.0, 1.0, 0.0, 0.0), t!(1.0, 0.0, 0.0, 0.0, 0.0)]));
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
        assert!(!table.is_executable(&vec![t!(0.0, 0.0, 1.0, 0.0, 0.0), t!(0.0, 0.0, 1.0, 0.0, 0.0)]));
        assert!(table.is_executable(&vec![t!(1.0, 0.0, 1.0, 0.0, 0.0), t!(1.0, 0.0, 1.0, 0.0, 0.0)]));
        assert!(table.is_executable(&vec![t!(0.0, 0.0, 1.0, 1.0, 0.0), t!(0.0, 0.0, 1.0, 0.0, 1.0)]));
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

        let rez = table.execute(vec![t!(0.0, 0.0, 1.0, 0.0, 0.0), t!(0.0, 1.0, 0.0, 0.0 , 0.0) ]);
        assert_eq!(rez, vec![t!(0.0, 1.0, 0.0, 0.0, 0.0)]);
        let rez = table.execute(vec![t!(0.0, 0.5, 0.5, 0.0, 0.0), t!(0.5, 0.5, 0.0, 0.0 , 0.0) ]);
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
        assert!( table.is_executable(&vec![FuzzyToken::zero_token(), FuzzyToken::zero_token()]));
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
        assert!(!table.is_executable(&vec![t!(0.0, 0.0, 1.0, 0.0, 0.0), t!(0.0, 0.0, 1.0, 0.0, 0.0)]));
        assert!(table.is_executable(&vec![t!(0.0, 0.0, 0.0, 1.0, 0.0), t!(0.0, 0.0, 1.0, 0.0, 0.0)]));
    }

    #[test]
    fn TwoXTwoTable_execute_default_table(){
        let table = TwoXTwoTable::default_table();

        let rez = table.execute(vec![FuzzyToken::Phi, FuzzyToken::Phi]);
        assert_eq!(rez, vec![FuzzyToken::Phi, FuzzyToken::Phi]);

        let rez = table.execute(vec![t!(0.0, 0.0, 1.0, 0.0, 0.0), t!(0.0, 1.0, 0.0, 0.0 , 0.0) ]);
        assert_eq!(rez, vec![t!(0.0, 1.0, 0.0, 0.0, 0.0),t!(0.0, 1.0, 0.0, 0.0, 0.0)]);
        let rez = table.execute(vec![t!(0.0, 0.5, 0.5, 0.0, 0.0), t!(0.5, 0.5, 0.0, 0.0 , 0.0) ]);
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

        let rez = table.execute(vec![t!(0.6, 0.4, 0.0, 0.0, 0.0), t!(0.0, 0.3, 0.0, 0.7, 0.0)]);
        assert_eq!(rez, vec![t!(0.25, 0.0, 0.0, 0.0, 0.75), t!(0.25, 0.0, 0.0, 0.0, 0.75)]);
    }


}
