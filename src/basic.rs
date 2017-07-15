use std::slice::Iter;
use std::mem;
use self::FuzzyValue::*;
use self::FuzzyToken::*;

#[derive( PartialEq, Eq,Hash, Debug, Clone, Copy)]
pub enum FuzzyValue {
    NL,
    NM,
    ZR,
    PM,
    PL,
}

impl FuzzyValue {
    pub fn index(&self) -> usize {
        match *self {
            FuzzyValue::NL => 0,
            FuzzyValue::NM => 1,
            FuzzyValue::ZR => 2,
            FuzzyValue::PM => 3,
            FuzzyValue::PL => 4,
        }

    }

    pub fn iter() -> Iter<'static, FuzzyValue> {
        static FV : [FuzzyValue; 5]= [NL, NM, ZR, PM, PL];
        FV.into_iter()
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum FuzzyToken {
    Phi,
    Exist([f32; 5]),
}

#[derive(PartialEq, Debug, Clone)]
pub enum UnifiedToken{
    Phi,
    Exist(f32),
}

impl UnifiedToken {
    pub fn zero_token() -> UnifiedToken {
        UnifiedToken::Exist(0.0)
    }

    pub fn from_option(o : Option<f32>) -> UnifiedToken {
        match o {
            None => UnifiedToken::Phi ,
            Some(v) => {
                assert!(!v.is_nan());
                UnifiedToken::Exist(v)},
        }
    }

    pub fn from_val(val : f32) -> UnifiedToken {
        assert!(!val.is_nan());
        UnifiedToken::Exist(val)
    }

    pub fn as_option(&self) -> Option<f32>  {
        match *self {
            UnifiedToken::Phi => Option::None,
            UnifiedToken::Exist(v) => Option::Some(v)
        }
    }

    pub fn unite(&mut self, t : UnifiedToken) {
        match t {
            UnifiedToken::Phi => {/*do nothing*/},
            UnifiedToken::Exist(v) => {
                match *self {
                 UnifiedToken::Exist(ref mut val_place) => {*val_place = (*val_place +v )/2.0 ;},
                 UnifiedToken::Phi => {mem::replace(self, UnifiedToken::Exist(v));},
                }
            },
        }
    }
}

impl FuzzyToken {

    pub fn zero_token() -> FuzzyToken {
        let mut map = [0.0; 5];
        map[ZR.index()] = 1.0;
        Exist(map)
    }

    pub fn from_arr(ar: [f32;5]) -> FuzzyToken {
        Exist(ar)
    }


    pub fn add_to_val(&mut self, fv: FuzzyValue, to_add :f32) {
        match *self {
            Phi => {
                let mut map = [0.0; 5];
                map[fv.index()] = to_add;
                mem::replace(self, Exist(map));
            }
            Exist(ref mut map) => {
                map[fv.index()] += to_add;
            }
        }

    }

    pub fn unite(&mut self, ft : FuzzyToken) {
        match ft {
            Phi => {/*do nothing*/},
            Exist(map) => {
                for fv in FuzzyValue::iter(){
                    self.add_to_val(*fv, map[fv.index()])
                }
                self.normailze();
            }
        }
    }


    pub fn get_val(&self, fv : FuzzyValue) -> f32 {
        match *self {
            Phi => 0.0,
            Exist(ref arr) => {
                arr[fv.index()]
            }
        }
    }

    pub fn nonzero_values(&self) -> Vec<&FuzzyValue> {
        match *self {
            Phi => vec![],
            Exist(ref arr) => {
                let mut  to_ret = vec![];
                for fv in FuzzyValue::iter() {
                    if arr[fv.index()] != 0.0 {
                        to_ret.push(fv);
                    }
                }
                to_ret
            }
        }
    }

    pub fn normailze(&mut self) {
        match *self  {
            Phi => {},
            Exist(ref mut arr) => {
                let sum = arr.iter().fold(0.0, |acc, &x| acc +x);
                if sum.is_nan() {
                    for i in 0..5 {
                        arr[i] = 0.0;
                    }
                    arr[2] = 1.0;
                    return
                }
                for x in arr.iter_mut() {
                    *x =(*x) / sum;
                }
            }
        }
    }

}

pub trait Fuzzyfier {
    fn fuzzyfy(&self, x: Option<f32>) -> FuzzyToken;
}

pub trait Defuzzyfier {
    fn defuzzyfy(&self, tk: FuzzyToken) -> Option<f32>;
    fn limit(&self, v: f32) -> f32;
}

#[derive(PartialEq, Debug)]
pub struct TriangleFuzzyfier {
    limits : [[f32; 3]; 5],
}

impl TriangleFuzzyfier {
    pub fn with_border_vals(nl: f32, nm: f32, zr: f32, pm: f32, pl: f32) -> TriangleFuzzyfier {
        TriangleFuzzyfier{limits: [
            [-1.0, nl, nm],
            [ nl, nm, zr],
            [ nm, zr, pm],
            [ zr, pm, pl],
            [pm, pl, 1.0],
        ]}
    }

