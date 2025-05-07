use bevy::prelude::*;

pub struct Node<A: Asset + Sized> {
    handle: Handle<A>,
    next: Option<Box<dyn Fn(&A)>>,
}

impl<A: Asset> Node<A> {
    pub fn new(handle: Handle<A>) -> Self {
        Self { handle, next: None }
    }

    pub fn then(mut self, next: impl Fn(&A) + 'static) -> Self {
        self.next = Some(Box::new(next));
        self
    }

    pub(crate) fn call_then(node: In<Self>, assets: &Res<Assets<A>>) {
        if let Some(next) = node.0.next {
            let loaded_asset = assets.get(node.0.handle.id()).unwrap();
            next(loaded_asset);
        }
    }
}
