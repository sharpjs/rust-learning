use std::collections::HashMap;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct SymbolId(usize);

#[derive(Clone, Debug)]
pub struct Symbol {
    pub name: String,
    // other fields here
}

pub struct SymbolTable {
    map: HashMap<String, SymbolId>,
    vec: Vec<Box<Symbol>>
}

impl SymbolTable {
    fn new() -> SymbolTable {
        SymbolTable {
            map: HashMap::new(),
            vec: Vec    ::new()
        }
    }

    fn intern<S>(&mut self, name: S) -> SymbolId
        where S: Into<String>
    {
        let name = name.into();

        // If a symbol exists by that name, return it
        if let Some(&idx) = self.map.get(&name) {
            return idx;
        }

        // Else, build a new symbol and return it
        let id  = SymbolId(self.vec.len());
        let key = name.clone();
        let sym = Box::new(Symbol {
            name: name,
            // other fields here
        });
        self.vec.push(sym);
        self.map.insert(key, id);
        id
    }

    fn get(&self, id: SymbolId) -> &Symbol {
        &*self.vec[id.0]
    }
}

#[test]
fn test() {
    let mut t = SymbolTable::new();

    let a = t.intern("Hello");
    let b = t.intern("Hello");

    assert_eq!(a, b);
}

