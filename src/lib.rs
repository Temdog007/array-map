#![feature(test)]

#[macro_export]
macro_rules! make_map {
    ($name:ident, $key:ty, $value:ty, $width:literal, $height:literal) => {
        pub mod $name {

            use serde::{Deserialize, Serialize};

            #[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
            pub struct ArrayMap {
                count: $key,
                keys: [[$key; $height]; $width],
                values: [[Option<$value>; $height]; $width],
            }

            impl ArrayMap {
                pub fn new() -> Self {
                    unsafe { std::mem::zeroed() }
                }
                pub fn insert(&mut self, k: $key, v: $value) {
                    if self.is_full() {
                        panic!("Map is full");
                    }

                    if k >= ArrayMap::size() {
                        panic!("Key {} is out of range", k);
                    }

                    unsafe {
                        self.set_key(self.count as usize, k);
                        self.set_value(k as usize, Some(v));
                    }
                    self.count += 1;
                }

                pub fn get(&self, k: $key) -> &Option<$value> {
                    unsafe { self.get_ref_value(k as usize) }
                }

                pub fn remove(&mut self, k: $key) -> Option<$value> {
                    if self.is_empty() {
                        return None;
                    }

                    if k >= ArrayMap::size() {
                        panic!("Key {} is out of range", k);
                    }

                    unsafe {
                        match self.get_value(k as usize) {
                            s @ Some(_) => {
                                self.set_value(k as usize, None);
                                self.count -= 1;
                                s
                            }
                            None => None,
                        }
                    }
                }

                pub fn clear(&mut self) {
                    self.count = 0;
                    unsafe { self.values = std::mem::zeroed() };
                }

                pub fn replace(&mut self, k: $key, v: $value) -> Option<$value> {
                    let old_v = self.remove(k);
                    self.insert(k, v);
                    old_v
                }

                pub fn is_full(&self) -> bool {
                    self.len() == ArrayMap::size()
                }

                pub fn is_empty(&self) -> bool {
                    self.len() == 0
                }

                pub fn len(&self) -> $key {
                    self.count
                }

                pub const fn size() -> $key {
                    $width * $height
                }

                unsafe fn set_key(&mut self, index: usize, k: $key) {
                    let ptr = &mut self.keys[0][0] as *mut $key;
                    *ptr.add(index) = k;
                }

                unsafe fn swap_remove_key(&mut self, index: usize) {
                    let ptr = &mut self.keys[0][0] as *mut $key;
                    let current = ptr.add(index);
                    let last = ptr.add(self.len() as usize - 1);
                    current.swap(last);
                    self.count -= 1;
                }

                unsafe fn get_key(&self, index: usize) -> $key {
                    let ptr = &self.keys[0][0] as *const $key;
                    *ptr.add(index)
                }

                unsafe fn set_value(&mut self, index: usize, v: Option<$value>) {
                    let ptr = &mut self.values[0][0] as *mut Option<$value>;
                    *ptr.add(index) = v;
                }

                unsafe fn get_value(&self, index: usize) -> Option<$value> {
                    let ptr = &self.values[0][0] as *const Option<$value>;
                    *ptr.add(index)
                }

                unsafe fn get_ref_value(&self, index: usize) -> &Option<$value> {
                    let ptr = &self.values[0][0] as *const Option<$value>;
                    &*ptr.add(index)
                }

                unsafe fn get_mut_value<'a>(&'a mut self, index: usize) -> *mut Option<$value> {
                    let ptr = &mut self.values[0][0] as *mut Option<$value>;
                    ptr.add(index)
                }

                pub fn iter(&self) -> impl Iterator<Item = ($key, &$value)> {
                    Iter {
                        map: self,
                        count: 0,
                    }
                }

                pub fn iter_mut(&mut self) -> impl Iterator<Item = ($key, &mut $value)> {
                    IterMut {
                        map: unsafe { std::ptr::NonNull::new_unchecked(self as *mut ArrayMap) },
                        count: 0,
                        _marker: std::marker::PhantomData,
                    }
                }
            }

            struct Iter<'a> {
                map: &'a ArrayMap,
                count: $key,
            }

            impl<'a> Iterator for Iter<'a> {
                type Item = ($key, &'a $value);

                fn next(&mut self) -> Option<Self::Item> {
                    while self.count < self.map.len() {
                        unsafe {
                            let k = self.map.get_key(self.count as usize);
                            self.count += 1;
                            if let Some(s) = self.map.get_ref_value(k as usize) {
                                return Some((k, s));
                            }
                        }
                    }
                    None
                }
            }

            struct IterMut<'a> {
                map: std::ptr::NonNull<ArrayMap>,
                count: $key,
                _marker: std::marker::PhantomData<&'a ArrayMap>,
            }

            impl<'a> Iterator for IterMut<'a> {
                type Item = ($key, &'a mut $value);

                fn next(&mut self) -> Option<Self::Item> {
                    unsafe {
                        let ptr = self.map.as_ptr();
                        let map = &mut *ptr;
                        while self.count < map.len() {
                            let k = map.get_key(self.count as usize);
                            let ptr = map.get_mut_value(k as usize);
                            let opt = &mut *ptr;
                            match opt {
                                Some(s) => {
                                    self.count += 1;
                                    return Some((k, s));
                                }
                                None => map.swap_remove_key(k as usize),
                            }
                        }
                    }
                    None
                }
            }
        }
    };
}

