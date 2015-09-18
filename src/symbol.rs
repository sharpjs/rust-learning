use std::collections::HashMap;

struct StringTable<'a> {
    map: HashMap<&'a str, &'a str>
}

impl<'a> StringTable<'a> {
    fn new() -> StringTable<'a> {
        StringTable { map: HashMap::new() }
    }

    fn intern(&mut self, s: &'a str) -> &'a str {
        let map = &mut self.map;
        if let Some(s) = map.get(&s) {
            return *s;
        }
        map.insert(s, s);
        s
    }
}

#[test]
fn test() {
    let a = "hello";
    let b = String::from("h") + "ello";

    let mut t = StringTable::new();

    //assert!("a".as_ptr() == "b".as_ptr());

    assert_eq!(t.intern( a).as_ptr(), a.as_ptr());
    assert_eq!(t.intern(&b).as_ptr(), a.as_ptr());
}

