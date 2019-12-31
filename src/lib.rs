#![feature(test)]

#[macro_export]
macro_rules! make_map {
    ($name:ident, $key:ty, $value:ty, $width:literal, $height:literal) => {
        pub mod $name {

            use serde::{Deserialize, Serialize};

            #[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
            struct Entry {
                key: $key,
                value: $value,
            }

            #[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
            pub struct ArrayMap {
                count: $key,
                keys: [[$key; $height]; $width],
                values: [[Option<Entry>; $height]; $width],
            }

            impl ArrayMap {
                pub fn new() -> Self {
                    unsafe { std::mem::zeroed() }
                }
                pub fn insert(&mut self, k: $key, v: $value) {
                    if self.is_full() {
                        panic!("Map is full");
                    }

                    unsafe {
                        let mut insert_k = k % ArrayMap::size();
                        while self.get_ref_value(insert_k as usize).is_some() {
                            insert_k = insert_k.wrapping_add(1);
                        }
                        self.set_key(self.count as usize, k);
                        self.set_value(insert_k as usize, Some(Entry { key: k, value: v }));
                    }
                    self.count += 1;
                }

                pub fn get(&self, mut k: $key) -> Option<&$value> {
                    let mut insert_k = k % ArrayMap::size();
                    loop {
                        match unsafe { self.get_ref_value(insert_k as usize) } {
                            Some(entry) if entry.key == k => return Some(&entry.value),
                            Some(_) => insert_k = insert_k.wrapping_add(1),
                            None => return None,
                        }
                    }
                }

                pub fn remove(&mut self, mut k: $key) -> Option<$value> {
                    if self.is_empty() {
                        return None;
                    }

                    k = k % ArrayMap::size();

                    unsafe {
                        loop {
                            match self.get_value(k as usize) {
                                Some(entry) if entry.key == k => {
                                    self.set_value(k as usize, None);
                                    self.count -= 1;
                                    return Some(entry.value);
                                }
                                Some(_) => k = k.wrapping_add(1),
                                None => return None,
                            }
                        }
                    }
                }

                pub fn clear(&mut self) {
                    self.count = 0;
                    unsafe { self.values = std::mem::zeroed() };
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

                unsafe fn set_value(&mut self, index: usize, v: Option<Entry>) {
                    let ptr = &mut self.values[0][0] as *mut Option<Entry>;
                    *ptr.add(index) = v;
                }

                unsafe fn get_value(&self, index: usize) -> Option<Entry> {
                    let ptr = &self.values[0][0] as *const Option<Entry>;
                    *ptr.add(index)
                }

                unsafe fn get_ref_value(&self, index: usize) -> &Option<Entry> {
                    let ptr = &self.values[0][0] as *const Option<Entry>;
                    &*ptr.add(index)
                }

                unsafe fn get_mut_value<'a>(&'a mut self, index: usize) -> *mut Option<Entry> {
                    let ptr = &mut self.values[0][0] as *mut Option<Entry>;
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
                            if let Some(entry) = self.map.get_ref_value(k as usize) {
                                return Some((entry.key, &entry.value));
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
                                Some(entry) => {
                                    self.count += 1;
                                    return Some((entry.key, &mut entry.value));
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

    use std::collections::HashMap;

    make_map!(Map, u32, u32, 8, 8);
    type TestMap = Map::ArrayMap;
    make_map!(Map2, u16, u16, 32, 32);
    type BenchMap = Map2::ArrayMap;

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
        assert_eq!(t.get(5), Some(&32));

        assert_eq!(t.remove(5), Some(32));
        assert!(t.is_empty());
    }

    #[test]
    fn iterator() {
        let mut t: TestMap = Default::default();

        for i in 0..64 {
            t.insert(i, i * 32);
        }

        for (k, v) in t.iter() {
            assert_eq!(*v, k * 32);
        }

        assert!(t.is_full());
        for i in (0..64).step_by(2) {
            assert_eq!(t.remove(i), Some(i * 32));
        }

        for (k, v) in t.iter() {
            assert_eq!(k % 2, 1);
            assert_eq!(*v, k * 32);
        }

        assert_eq!(t.len(), 32);

        for (_, v) in t.iter_mut() {
            *v *= 2;
        }

        for (k, v) in t.iter() {
            assert_eq!(*v, k * 64, "Bad value a {}", k);
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
    fn same_key() {
        let mut t: TestMap = Default::default();

        for (k, v) in (0..10).map(|i| (i * TestMap::size(), i)) {
            t.insert(k, v);
        }

        for (k, v) in (0..10).map(|i| (i * TestMap::size(), i)) {
            assert_eq!(t.get(k), Some(&v), "Map Bad with key {}: {:?}", k, t);
        }
    }

    trait InsertMap<K, V> {
        fn insert(&mut self, k: K, v: V);

        fn remove(&mut self, k: K);

        fn clear(&mut self);
    }

    impl InsertMap<u16, u16> for HashMap<u16, u16> {
        fn insert(&mut self, k: u16, v: u16) {
            self.insert(k, v);
        }
        fn remove(&mut self, k: u16) {
            self.remove(&k);
        }
        fn clear(&mut self) {
            self.clear();
        }
    }

    impl InsertMap<u16, u16> for BenchMap {
        fn insert(&mut self, k: u16, v: u16) {
            self.insert(k, v);
        }
        fn remove(&mut self, k: u16) {
            self.remove(k);
        }
        fn clear(&mut self) {
            self.clear();
        }
    }

    fn do_map_test1<T: InsertMap<u16, u16>>(map: &mut T) {
        for i in 0..256 {
            for j in 0..4 {
                map.insert(i, i * j);
            }
        }
        for i in (0..256).step_by(3) {
            map.remove(i);
        }

        map.clear();
    }

    fn do_map_test2<T: InsertMap<u16, u16>>(map: &mut T) {
        for i in 0..1024 {
            map.insert(i, i);
        }
        for i in (0..1024).step_by(3) {
            map.remove(i);
        }

        map.clear();
    }

    fn do_map_test3<T: InsertMap<u16, u16>>(map: &mut T) {
        for i in (0..1024).step_by(2) {
            map.insert(i, i);
        }
        for i in (0..1024).step_by(5) {
            map.insert(i, i);
        }

        map.clear();
    }

    #[bench]
    fn array_map_dup_keys(b: &mut Bencher) {
        let mut map = BenchMap::new();
        // println!("BenchMap: {:?}", std::mem::size_of_val(&map));
        b.iter(move || do_map_test1(&mut map));
    }

    #[bench]
    fn array_map_unique_keys(b: &mut Bencher) {
        let mut map = BenchMap::new();
        // println!("BenchMap: {:?}", std::mem::size_of_val(&map));
        b.iter(move || do_map_test2(&mut map));
    }

    #[bench]
    fn array_map_some_dup_keys(b: &mut Bencher) {
        let mut map = BenchMap::new();
        // println!("BenchMap: {:?}", std::mem::size_of_val(&map));
        b.iter(move || do_map_test3(&mut map));
    }

    #[bench]
    fn std_hash_map_dup_keys(b: &mut Bencher) {
        let mut map: HashMap<u16, u16> = HashMap::with_capacity(1024);
        // println!("BenchMap: {:?}", std::mem::size_of_val(&map));
        b.iter(move || do_map_test1(&mut map));
    }

    #[bench]
    fn std_hash_map_unique_keys(b: &mut Bencher) {
        let mut map: HashMap<u16, u16> = HashMap::with_capacity(1024);
        // println!("BenchMap: {:?}", std::mem::size_of_val(&map));
        b.iter(move || do_map_test2(&mut map));
    }

    #[bench]
    fn std_hash_map_some_keys(b: &mut Bencher) {
        let mut map: HashMap<u16, u16> = HashMap::with_capacity(1024);
        // println!("BenchMap: {:?}", std::mem::size_of_val(&map));
        b.iter(move || do_map_test3(&mut map));
    }
}
