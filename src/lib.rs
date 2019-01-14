//! A small library defining a pool of values which allows multiple indexing through
//! unique primary keys.
//! 
//! # Example
//! 
//! ```
//! use type_pool::TypePool;
//! 
//! let mut pool = TypePool::new();
//! let key1 = pool.insert(10,);
//! let key2 = pool.insert(1,);
//! let key3 = pool.insert(10,);
//! 
//! assert_eq!(pool[key1], 10);
//! assert_eq!(pool[key2], 1);
//! assert_eq!(pool[key3], 10);
//! 
//! let (keys, values) = pool.get_set([key1, key2, key3].iter().cloned().collect());
//! ```
//! 
//! Author --- daniel.bechaz@gmail.com  
//! Last Moddified --- 2019-01-14

#![deny(missing_docs,)]

use std::{
  hash, ops,
  num::NonZeroUsize,
  collections::{HashMap, HashSet,},
  marker::PhantomData,
};

#[macro_use]
extern crate subvert;

/// A key issued by a [TypePool].
pub struct PoolKey<T,>(usize, NonZeroUsize, PhantomData<T,>,);

impl<T,> PartialEq for PoolKey<T,> {
  #[inline]
  fn eq(&self, rhs: &Self,) -> bool {
    self.0 == rhs.0 && self.1 == rhs.1
  }
}

impl<T,> Eq for PoolKey<T,> {}

impl<T,> Clone for PoolKey<T,> {
  fn clone(&self,) -> Self { *self }
}

impl<T,> Copy for PoolKey<T,> {}

impl<T,> hash::Hash for PoolKey<T,> {
  #[inline]
  fn hash<H: hash::Hasher,>(&self, hasher: &mut H,) {
    self.0.hash(hasher,)
  }
}

/// A pool of `T` values.
pub struct TypePool<T,> {
  pool: Box<HashMap<usize, T,>>,
  next_id: usize,
}

impl<T,> TypePool<T,> {
  /// Returns a new empty TypePool.
  #[inline]
  pub fn new() -> Self {
    Self {
      pool: Box::new(HashMap::new()),
      next_id: 0,
    }
  }
  /// Returns `true` if `key` is owned by this TypePool.
  #[inline]
  pub fn owns_key(&self, key: &PoolKey<T,>,) -> bool {
    key.1.get() == self as *const Self as usize
  }
  /// Returns `true` if this TypePool contains `key`.
  #[inline]
  pub fn contains_key(&self, key: &PoolKey<T,>,) -> bool {
    self.owns_key(key,) && self.pool.contains_key(&key.0,)
  }
  /// Returns the number of values in this TypePool.
  #[inline]
  pub fn len(&self,) -> usize { self.pool.len() }
  /// Returns `true` the TypePool is empty.
  #[inline]
  pub fn is_empty(&self,) -> bool { self.len() == 0 }
  /// Inserts `value` into the TypePool.
  /// 
  /// Returns the [PoolKey] of the inserted value.
  pub fn insert(&mut self, value: T,) -> PoolKey<T,> {
    use std::usize;

    impl<T,> TypePool<T,> {
      fn get_next_id(&mut self,) -> usize {
        let id = (self.next_id..=usize::MAX)
          .chain(0..self.next_id,)
          .find(|key,| !self.pool.contains_key(key,),)
          .unwrap();
        
        self.next_id = id + 1;
        id
      }
    }

    assert_ne!(self.len(), usize::MAX, "`TypePool` is full",);

    let id = self.get_next_id();

    self.pool.insert(id, value,);

    PoolKey(id, unsafe { NonZeroUsize::new_unchecked(&mut self.pool as *const _ as usize,) }, PhantomData,)
  }
  /// Returns unique references too all the values referenced by `keys`.
  /// 
  /// The index of a [PoolKey] in the output is the index of the corresponding value.
  /// 
  /// # Params
  /// 
  /// keys --- The set of [PoolKey]s to get references too.  
  /// 
  /// # Panics
  /// 
  /// If any of the keys in `keys` are not in this TypePool.
  /// 
  /// # Example
  /// 
  /// ```
  /// use type_pool::TypePool;
  /// 
  /// let mut pool = TypePool::new();
  /// let key1 = pool.insert(10,);
  /// let key2 = pool.insert(1,);
  /// 
  /// let (keys, values) = pool.get_set([key1, key2].iter().cloned().collect());
  /// ```
  pub fn get_set(&mut self, keys: HashSet<PoolKey<T,>>,) -> (Box<[PoolKey<T,>]>, Box<[&mut T]>,) {
    let values = keys.iter()
      .cloned()
      .map(|key,| unsafe { steal!(&mut self[key]) },)
      .collect();
    let keys = keys.iter()
      .cloned()
      .collect();
    
    (keys, values,)
  }
}

impl<T,> TypePool<T,> {
  /// Inserts all of the values from `iter` into a new TypePool and returns the TypePool
  /// and the keys.
  /// 
  /// # Params
  /// 
  /// iter --- The values to insert.  
  pub fn from_iter<I,>(iter: I,) -> (Self, Box<[PoolKey<T,>]>,)
    where I: IntoIterator<Item = T>, {
    let iter = iter.into_iter();
    let mut pool = TypePool::new();
    let mut keys = {
      let cap = iter.size_hint();

      Vec::with_capacity(cap.1.unwrap_or(cap.0,),)
    };

    keys.extend(iter.map(|v,| pool.insert(v,),),);

    (pool, keys.into(),)
  }
}

impl<T,> Default for TypePool<T,> {
  #[inline]
  fn default() -> Self { Self::new() }
}

impl<T,> ops::Index<PoolKey<T,>> for TypePool<T,> {
  type Output = T;

  #[inline]
  fn index(&self, key: PoolKey<T,>,) -> &Self::Output {
    assert!(self.owns_key(&key,), "`TypePool::index` `key` must be issued from the pool",);

    self.pool.get(&key.0,).expect("`TypePool::index` `key` does not exist",)
  }
}

impl<T,> ops::IndexMut<PoolKey<T,>> for TypePool<T,> {
  #[inline]
  fn index_mut(&mut self, key: PoolKey<T,>,) -> &mut Self::Output {
    assert!(self.owns_key(&key,), "`TypePool::index_mut` `key` must be issued from the pool",);

    self.pool.get_mut(&key.0,).expect("`TypePool::index_mut` `key` does not exist",)
  }
}
