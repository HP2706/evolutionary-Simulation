use crate::simulation::types::RoundState;
use polars::prelude::*;
use serde_json::{self, Value};

pub fn read_json<T>(filename: &str) -> Result<T, serde_json::Error> 
where
    T: for<'de> serde::de::Deserialize<'de>, // Add this trait bound
{
    let contents = std::fs::read_to_string(filename).expect("Failed to read file");
    let json: Value = serde_json::from_str(&contents)?;
    convert_value(json)
}

fn convert_value<T>(value: Value) -> Result<T, serde_json::Error>
where
    T: for<'de> serde::de::Deserialize<'de>, // Ensure this trait bound is here
{
    serde_json::from_value(value)
}
pub fn write_to_parquet(filename : &str){

}

