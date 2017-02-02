use std;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::collections::BTreeMap;

use basic::*;
use tables::*;
use petri_net::{FuzzyPetriNetBuilder,FuzzyTableE};

use self::NotExpectedJsonFormat::*;

extern crate rustc_serialize;
use self::rustc_serialize::json::Json;
use std::mem;

#[derive(Debug)]
enum NotExpectedJsonFormat {
    JsonKeyNotFound(&'static str),
    WrongNumberOfStuff(&'static str),
    WrongJsonValue(&'static str),
}

type Result<T> = std::result::Result<T, NotExpectedJsonFormat>;

impl fmt::Display for NotExpectedJsonFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            JsonKeyNotFound(ref key) =>
                write!(f, "Json key not found {}", key),
            WrongNumberOfStuff(ref key) =>
                write!(f, "wrong number of stuff {}", key ),
            WrongJsonValue(ref key) =>
                write!(f, "wrong json value {}", key),

        }
    }
}


fn my_file_read(fname: &str) -> String {
    let path = Path::new(fname);
    let mut f = File::open(&path)
        .expect("File reading problem");
    let mut s = String::new();
    f.read_to_string(&mut s).expect("Somthing went reaaly wrong");
    s
}
static TR_NR: &'static str = "transitionCntr";
static PL_NR: &'static str = "placeCntr";
static INP_PL: &'static str = "isInputPlaces";
static OUT_TR: &'static str = "isOutputTransition";
static INIT_PL: &'static str = "initialMarkingOfThePlaces";
static PHI: &'static str = "phi";
static FV_S: &'static str = "fuzzyValues";
static TR_TO_PL: &'static str = "fromTransToPlace";
static PL_TO_PL: &'static str = "fromPlaceToTrans";
static WEIGTHS: &'static str = "weights";
static TBL_TRS: &'static str = "tableForTransition";
static TBL_TYPE: &'static str = "type";
static TBL_DATA: &'static str = "data";
macro_rules! mine {
    ($obj:ident, $fnc:ident, $idd: ident ) => {
         $obj.get($idd).ok_or(JsonKeyNotFound($idd))?
             .$fnc().ok_or(WrongJsonValue($idd))?;
    }
}

macro_rules! assert_length {
    ($vecc: ident, $len: ident, $err_hint: ident ) => {
            if $vecc.len() != $len {return Err(WrongNumberOfStuff($err_hint))};
    }
}


fn deseralize(what :&str) -> Result<FuzzyPetriNetBuilder> {
     let data = Json::from_str(what).unwrap();
     let obj = data.as_object().unwrap();

     let tr_nr = mine!(obj, as_u64, TR_NR) as usize;
     let pl_nr = mine!(obj, as_u64, PL_NR) as usize;

     let inp_pl_jsons = mine!(obj, as_array, INP_PL);
     let inp_pl = mine_bool_vec(inp_pl_jsons, INP_PL)?;
     assert_length!(inp_pl, pl_nr, INP_PL);

     let out_tr_jsons = mine!(obj, as_array, OUT_TR);
     let out_tr = mine_bool_vec(out_tr_jsons, OUT_TR)?;
     assert_length!(out_tr, tr_nr, OUT_TR);

     let inital_marking_jsons = mine!(obj, as_array, INIT_PL);
     let mut init_marking  = mine_init_marking(inital_marking_jsons)?;
     assert_length!(init_marking, pl_nr, INIT_PL);

     let tr_to_pl_jsons  = mine!(obj, as_array, TR_TO_PL);
     let tr_to_pl = mine_arcs(tr_to_pl_jsons, TR_TO_PL)?;
     assert_length!(tr_to_pl, tr_nr, TR_TO_PL);

     let pl_to_tr_jsons  = mine!(obj, as_array, PL_TO_PL);
     let pl_to_tr = mine_arcs(pl_to_tr_jsons, PL_TO_PL)?;
     assert_length!(pl_to_tr, pl_nr, PL_TO_PL);

     let weigth_jsons  = mine!(obj, as_object, WEIGTHS);
     let weigths = mine_weigth(weigth_jsons)?;
     assert_length!(weigths, pl_nr, WEIGTHS);

     let table_jsons  = mine!(obj, as_array, TBL_TRS);
     let mut tables = mine_tables(table_jsons)?;
     assert_length!(tables, tr_nr, TBL_TRS);


     let mut bld = FuzzyPetriNetBuilder::new();

     for tr_id in 0..tr_nr {
         if out_tr[tr_id] {
             bld.add_out_trans(extact_from_vec(&mut tables, tr_id));
         } else {
             //TODO where the fuck are the delays ??
             bld.add_trans(0,extact_from_vec(&mut tables, tr_id));
         }
     }

     for pl_id in 0..pl_nr {
         if inp_pl[pl_id] {
             bld.add_inp_place();
         } else {
             bld.add_place();
         }
         bld.set_initial_token(pl_id, extract_from_token_map(&mut init_marking, pl_id));
     }

     for (pl_id, tr_id, weight) in weigths {
         bld.add_arc_from_place_to_trans(pl_id, tr_id, weight);
     }

     for tr_id in 0..tr_to_pl.len() {
         for pl_id in &tr_to_pl[tr_id] {
             bld.add_arc_from_trans_to_place(tr_id, *pl_id);
         }
     }

     Ok(bld)
}