    pub fn with_min_max(min: f32, max: f32) -> TriangleFuzzyfier {
        let  step = (max - min) / 4.0;
        TriangleFuzzyfier::with_border_vals(min, min + step, min + 2.0 * step, min + 3.0 * step, max)
    }

    pub fn default() -> TriangleFuzzyfier {
        TriangleFuzzyfier::with_min_max(-1.0, 1.0)
    }

    fn calc_with_rigth(center: f32, right_limit: f32, value: f32) -> f32 {
        (right_limit - value) / (right_limit - center)
    }

    fn calc_with_left(center: f32, left_limit: f32, value: f32) -> f32 {
        (value - left_limit) / (center - left_limit)
    }

    fn  calc_in_midle(defuz_values:&[f32;3],  value : f32) -> f32 {

        if (defuz_values[0] >= value) || (defuz_values[2] <= value) {
           0.0
        } else if value == defuz_values[1] {
           1.0
        } else if value < defuz_values[1] {
           TriangleFuzzyfier::calc_with_left(defuz_values[1], defuz_values[0], value)
        } else {
           TriangleFuzzyfier::calc_with_rigth(defuz_values[1], defuz_values[2], value)
        }
  }
}
macro_rules! limits_of {
    ($self_:ident,$fuzzy_val:ident) => (
        $self_.limits[($fuzzy_val).index()]
                          )
}
const EPS: f32 = 0.00000000001;

impl Fuzzyfier for TriangleFuzzyfier {
    fn fuzzyfy(&self, x: Option<f32>) -> FuzzyToken {
        match x {
            None => Phi,
            Some(val) => {
                let mut ft =  Phi;
                if limits_of!(self,NL)[1] > val {
                     ft.add_to_val(NL, 1.0);
                } else if limits_of!(self,NL)[1]  <= val && limits_of!(self,NL)[2] >= val {
                        let rez = TriangleFuzzyfier::calc_with_rigth(
                            limits_of!(self,NL)[1], limits_of!(self,NL)[2], val);
                         ft.add_to_val(NL, rez);
                    }

                ft.add_to_val(NM, TriangleFuzzyfier::calc_in_midle(&limits_of!(self,NM), val));
                ft.add_to_val(ZR, TriangleFuzzyfier::calc_in_midle(&limits_of!(self,ZR), val));
                ft.add_to_val(PM, TriangleFuzzyfier::calc_in_midle(&limits_of!(self,PM), val));

                if limits_of!(self, PL)[1] < val {
                    ft.add_to_val(PL, 1.0);
                } else if limits_of!(self, PL)[1]  >= val && limits_of!(self, PL)[0]  <= val {
                        let rez = TriangleFuzzyfier::calc_with_left(
                            limits_of!(self, PL)[1], limits_of!(self, PL)[0], val);
                         ft.add_to_val(PL, rez);
                    }
                ft.normailze();
                ft
            }
        }
    }
}


impl Defuzzyfier for TriangleFuzzyfier {

    fn limit(&self, v: f32) -> f32 {
        if  v < limits_of!(self, NL)[1] {
            limits_of!(self, NL)[1]
        } else if  v > limits_of!(self, PL)[1] {
            limits_of!(self, PL)[1]
        } else {
            v
        }
    }

