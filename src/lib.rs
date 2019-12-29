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
                pub fn insert(&mut self, k: $key, v: $value) {
                    if self.is_full() {
                        panic!("Map is full");
                    }

                    if k > ArrayMap::size() {
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

                    if k > ArrayMap::size() {
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
                        map: self,
                        count: 0,
                        ptr: unsafe { std::ptr::NonNull::new_unchecked(std::ptr::null_mut()) },
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
                map: &'a mut ArrayMap,
                count: $key,
                ptr: std::ptr::NonNull<Option<$value>>,
            }

            impl<'a> Iterator for IterMut<'a> {
                type Item = ($key, &'a mut $value);

                fn next(&mut self) -> Option<Self::Item> {
                    while self.count < self.map.len() {
                        unsafe {
                            let k = self.map.get_key(self.count as usize);
                            self.ptr = std::ptr::NonNull::new_unchecked(
                                self.map.get_mut_value(k as usize),
                            );
                            let ptr = self.ptr.as_ptr();
                            let opt = &mut *ptr;
                            match opt {
                                Some(s) => {
                                    self.count += 1;
                                    return Some((k, s));
                                }
                                None => self.map.swap_remove_key(k as usize),
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
    use super::*;

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
    fn multiple_insert_remove() {
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
    }
}