fn extact_from_vec(vec :&mut Vec<FuzzyTableE>, tr_id:usize) -> FuzzyTableE {
    let replace_with = FuzzyTableE::oxo(OneXOneTable::default_table());
    mem::replace(&mut vec[tr_id], replace_with)
}

fn extract_from_token_map(vec :&mut Vec<FuzzyToken>, pl_id: usize ) -> FuzzyToken {
    let replace_with = FuzzyToken::Phi;
    mem::replace(&mut vec[pl_id], replace_with)
}

fn mine_tables(table_jsons: &Vec<Json>) ->Result<Vec<FuzzyTableE>> {
    let mut to_ret = Vec::new();
    for table_json in table_jsons {
        let table_obj = table_json.as_object().ok_or(WrongJsonValue(TBL_TRS))?;
        let table_type = table_obj
            .get(TBL_TYPE).ok_or(WrongJsonValue(TBL_TYPE))?
            .as_string().ok_or(WrongJsonValue(TBL_TYPE))?;
        let table_data_obj = table_obj
            .get(TBL_DATA).ok_or(WrongJsonValue(TBL_DATA))?
            .as_object().ok_or(WrongJsonValue(TBL_DATA))?;
        let table = match table_type.as_ref() {
            "1x1" => mine_oxo(table_data_obj),
            "2x1" => mine_txo(table_data_obj),
            "1x2" => mine_oxt(table_data_obj),
            "2x2" => mine_txt(table_data_obj),
            _     => {return Err(WrongJsonValue(TBL_TYPE))}
        };
        to_ret.push(table?);
    }
    Ok(to_ret)
}

 static VAL_TBL: &'static str = "valTable";

 fn mine_oxo(data : &BTreeMap<String, Json> ) -> Result<FuzzyTableE> {
     let value_table = mine!(data, as_object, VAL_TBL);
     let mut table_arr = [TableValue::Phi; 6];

     for (key, value) in value_table {
         let value_str = value.as_string().ok_or(WrongJsonValue("fuzzy value"))?;
         let key_table_value = mine_table_val(&key)?;
         let value_table_value = mine_table_val(&value_str)?;
         table_arr[key_table_value.index()] = value_table_value;
     }
     Ok(FuzzyTableE::oxo(OneXOneTable::from_arr(table_arr)))
 }

 static RULE_TBL: &'static str = "ruleTable";
 fn mine_txo(data : &BTreeMap<String, Json> ) -> Result<FuzzyTableE> {
     let value_table = mine!(data, as_object, RULE_TBL);
     let mut tbl = [TableValue::Phi; 36];
     for (big_index_str, value) in value_table {
        let big_index = mine_table_val(&big_index_str)?.index();
        let small_tbl = value.as_object().ok_or(WrongJsonValue("fuzzy value"))?;
        for (small_index_str, final_val_json) in small_tbl {
            let small_index = mine_table_val(&small_index_str)?.index();
            let final_fv = mine_table_val(final_val_json.as_string().ok_or(WrongJsonValue("fuzzy value") )?)?;
            tbl[big_index*6 + small_index] = final_fv;
        }
     }
     Ok(FuzzyTableE::txo(TwoXOneTable::from_arr(tbl)))
 }

 static VAL_TBL1: &'static str = "valTable1";
 static VAL_TBL2: &'static str = "valTable2";

 fn mine_oxt(data : &BTreeMap<String, Json> ) -> Result<FuzzyTableE> {
     let value_table1 = mine!(data, as_object, VAL_TBL1);
     let value_table2 = mine!(data, as_object, VAL_TBL2);
     let mut table_arr = [TableValue::Phi; 12];

     for (key, value) in value_table1 {
         let value_str = value.as_string().ok_or(WrongJsonValue("fuzzy value"))?;
         let key_table_value = mine_table_val(&key)?;
         let value_table_value = mine_table_val(&value_str)?;
         table_arr[key_table_value.index()*2] = value_table_value;
     }

     for (key, value) in value_table2 {
         let value_str = value.as_string().ok_or(WrongJsonValue("fuzzy value"))?;
         let key_table_value = mine_table_val(&key)?;
         let value_table_value = mine_table_val(&value_str)?;
         table_arr[key_table_value.index()*2 +1 ] = value_table_value;
     }

     Ok(FuzzyTableE::oxt(OneXTwoTable::from_arr(table_arr)))
 }
 static RULE_TBL1: &'static str = "ruleTable1";
 static RULE_TBL2: &'static str = "ruleTable2";
 fn mine_txt(data : &BTreeMap<String, Json> ) -> Result<FuzzyTableE> {

     let value_table1 = mine!(data, as_object, RULE_TBL1);
     let value_table2 = mine!(data, as_object, RULE_TBL2);
     let mut tbl = [TableValue::Phi; 72];

     for (big_index_str, value) in value_table1 {
        let big_index = mine_table_val(&big_index_str)?.index();
        let small_tbl = value.as_object().ok_or(WrongJsonValue("fuzzy value"))?;
        for (small_index_str, final_val_json) in small_tbl {
            let small_index = mine_table_val(&small_index_str)?.index();
            let final_fv = mine_table_val(final_val_json.as_string().ok_or(WrongJsonValue("fuzzy value") )?)?;
            tbl[(big_index*6 + small_index) * 2] = final_fv;
        }
     }

     for (big_index_str, value) in value_table2 {
        let big_index = mine_table_val(&big_index_str)?.index();
        let small_tbl = value.as_object().ok_or(WrongJsonValue("fuzzy value"))?;
        for (small_index_str, final_val_json) in small_tbl {
            let small_index = mine_table_val(&small_index_str)?.index();
            let final_fv = mine_table_val(final_val_json.as_string().ok_or(WrongJsonValue("fuzzy value") )?)?;
            tbl[(big_index*6 + small_index) * 2 +1] = final_fv;
        }
     }


     Ok(FuzzyTableE::txt(TwoXTwoTable::from_arr(tbl)))
 }

 fn mine_table_val(what : &str) -> Result<TableValue> {
     let rez = match what {
         "NL" =>  TableValue::E(FuzzyValue::NL),
         "NM" =>  TableValue::E(FuzzyValue::NM),
         "ZR" =>  TableValue::E(FuzzyValue::ZR),
         "PM" =>  TableValue::E(FuzzyValue::PM),
         "PL" =>  TableValue::E(FuzzyValue::PL),
         "FF" =>  TableValue::Phi,
           _  => {return Err(WrongJsonValue("fuzzy value"))},
     };
     Ok(rez)
 }

