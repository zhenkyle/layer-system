#![deny(missing_docs)]
#![no_std]
/*!
See [README.md]
**/
extern crate alloc;
use alloc::boxed::Box;
use alloc::vec::Vec;

/// A special action for the layer.
pub enum ChangeAction {
    /// No special action to the layer.
    None,
    /// Pass the event to the next layer.
    Pass,
    /// Remove the current layer.
    Remove,
    /// Remove all layers.
    Clear,
}

/// The action, that will be done after handling an event by a layer.
pub struct Change<S, E> {
    /// Add new layers on top of the current layer.
    add: Vec<Box<dyn Layer<S, E>>>,
    /// Special actions.
    action: ChangeAction,
}

impl<S, E> Change<S, E> {
    /// A simple change doing nothing.
    pub fn none() -> Self {
        Self {
            add: Vec::new(),
            action: ChangeAction::None,
        }
    }

    /// A change passing the event to the next layer.
    pub fn pass() -> Self {
        Self {
            add: Vec::new(),
            action: ChangeAction::Pass,
        }
    }

    /// A change just adding new layers.
    pub fn add(add: Vec<Box<dyn Layer<S, E>>>) -> Self {
        Self {
            add,
            action: ChangeAction::None,
        }
    }

    /// A simple change removing the current layer.
    pub fn remove() -> Self {
        Self {
            add: Vec::new(),
            action: ChangeAction::Remove,
        }
    }

    /// A change replacing the current layer with new layers.
    pub fn replace(add: Vec<Box<dyn Layer<S, E>>>) -> Self {
        Self {
            add,
            action: ChangeAction::Remove,
        }
    }

    /// A change removing all layers.
    pub fn close() -> Self {
        Self {
            add: Vec::new(),
            action: ChangeAction::Clear,
        }
    }

    /// A change replacing all layers with a new stack of layers.
    pub fn clear(add: Vec<Box<dyn Layer<S, E>>>) -> Self {
        Self {
            add,
            action: ChangeAction::Clear,
        }
    }
}

/// A trait, every layer has to implement, in order to be used by the layer manager;
pub trait Layer<S, E> {
    /// Executed for all layers from bottom to top. Most useful for rendering.
    fn passive_update(&mut self, _state: &mut S, _event: &E) {}

    /// Executed for top layer and optionally for more layers. Most useful for click events.
    fn update(&mut self, _state: &mut S, _event: &E) -> Change<S, E>;
}

/// The layer manager deals with the layers you create.
pub struct LayerManager<S, E>(Vec<Box<dyn Layer<S, E>>>);

impl<S, E> LayerManager<S, E> {
    /// Create a new layer manager containing specified initial layers.
    pub fn new(layers: Vec<Box<dyn Layer<S, E>>>) -> Self {
        LayerManager::<S, E>(layers)
    }

    /// Checks if the layer manger is still active. When not active, the program should terminate or new layers should be added before calling `update` again.
    pub fn is_active(&self) -> bool {
        !self.0.is_empty()
    }

    /// Everytime the program recieves or generates an event, which should be handled by a layer, this method has to be called.
    pub fn update(&mut self, state: &mut S, event: E) {
        let count = self.0.len();
        let mut i = count;
        while i > 0 {
            i -= 1;
            let layer = &mut self.0[i];
            let Change { add, action } = layer.update(state, &event);
            let add_index = i + 1;
            for (i, added) in add.into_iter().enumerate() {
                self.0.insert(add_index + i, added);
            }
            use ChangeAction::*;
            match action {
                None => (),
                Pass => continue,
                Remove => {
                    self.0.remove(i);
                }
                Clear => self.0.clear(),
            }
            break;
        }

        for layer in self.0.iter_mut() {
            layer.passive_update(state, &event);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    pub enum Event {
        Idle,
        Input,
        Exit,
    }

    pub struct GlobalState;

    pub struct MainLayer;

    impl Layer<GlobalState, Event> for MainLayer {
        fn update(
            &mut self,
            _state: &mut GlobalState,
            event: &Event,
        ) -> Change<GlobalState, Event> {
            match event {
                Event::Input => Change::add(vec![Box::new(TopLayer)]),
                Event::Idle => Change::none(),
                Event::Exit => Change::remove(),
            }
        }
    }

    pub struct TopLayer;

    impl Layer<GlobalState, Event> for TopLayer {
        fn update(
            &mut self,
            _state: &mut GlobalState,
            event: &Event,
        ) -> Change<GlobalState, Event> {
            match event {
                Event::Input => Change::pass(),
                Event::Idle => Change::none(),
                Event::Exit => Change::remove(),
            }
        }
    }

    #[test]
    fn example() {
        let mut manager = LayerManager::new(vec![Box::new(MainLayer), Box::new(TopLayer)]);
        let mut state = GlobalState;

        manager.update(&mut state, Event::Idle);
        manager.update(&mut state, Event::Input);
        manager.update(&mut state, Event::Idle);

        while manager.is_active() {
            manager.update(&mut state, Event::Exit);
        }
    }
}
