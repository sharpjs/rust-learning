#![allow(dead_code)]
#![allow(unused_variables)]

macro_rules! is {
    { $val:expr => $( $pat:pat ),* } => {
        match $val {
            $( $pat => true ),* ,
            _ => false
        }
    };
    { $val:expr => $( $pat:pat if $cond:expr ),* } => {
        match $val {
            $( $pat if $cond => true ),* ,
            _ => false
        }
    };
}

mod char;
mod lexer;
mod parser;
mod symbol;

fn main() {
}

