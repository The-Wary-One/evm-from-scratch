mod int256;

pub use int256::*;

pub trait State {}

#[derive(Debug)]
pub struct Init;
impl State for Init {}

#[derive(Debug)]
pub struct Ready;
impl State for Ready {}

#[derive(Debug)]
pub struct Completed;
impl State for Completed {}
