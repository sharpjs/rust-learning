use symbol::Symbol;

#[derive(Debug, PartialEq, Eq, Clone /*, Copy*/ )]
pub enum Token {
    Id      (Symbol),
    IntLit  (u64),
}

