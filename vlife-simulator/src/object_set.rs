use indexmap::{map::Slice, IndexMap};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct ObjectHandle<T>(usize, PhantomData<fn() -> T>);

impl<T> Clone for ObjectHandle<T> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<T> Copy for ObjectHandle<T> {}

impl<T> PartialEq for ObjectHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        usize::eq(&self.0, &other.0)
    }
}

impl<T> Eq for ObjectHandle<T> {}

impl<T> Hash for ObjectHandle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        usize::hash(&self.0, state)
    }
}

pub struct ObjectSet<T> {
    next_id: usize,
    objects: IndexMap<usize, T>,
}

impl<T> ObjectSet<T> {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            objects: IndexMap::new(),
        }
    }

    // pub fn index_of(&self, handle: ObjectHandle<T>) -> Option<usize> {
    //     self.objects.split_off()
    //     self.objects.get_index_of(&handle.0)
    // }

    pub fn get_pair_mut(
        &mut self,
        handle1: ObjectHandle<T>,
        handle2: ObjectHandle<T>,
    ) -> Option<(&mut T, &mut T)> {
        let index1 = self.objects.get_index_of(&handle1.0);
        let index2 = self.objects.get_index_of(&handle2.0);
        if let Some((index1, index2)) = index1.zip(index2) {
            let slice = self.objects.as_mut_slice();
            if index1 <= index2 {
                let (left, right) = slice.split_at_mut(index1 + 1);
                Some((&mut left[index1], &mut right[index2 - index1 - 1]))
            } else {
                let (left, right) = slice.split_at_mut(index2 + 1);
                Some((&mut right[index1 - index2 - 1], &mut left[index2]))
            }
        } else {
            None
        }
    }

    pub fn get(&self, handle: ObjectHandle<T>) -> Option<&T> {
        self.objects.get(&handle.0)
    }

    pub fn get_mut(&mut self, handle: ObjectHandle<T>) -> Option<&mut T> {
        self.objects.get_mut(&handle.0)
    }

    pub fn insert(&mut self, object: T) -> ObjectHandle<T> {
        let id = self.next_id;
        self.next_id += 1;
        self.objects.insert(id, object);
        ObjectHandle(id, PhantomData)
    }

    pub fn remove(&mut self, handle: ObjectHandle<T>) -> Option<T> {
        self.objects.remove(&handle.0)
    }

    pub fn keys(&self) -> impl Iterator<Item = ObjectHandle<T>> + '_ {
        self.objects.keys().map(|id| ObjectHandle(*id, PhantomData))
    }

    pub fn iter(&self) -> impl Iterator<Item = (ObjectHandle<T>, &T)> {
        self.objects
            .iter()
            .map(|(id, object)| (ObjectHandle(*id, PhantomData), object))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (ObjectHandle<T>, &mut T)> {
        self.objects
            .iter_mut()
            .map(|(id, object)| (ObjectHandle(*id, PhantomData), object))
    }

    pub fn slice_mut(&mut self) -> &mut Slice<usize, T> {
        self.objects.as_mut_slice()
    }
}

impl<T> Default for ObjectSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

// pub struct Combinations<'a, T> {
//     elements: &'a mut Slice<usize, T>,
//     index1: usize,
//     index2: usize,
// }
//
// impl<'a, T> Iterator for Combinations<'a, T> {
//     type Item = (&'a mut T, &'a mut T);
//     fn next(&mut self) -> Option<Self::Item> {
//         let len = self.elements.len();
//         if self.index2 >= len {
//             self.index1 += 1;
//             self.index2 = self.index1 + 1;
//         }
//         if self.index1 >= len {
//             None
//         } else {
//             let (left, right) = self.elements.split_at_mut(self.index1 + 1);
//             Some((&mut left[self.index1], &mut right[self.index2 - self.index1 - 1]))
//         }
//     }
// }
