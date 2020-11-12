use crate::droplet::Droplet;
use std::collections::VecDeque;
use std::ops::{Index, IndexMut};

pub struct Droplets {
    droplets: Vec<Droplet>,
    unused: VecDeque<usize>,
}

impl Droplets {
    pub fn new() -> Self {
        Droplets {
            droplets: Vec::new(),
            unused: VecDeque::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut result = Droplets {
            droplets: Vec::with_capacity(capacity),
            unused: VecDeque::with_capacity(capacity),
        };

        for i in 0..capacity {
            let mut d = Droplet::new();
            d.deleted = true;

            result.droplets.push(d);
            result.unused.push_back(i);
        }

        result
    }

    pub fn checkout(&mut self) -> Option<(usize, &mut Droplet)> {
        if let Some(unused) = self.unused.pop_front() {
            let droplet = &mut self.droplets[unused];

            droplet.deleted = false;

            return Some((unused, droplet));
        }

        None
    }

    pub fn free(&mut self, index: usize) {
        let droplet = &mut self.droplets[index];

        droplet.deleted = true;

        self.unused.push_back(index);
    }

    pub fn len(&self) -> usize {
        self.droplets.len()
    }
}

impl<'a> IntoIterator for &'a Droplets {
    type Item = &'a Droplet;
    type IntoIter = std::slice::Iter<'a, Droplet>;

    fn into_iter(self) -> Self::IntoIter {
        self.droplets.iter()
    }
}

impl<'a> IntoIterator for &'a mut Droplets {
    type Item = &'a mut Droplet;
    type IntoIter = std::slice::IterMut<'a, Droplet>;

    fn into_iter(self) -> Self::IntoIter {
        self.droplets.iter_mut()
    }
}

impl Index<usize> for Droplets {
    type Output = Droplet;

    fn index(&self, index: usize) -> &Self::Output {
        &self.droplets[index]
    }
}

impl IndexMut<usize> for Droplets {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.droplets[index]
    }
}