    fn defuzzyfy(&self, tk: FuzzyToken) -> Option<f32> {
        match tk {
            Phi => None ,
            Exist(_) => {
                let mut sum = 0.0;
                let mut weight_sum = 0.0;
                for fuzzy_value in FuzzyValue::iter() {
                    weight_sum += limits_of!(self,fuzzy_value)[1] * tk.get_val(*fuzzy_value);
                    sum +=tk.get_val(*fuzzy_value);

                }
                Some(weight_sum/ sum)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FuzzyToken,  TriangleFuzzyfier, Fuzzyfier, Defuzzyfier, UnifiedToken} ;
    use super::FuzzyToken::*;
    use super::FuzzyValue::*;

    #[test]
    fn TriangleFuzzyfier_zero_scale_test(){
        let ff = TriangleFuzzyfier::with_min_max(0.0*-1.0, 0.0);
        let rez = ff.fuzzyfy(Option::Some(0.0));
        assert_eq!(FuzzyToken::from_arr([0.0, 0.0, 1.0, 0.0, 0.0]), rez);
    }

    #[test]
    fn zero_token_test() {
        let ft = FuzzyToken::zero_token();
        assert_eq!(ft.get_val(ZR), 1.0);
        assert_eq!(ft.get_val(PL), 0.0);
    }
    #[test]
    fn unite_test(){
        let mut ft_one = FuzzyToken::from_arr([0.0, 1.0, 0.0, 0.0, 0.0]);
        let mut ft_two = FuzzyToken::from_arr([0.0, 0.0, 1.0, 1.0, 0.0]);
        ft_two.normailze();
        ft_one.unite(ft_two);
        assert_eq!(ft_one, Exist([0.00,0.50,0.25,0.25,0.00]));

        let mut ft_three = FuzzyToken::Phi;
        let ft_four = FuzzyToken::zero_token();
        ft_three.unite(ft_four);
        assert_eq!(ft_three, Exist([0.00,0.00,1.00,0.00,0.00]));

    }

    #[test]
    fn add_to_val_test() {
        let mut ft = Phi;
        ft.add_to_val(NL, 0.7);
        assert_eq!(ft.get_val(NL), 0.7);
        ft.add_to_val(NL, 0.5);
        assert_eq!(ft.get_val(NL), 1.2);
        ft.add_to_val(PM, 0.3);
        assert_eq!(ft, Exist([1.2, 0.0, 0.0, 0.3, 0.0]));
    }


    #[test]
    fn normialize_test() {
        let mut ft = FuzzyToken::from_arr([1.0,1.0,1.5,2.0,2.5]);
        ft.normailze();
        assert_eq!(ft, Exist([0.125, 0.125, 0.1875, 0.250, 0.3125]));
    }

    #[test]
    fn nonzero_values_test() {
        let  ft = FuzzyToken::from_arr([1.0,1.0,0.0,2.0,0.0]);
        assert_eq!(ft.nonzero_values(), vec![&NL, &NM, &PM]);
    }

    #[test]
    fn fuzzyfie_test() {
        let fuzzyfier = TriangleFuzzyfier::with_border_vals(-1.0, -0.5, 0.0, 0.5, 1.0);
        let rez_tk = fuzzyfier.fuzzyfy(Some(-1.2));
        assert_eq!(rez_tk, Exist([1.0, 0.0, 0.0, 0.0, 0.0]));
        let rez_tk = fuzzyfier.fuzzyfy(Some(1.2));
        assert_eq!(rez_tk, Exist([0.0, 0.0, 0.0, 0.0, 1.0]));
        let rez_tk = fuzzyfier.fuzzyfy(Some(0.75));
        assert_eq!(rez_tk, Exist([0.0, 0.0, 0.0, 0.5, 0.5]));
        let rez_tk = fuzzyfier.fuzzyfy(Some(-0.25));
        assert_eq!(rez_tk, Exist([0.0, 0.5, 0.5, 0.0, 0.0]));
        let rez_tk = fuzzyfier.fuzzyfy(Some(-0.20));
        assert_eq!(rez_tk, Exist([0.0, 0.4, 0.6, 0.0, 0.0]));
        let rez_tk = fuzzyfier.fuzzyfy(None);
        assert_eq!(rez_tk, Phi);
    }

    #[test]
    fn defuzzy_test() {
        let fuzzyfier = TriangleFuzzyfier::with_border_vals(-1.0, -0.5, 0.0, 0.5, 1.0);
        let rez = fuzzyfier.defuzzyfy(Phi);
        assert_eq!(rez, None);
        let rez =fuzzyfier.defuzzyfy(Exist([1.0, 0.0, 0.0, 0.0, 0.0]));
        assert_eq!(rez, Some(-1.0));
        let rez =fuzzyfier.defuzzyfy(Exist([0.0, 0.0, 0.5, 0.5, 0.0]));
        assert_eq!(rez, Some(0.25));
    }

    #[test]
    fn min_max_test() {
        let fuzzyfier = TriangleFuzzyfier::with_border_vals(-2.0, -1.0, 0.0, 1.0, 2.0);
        let fuzzyfier_second = TriangleFuzzyfier::with_min_max(-2.0, 2.0);
        assert_eq!(fuzzyfier, fuzzyfier_second);
    }

    #[test]
    fn unified_token_test() {
        let mut phi =UnifiedToken::Phi;
        let mut zero = UnifiedToken::zero_token();
        phi.unite(zero);
        assert_eq!(phi, UnifiedToken::Exist(0.0));
    }

    #[test]
    fn unified_token_test_as_option() {
        let  phi =UnifiedToken::Phi;
        let  zero = UnifiedToken::zero_token();
        assert_eq!(Option::None, phi.as_option());
        assert_eq!(Option::Some(0.0), zero.as_option());
    }

}
