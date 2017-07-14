use std;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::collections::BTreeMap;

use basic::*;
use tables::*;
use unified_petri_net::{UnifiedPetriNetBuilder,UnifiedTableE};

use self::NotExpectedJsonFormat::*;

extern crate rustc_serialize;
use self::rustc_serialize::json::Json;
use std::mem;

#[derive(Debug)]
pub enum NotExpectedJsonFormat {
    JsonKeyNotFound(&'static str),
    WrongNumberOfStuff(&'static str),
    WrongJsonValue(&'static str),
}

pub type Result<T> = std::result::Result<T, NotExpectedJsonFormat>;

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


pub fn my_file_read(fname: &str) -> String {
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
static IS_PHI: &'static str = "isPhi";
static TOKNE_VAL: &'static str = "val";
static SCALES: &'static str = "scaleForPlace";
static TR_TO_PL: &'static str = "fromTransToPlace";
static PL_TO_PL: &'static str = "fromPlaceToTrans";
static TBL_TRS: &'static str = "tableForTransition";
static TBL_TYPE: &'static str = "unfiedType";
static TBL_DATA: &'static str = "unifiedData";
static DELAY: &'static str = "delayForTransition";

static TABEL: &'static str = "table";
static OPERATOR: &'static str = "op";

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

pub fn deseralize(what :&str)-> Result<UnifiedPetriNetBuilder>  {
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

     let scales_json = mine!(obj, as_array, SCALES);
     let scales = mine_scale(scales_json, SCALES)?;

     let delay_jsons = mine!(obj, as_array, DELAY);
     let delays = mine_delays(delay_jsons, DELAY)?;
     assert_length!(delays, tr_nr, DELAY);

     let table_jsons  = mine!(obj, as_array, TBL_TRS);
     let mut tables = mine_tables(table_jsons)?;
     assert_length!(tables, tr_nr, TBL_TRS);



     let mut bld = UnifiedPetriNetBuilder::new();

     for tr_id in 0..tr_nr {
         if out_tr[tr_id] {
             bld.add_out_transition(extact_from_vec(&mut tables, tr_id));
         } else {
             bld.add_transition(delays[tr_id] as i32, extact_from_vec(&mut tables, tr_id));
         }
     }

     for pl_id in 0..pl_nr {
         if inp_pl[pl_id] {
             bld.add_inp_place(scales[pl_id] as f32);
         } else {
             bld.add_place(scales[pl_id] as f32);
         }
         bld.set_innitial_marking(pl_id, extract_from_token_map(&mut init_marking, pl_id));
     }

     for tr_id in 0..tr_to_pl.len() {
         for pl_id in &tr_to_pl[tr_id]{
             bld.connect_transition_with_place(tr_id, *pl_id);
         }
     }


     for pl_id in 0..pl_to_tr.len() {
         for tr_id in &pl_to_tr[pl_id] {
             bld.connect_place_with_transition(pl_id, * tr_id);
         }
     }


     Ok(bld)
}

fn extact_from_vec(vec :&mut Vec<UnifiedTableE>, tr_id:usize) -> UnifiedTableE {
    let replace_with = UnifiedTableE::oxo(UnifiedOneXOneTable::default_table());
    mem::replace(&mut vec[tr_id], replace_with)
}

fn extract_from_token_map(vec :&mut Vec<UnifiedToken>, pl_id: usize ) -> UnifiedToken {
    let replace_with = UnifiedToken::Phi;
    mem::replace(&mut vec[pl_id], replace_with)
}

fn mine_tables(table_jsons: &Vec<Json>) ->Result<Vec<UnifiedTableE>> {
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
            "u1x1" => mine_oxo(table_data_obj),
            "u2x1" => mine_txo(table_data_obj),
            "u1x2" => mine_oxt(table_data_obj),
            "u2x2" => mine_txt(table_data_obj),
            _     => {return Err(WrongJsonValue(TBL_TYPE))}
        };
        to_ret.push(table?);
    }
    Ok(to_ret)
}
static VAL_TBL: &'static str = "valTable";

fn mine_oxo(data :&BTreeMap<String, Json>) -> Result<UnifiedTableE> {
    let full_table = mine!(data, as_object, TABEL);
    let single_table = mine!(full_table, as_object, VAL_TBL);
    let mut table_arr = [TableValue::Phi; 6];
    for (key, val) in single_table {
        let value_str = val.as_string().ok_or(WrongJsonValue("fuzzy value"))?;
        let key_v = mine_table_val(key)?;
        let val_v = mine_table_val(value_str)?;
        table_arr[key_v.index()] = val_v;
    }

    Ok(UnifiedTableE::oxo(UnifiedOneXOneTable::from_arr(table_arr)))
}

static RULE_TBL: &'static str = "ruleTable";

fn mine_txo(data :&BTreeMap<String, Json>) -> Result<UnifiedTableE> {
    let full_table = mine!(data, as_object, TABEL);
    let single_table = mine!(full_table, as_object, RULE_TBL);
     let mut tbl = [TableValue::Phi; 36];
     for (big_index_str, value) in single_table {
        let big_index = mine_table_val(&big_index_str)?.index();
        let small_tbl = value.as_object().ok_or(WrongJsonValue("fuzzy value"))?;
        for (small_index_str, final_val_json) in small_tbl {
            let small_index = mine_table_val(&small_index_str)?.index();
            let final_fv = mine_table_val(final_val_json.as_string()
                                          .ok_or(WrongJsonValue("fuzzy value") )?)?;
            tbl[big_index*6 + small_index] = final_fv;
        }
     }

    let op_str = mine!(data, as_string, OPERATOR);
    let op = mine_operator(&op_str)?;
    Ok(UnifiedTableE::txo(UnifiedTwoXOneTable::from_arr(tbl, op)))
}

static VAL_TBL1: &'static str = "valTable1";
static VAL_TBL2: &'static str = "valTable2";

fn mine_oxt(json :&BTreeMap<String, Json>) -> Result<UnifiedTableE>{
     let data = mine!(json, as_object, TABEL);
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

    Ok(UnifiedTableE::oxt(UnifiedOneXTwoTable::from_arr(table_arr)))
}

static RULE_TBL_ONE: &'static str = "ruleTable1";
static RULE_TBL_TWO: &'static str = "ruleTable2";

fn mine_txt(data :&BTreeMap<String, Json>) -> Result<UnifiedTableE>{
    let full_table = mine!(data, as_object, TABEL);
    let first_table = mine!(full_table, as_object, RULE_TBL_ONE);
    let second_table = mine!(full_table, as_object, RULE_TBL_TWO);

     let mut tbl = [TableValue::Phi; 72];

     for (big_index_str, value) in first_table {
        let big_index = mine_table_val(&big_index_str)?.index();
        let small_tbl = value.as_object().ok_or(WrongJsonValue("fuzzy value"))?;
        for (small_index_str, final_val_json) in small_tbl {
            let small_index = mine_table_val(&small_index_str)?.index();
            let final_fv = mine_table_val(final_val_json.as_string()
                                          .ok_or(WrongJsonValue("fuzzy value") )?)?;
            tbl[(big_index*6 + small_index) * 2] = final_fv;
        }
     }

     for (big_index_str, value) in second_table {
        let big_index = mine_table_val(&big_index_str)?.index();
        let small_tbl = value.as_object().ok_or(WrongJsonValue("fuzzy value"))?;
        for (small_index_str, final_val_json) in small_tbl {
            let small_index = mine_table_val(&small_index_str)?.index();
            let final_fv = mine_table_val(final_val_json.as_string()
                                          .ok_or(WrongJsonValue("fuzzy value") )?)?;
            tbl[(big_index*6 + small_index) * 2 +1] = final_fv;
        }
     }

    let op_str = mine!(data, as_string, OPERATOR);
    let op = mine_operator(&op_str)?;
    Ok(UnifiedTableE::txt(UnifiedTwoXTwoTable::from_arr(tbl,op)))
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

fn mine_operator(what : &str) -> Result<Operator> {
     let rez = match what {
         "None" =>  Operator::NoOp,
         "PLUS" =>  Operator::Plus,
         "MINUS" =>  Operator::Minus,
         "MULT" =>  Operator::Mult,
         "DIV" =>  Operator::Div,
           _  => {return Err(WrongJsonValue("fuzzy value"))},
     };
     Ok(rez)
 }


fn mine_delays(scales: &Vec<Json>, talking_about: &'static str) -> Result<Vec<i64>> {
    scales.iter()
        .map(|inner_json| inner_json.as_i64().ok_or(WrongJsonValue(talking_about)))
        .collect()
}

fn mine_scale(scales: &Vec<Json>, talking_about: &'static str) -> Result<Vec<f64>> {
    scales.iter()
        .map(|inner_json| inner_json.as_f64().ok_or(WrongJsonValue(talking_about)))
        .collect()
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

fn mine_init_marking(inital_markings: &Vec<Json>) -> Result<Vec<UnifiedToken>> {
    let mut to_ret = Vec::new();
    for js_val in inital_markings {
        let val = js_val.as_object().ok_or(WrongJsonValue(INIT_PL))?;
        let phi = mine!(val, as_boolean,IS_PHI);
        if phi {
            to_ret.push(UnifiedToken::Phi);
        } else {
            let token_value = mine!(val, as_f64, TOKNE_VAL);
            to_ret.push(UnifiedToken::from_val(token_value as f32));
        }
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn max_try_run(){

        let ww = my_file_read("unified_nets/maxTableTryOut.json");
        let rez = deseralize(&ww);
        assert!(rez.is_ok());

        let bld = rez.unwrap();
        let (net, _) = bld.build();
        assert!(net.get_place_nr()==3);
        assert!(net.get_trans_nr()==2);
    }

}
