#![no_std]

use core::mem::MaybeUninit;

pub trait BitAlloc {
    const CAP: usize;

    fn set(&mut self, idx: usize);
    fn clear(&mut self, idx: usize);
    fn get(&self, idx: usize) -> bool;

    fn clear_all(&mut self);
    fn count_ones(&self) -> usize;
    fn first_zero(&self) -> Option<usize>;
}

/// 固定 64 位位图（无需 const 泛型）
#[derive(Default)]
pub struct Bitmap64 {
    map: [usize; 1],
}

impl Bitmap64 {
    pub fn new() -> Self {
        Self { map: [0] }
    }
}

impl BitAlloc for Bitmap64 {
    const CAP: usize = 64;

    fn set(&mut self, idx: usize) {
        self.map[0] |= 1 << idx;
    }

    fn clear(&mut self, idx: usize) {
        self.map[0] &= !(1 << idx);
    }

    fn get(&self, idx: usize) -> bool {
        (self.map[0] >> idx) & 1 != 0
    }

    fn clear_all(&mut self) {
        self.map[0] = 0;
    }

    fn count_ones(&self) -> usize {
        self.map[0].count_ones() as usize
    }

    fn first_zero(&self) -> Option<usize> {
        let val = self.map[0];
        for i in 0..64 {
            if (val & (1 << i)) == 0 {
                return Some(i);
            }
        }
        None
    }
}

pub fn default_array<T: Default, const N: usize>() -> [T; N] {
    let mut arr: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
    for i in 0..N {
        arr[i] = MaybeUninit::new(T::default());
    }
    unsafe { core::mem::transmute_copy::<_, [T; N]>(&arr) }
}

pub struct BitMapNode<T: BitAlloc> {
    inner: [T; 64],
}

impl<T: BitAlloc + Default> BitMapNode<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: default_array(),
        }
    }
}

impl<T: BitAlloc + Default> Default for BitMapNode<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: BitAlloc> BitAlloc for BitMapNode<T> {
    const CAP: usize = 64 * T::CAP;

    fn set(&mut self, idx: usize) {
        let i = idx / T::CAP;
        self.inner[i].set(idx % T::CAP);
    }

    fn clear(&mut self, idx: usize) {
        let i = idx / T::CAP;
        self.inner[i].clear(idx % T::CAP);
    }

    fn get(&self, idx: usize) -> bool {
        let i = idx / T::CAP;
        self.inner[i].get(idx % T::CAP)
    }

    fn clear_all(&mut self) {
        for elem in &mut self.inner {
            elem.clear_all();
        }
    }

    fn count_ones(&self) -> usize {
        self.inner.iter().map(|e| e.count_ones()).sum()
    }

    fn first_zero(&self) -> Option<usize> {
        for (i, e) in self.inner.iter().enumerate() {
            if let Some(bit) = e.first_zero() {
                return Some(i * T::CAP + bit);
            }
        }
        None
    }
}

pub type BitAlloc4K = BitMapNode<BitMapNode<Bitmap64>>;
pub type BitAlloc64K = BitMapNode<BitAlloc4K>;
pub type BitAlloc1M = BitMapNode<BitAlloc64K>;