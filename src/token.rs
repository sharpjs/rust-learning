
#[derive(Debug, PartialEq, Eq, Clone /*, Copy*/ )]
pub enum Token<'a> {
    Id  (String),
    Add (&'a Token<'a>, &'a Token<'a>)
}

