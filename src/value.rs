use iroha_data_model::prelude::*;
use iroha_primitives::fixed::Fixed;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::str::FromStr;

pub struct ValueWrapper(Value);

impl ValueWrapper {
    pub fn inner(self) -> Value {
        self.0
    }
}

impl Distribution<ValueWrapper> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ValueWrapper {
        let value = match rng.gen_range(0..=6) {
            0 => Value::Numeric(NumericValue::U32(rng.gen())),
            1 => Value::Numeric(NumericValue::U128(rng.gen())),
            2 => Value::Bool(rng.gen()),
            3 => Value::String(format!("hello{}", rng.gen::<usize>())),
            4 => Value::Name(
                Name::from_str(format!("bob{}", rng.gen::<usize>()).as_str()).expect("Valid name"),
            ),
            5 => Value::Numeric(NumericValue::Fixed(
                Fixed::try_from(rng.gen::<f64>()).expect("Valid float num"),
            )),
            6 => {
                let len = rng.gen_range(0..=10);
                let mut vec = Vec::with_capacity(len);
                for _ in 0..len {
                    vec.push(Value::Numeric(NumericValue::U32(rng.gen())));
                }
                Value::Vec(vec)
            }
            _ => unreachable!(),
        };

        ValueWrapper(value)
    }
}