fn mine_weigth(weigth: &BTreeMap<String, Json>) -> Result<Vec<(usize, usize, f32)>> {
    let mut to_ret = Vec::new();
    for (key, value) in weigth.iter() {
        let from = key.parse::<usize>()
            .map_err(|_|WrongJsonValue(WEIGTHS) )?;
        let value_obj = value.as_object()
            .ok_or(WrongJsonValue(WEIGTHS))?;
        for (small_key, smal_val) in value_obj {
            let aim = small_key.parse::<usize>()
                .map_err(|_|WrongJsonValue(WEIGTHS) )?;
            let weight = smal_val.as_f64().
                ok_or(WrongJsonValue(WEIGTHS))? as f32;
            to_ret.push((from, aim, weight));
        }
    }
    Ok(to_ret)
}

fn mine_arcs( arc_jsons: &Vec<Json>,  talking_about: &'static str ) -> Result<Vec<Vec<usize>>> {
    let mut to_ret = Vec::new();
    for inner_json in arc_jsons {
        let unmined = inner_json.as_array()
            .ok_or(WrongJsonValue(talking_about))?;
        let mut to_push = Vec::new();
        for one_val_json in unmined {
            let one_val = one_val_json.as_u64()
                .ok_or(WrongJsonValue(talking_about))?;
            to_push.push(one_val as usize);
        }
        to_ret.push(to_push);


    }
    Ok(to_ret)
}

fn mine_bool_vec(bools: &Vec<Json>, talking_about: &'static str) -> Result<Vec<bool>> {
    let mut to_ret = Vec::new();
    for v in bools {
        let unmined = v.as_boolean()
            .ok_or(WrongJsonValue(talking_about))?;
        to_ret.push(unmined);
    }
    Ok(to_ret)
}

fn mine_init_marking(inital_markings: &Vec<Json>) -> Result<Vec<FuzzyToken>> {
    let mut to_ret = Vec::new();
    for js_val in inital_markings {
        let val = js_val.as_object().ok_or(WrongJsonValue(INIT_PL))?;
        let phi = mine!(val, as_boolean,PHI);
        if phi {
            to_ret.push(FuzzyToken::Phi);
        } else {
            let fuzzy_vals = mine!(val, as_array, FV_S);
            if fuzzy_vals.len() !=5 {return Err(WrongNumberOfStuff(FV_S))};

            let mut empty_token = FuzzyToken::Phi;

            for fv  in FuzzyValue::iter() {
                let  value_for_fv = fuzzy_vals[fv.index()].as_f64()
                    .ok_or(WrongJsonValue(FV_S)) ?;
                empty_token.add_to_val(*fv, value_for_fv as f32);
            }
            to_ret.push(empty_token);
        }
    }
    Ok(to_ret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use petri_net::DotStringBuilder;



    #[test]
    fn ww() {
        let ww = super::my_file_read("inputs/SimpleDelayPetriNet.json");
        let rez = super::deseralize(&ww).unwrap();
        let (net, _) = rez.build();
        let dot_bld = DotStringBuilder::build(&net);
        //dot_bld.write_to_file("hey.txt");


        //println!("{:?}", rez);
        assert!(true);
    }
}
