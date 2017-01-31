use std;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::collections::BTreeMap;

use basic::*;
use tables::*;

use self::NotExpectedJsonFormat::*;

extern crate rustc_serialize;
use self::rustc_serialize::json::Json;

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
static INIT_PL: &'static str = "initialMarkingOfThePlaces";
static PHI: &'static str = "phi";
static FV_S: &'static str = "fuzzyValues";
static TR_TO_PL: &'static str = "fromTransToPlace";
static PL_TO_PL: &'static str = "fromPlaceToTrans";
static WEIGTHS: &'static str = "weights";

macro_rules! mine {
    ($obj:ident, $fnc:ident, $idd: ident ) => {
         $obj.get($idd).ok_or(JsonKeyNotFound($idd))?
             .$fnc().ok_or(WrongJsonValue($idd))?;
    }
}



fn deseralize(what :&str) -> Result<i32> {
     let data = Json::from_str(what).unwrap();
     let obj = data.as_object().unwrap();

     let tr_nr = mine!(obj, as_u64, TR_NR) as usize;
     let pl_nr = mine!(obj, as_u64, PL_NR) as usize;

     let inp_pl_jsons = mine!(obj, as_array, INP_PL);
     let inp_pl = mine_is_inp_place(inp_pl_jsons)?;

     let inital_marking_jsons = mine!(obj, as_array, INIT_PL);
     let init_marking  = mine_init_marking(inital_marking_jsons)?;
     if init_marking.len() != pl_nr {return Err(WrongNumberOfStuff(INIT_PL))};

     let tr_to_pl_jsons  = mine!(obj, as_array, TR_TO_PL);
     let to_to_pl = mine_arcs(tr_to_pl_jsons, TR_TO_PL)?;
     if to_to_pl.len() != tr_nr {return Err(WrongNumberOfStuff(TR_TO_PL))};

     let pl_to_tr_jsons  = mine!(obj, as_array, PL_TO_PL);
     let pl_to_tr = mine_arcs(pl_to_tr_jsons, PL_TO_PL)?;
     if pl_to_tr.len() != pl_nr {return Err(WrongNumberOfStuff(PL_TO_PL))};

     let weigth_jsons  = mine!(obj, as_object, WEIGTHS);
     let weigths = mine_weigth(weigth_jsons)?;

     println!("{:?}", weigths);


     Ok(12)
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

fn mine_arcs( arc_jsons: &Vec<Json>,  talking_about :&'static str ) -> Result<Vec<Vec<usize>>> {
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

fn mine_is_inp_place(bools: &Vec<Json>) -> Result<Vec<bool>> {
    let mut to_ret = Vec::new();
    for v in bools {
        let unmined = v.as_boolean()
            .ok_or(WrongJsonValue(INP_PL))?;
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

    #[test]
    fn ww() {
        let ww = super::my_file_read("inputs/SimpleDelayPetriNet.json");
        let rez = super::deseralize(&ww);
        println!("{:?}", rez);
        assert!(true);
    }
}
