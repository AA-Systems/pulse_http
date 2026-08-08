use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

#[derive(Clone, Default, Debug)]
pub struct State {
    map: Arc<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<T: Send + Sync + 'static>(mut self, value: T) -> Self {
        let mut map = (*self.map).clone();
        map.insert(TypeId::of::<T>(), Arc::new(value));
        self.map = Arc::new(map);
        self
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|v| v.clone().downcast::<T>().ok())
    }
}
