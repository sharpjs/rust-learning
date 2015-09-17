
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Token<'a> {
    Id  (&'a str),
    Add (&'a Token<'a>, &'a Token<'a>)
}

