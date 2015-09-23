use std::borrow::Borrow;
use std::collections::HashMap;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Symbol(usize);

#[derive(Clone, Debug)]
pub struct SymbolInfo {
    pub name: String,
    // other fields here
}

pub struct SymbolTable {
    map: HashMap<String, Symbol>,
    vec: Vec<Box<SymbolInfo>>
}

impl SymbolTable {
    fn new() -> Self {
        SymbolTable {
            map: HashMap::new(),
            vec: Vec    ::new()
        }
    }

    fn intern<S>(&mut self, name: S) -> Symbol
        where S: Into<String>
    {
        // We own the name, thank you
        let name = name.into();

        // If a symbol exists by that name, return it
        if let Some(&idx) = self.map.get(&name) {
            return idx;
        }

        // Else, build a new symbol and return it
        let id  = Symbol(self.vec.len());
        let key = name.clone();
        let sym = Box::new(SymbolInfo {
            name: name,
            // other fields here
        });
        self.vec.push(sym);
        self.map.insert(key, id);
        id
    }

    fn get(&self, id: Symbol) -> &SymbolInfo {
        self.vec[id.0].borrow()
    }
}

#[test]
fn test() {
    let mut t = SymbolTable::new();

    let a = t.intern("Hello");
    let b = t.intern("Hello".to_string());

    assert_eq!(a, b);
}