mod tests {
    extern crate test;
    use super::*;
    use test::Bencher;

    make_map!(Map, u8, i32, 8, 8);
    type TestMap = Map::ArrayMap;

    #[test]
    fn constructor() {
        let t: TestMap = Default::default();
        assert!(t.is_empty());
        assert_eq!(t.len(), 0);
        assert!(!t.is_full());
        assert_eq!(TestMap::size(), 64);
    }

    #[test]
    fn insert_remove() {
        let mut t: TestMap = Default::default();

        t.insert(5, 32);
        assert!(!t.is_empty());
        assert_eq!(t.len(), 1);
        assert_eq!(t.get(5), &Some(32));

        assert_eq!(t.remove(5), Some(32));
        assert!(t.is_empty());
    }

    #[test]
    fn iterator() {
        let mut t: TestMap = Default::default();

        for i in 0..64 {
            t.insert(i, i as i32 * 32);
        }

        for (k, v) in t.iter() {
            assert_eq!(*v, k as i32 * 32);
        }

        assert!(t.is_full());
        for i in (0..64).step_by(2) {
            assert_eq!(t.remove(i), Some(i as i32 * 32));
        }

        for (k, v) in t.iter() {
            assert_eq!(k % 2, 1);
            assert_eq!(*v, k as i32 * 32);
        }

        assert_eq!(t.len(), 32);

        for (_, v) in t.iter_mut() {
            *v *= 2;
        }

        for (k, v) in t.iter() {
            assert_eq!(*v, k as i32 * 64, "Bad value a {}", k);
        }

        t.clear();
        assert!(t.is_empty());
        assert_eq!(t.len(), 0);
        assert!(!t.is_full());

        for (_, _) in t.iter() {
            panic!("Iterator should be empty!");
        }
    }

    #[test]
    fn replace() {
        let mut t: TestMap = Default::default();

        t.insert(10, 3);
        assert_eq!(t.get(10), &Some(3));
        assert_eq!(t.replace(10, 1000), Some(3));
        assert_eq!(t.get(10), &Some(1000));
    }

    #[test]
    #[should_panic]
    fn bad_insert() {
        let mut t: TestMap = Default::default();

        t.insert(64, 0);
    }

    #[test]
    #[should_panic]
    fn bad_remove() {
        let mut t: TestMap = Default::default();

        t.insert(0, 0);
        t.remove(64);
    }

    #[test]
    #[should_panic]
    fn bad_insert2() {
        let mut t: TestMap = Default::default();

        for i in 0..TestMap::size() {
            t.insert(i, 0);
        }
        t.insert(60, 1);
    }

    #[bench]
    fn std_hash_map(b: &mut Bencher) {
        let mut map = std::collections::HashMap::<u16, u16>::with_capacity(1024);
        // println!("HashMap: {:?}", std::mem::size_of_val(&map));
        b.iter(move || {
            for i in 0..1024 {
                map.insert(i, i);
            }
            for i in (0..1024).step_by(3) {
                map.remove(&i);
            }
            for i in map.iter() {
                let _ = i;
            }

            map.clear();
        });
    }

    #[bench]
    fn std_hash_map_mut(b: &mut Bencher) {
        let mut map = std::collections::HashMap::<u16, u16>::with_capacity(1024);
        // println!("HashMap: {:?}", std::mem::size_of_val(&map));
        b.iter(move || {
            for i in 0..1024 {
                map.insert(i, i);
            }
            for i in (0..1024).step_by(3) {
                map.remove(&i);
            }
            for i in map.iter_mut() {
                let _ = i;
            }

            map.clear();
        });
    }

    #[bench]
    fn array_map_mut(b: &mut Bencher) {
        make_map!(BenchMap, u16, u16, 32, 32);
        let mut map = BenchMap::ArrayMap::new();
        // println!("BenchMap: {:?}", std::mem::size_of_val(&map));
        b.iter(move || {
            for i in 0..1024 {
                map.insert(i, i);
            }
            for i in (0..1024).step_by(3) {
                map.remove(i);
            }
            for i in map.iter_mut() {
                let _ = i;
            }

            map.clear();
        });
    }

    #[bench]
    fn array_map(b: &mut Bencher) {
        make_map!(BenchMap, u16, u16, 32, 32);
        let mut map = BenchMap::ArrayMap::new();
        // println!("BenchMap: {:?}", std::mem::size_of_val(&map));
        b.iter(move || {
            for i in 0..1024 {
                map.insert(i, i);
            }
            for i in (0..1024).step_by(3) {
                map.remove(i);
            }
            for i in map.iter() {
                let _ = i;
            }

            map.clear();
        });
    }
}
