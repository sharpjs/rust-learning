use std::collections::HashMap;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Symbol(usize);

impl From<usize> for Symbol {
    #[inline]
    fn from(n: usize) -> Self { Symbol(n) }
}

impl Into<usize> for Symbol {
    #[inline]
    fn into(self) -> usize { self.0 }
}

#[derive(Clone)]
pub struct Interner {
    map: HashMap <String, usize>,
    vec: Vec     <String>
}

impl Interner {
    pub fn new() -> Self {
        Interner {
            map: HashMap::new(),
            vec: Vec    ::new()
        }
    }

    pub fn intern<S: AsRef<str>>(&mut self, name: S) -> Symbol
    {
        let key = name.as_ref();

        // If a symbol exists by that name, return it
        if let Some(&idx) = self.map.get(key) {
            return idx.into();
        }

        // Else, build a new symbol and return it
        let idx = self.vec.len();
        self.vec.push   (key.into()     );
        self.map.insert (key.into(), idx);
        idx.into()
    }

    pub fn get(&self, id: Symbol) -> &str {
        &self.vec[id.0]
    }
}

#[test]
fn test() {
    let mut t = Interner::new();

    let a = t.intern("Hello");
    let b = t.intern(&"Hello".to_string());

    assert_eq!(a, b);
}

