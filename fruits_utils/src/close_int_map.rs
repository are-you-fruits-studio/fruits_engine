// todo: ffi?
pub struct CloseIntMap<T> {
    inner: Vec<Option<Box<T>>>,
    offset: usize,
}

impl<T> CloseIntMap<T> {
    pub const fn new() -> Self {
        Self {
            inner: Vec::new(),
            offset: 0,
        }
    }

    pub fn insert(&mut self, k: usize, v: T) {
        if self.inner.len() == 0 {
            self.inner.push(Some(Box::new(v)));
            self.offset = k;
            return;
        }

        if k > self.offset {
            let idx = k - self.offset;

            if let Some(stored) = self.inner.get_mut(idx) {
                // todo: optimize
                *stored = Some(Box::new(v));
                return;
            } else {
                // todo: optimize
                while idx > self.inner.len() {
                    self.inner.push(None);
                }
                self.inner.push(Some(Box::new(v)));
                return;
            }
        }

        let temp_offset = self.offset - k;

        self.offset = k;

        // todo: optimize
        let old_inner = std::mem::replace(&mut self.inner, Vec::new());
        self.inner = std::iter::once(Some(Box::new(v))).chain(std::iter::repeat_with(|| None).take(temp_offset - 1)).chain(old_inner.into_iter()).collect();
    }

    pub fn get(&self, k: usize) -> Option<&T> {
        if k < self.offset {
            return None;
        }

        let idx = k - self.offset;

        let Some(stored) = self.inner.get(idx) else {
            return None;
        };
        
        stored.as_ref().map(|b| &**b)
    }

    pub fn get_mut(&mut self, k: usize) -> Option<&mut T> {
        if k < self.offset {
            return None;
        }

        let idx = k - self.offset;

        let Some(stored) = self.inner.get_mut(idx) else {
            return None;
        };
        
        stored.as_mut().map(|b| &mut **b)
    }
}