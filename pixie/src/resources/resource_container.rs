use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Resource container for hecs
/// Since hecs doesn't have built-in resource management like specs,
/// we implement a simple type-map based resource storage
pub struct ResourceContainer {
    resources: HashMap<TypeId, Box<dyn Any>>,
}

impl ResourceContainer {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    /// Insert a resource
    pub fn insert<T: 'static>(&mut self, resource: T) {
        self.resources.insert(TypeId::of::<T>(), Box::new(resource));
    }

    /// Get immutable reference to a resource
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|r| r.downcast_ref::<T>())
    }

    /// Get mutable reference to a resource
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.resources
            .get_mut(&TypeId::of::<T>())
            .and_then(|r| r.downcast_mut::<T>())
    }

    /// Check if a resource exists
    pub fn contains<T: 'static>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<T>())
    }

    /// Remove a resource
    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        self.resources
            .remove(&TypeId::of::<T>())
            .and_then(|r| r.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }
}

impl Default for ResourceContainer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestResource {
        value: i32,
    }

    #[test]
    fn test_insert_and_get() {
        let mut container = ResourceContainer::new();
        container.insert(TestResource { value: 42 });

        let resource = container.get::<TestResource>();
        assert!(resource.is_some());
        assert_eq!(resource.unwrap().value, 42);
    }

    #[test]
    fn test_get_mut() {
        let mut container = ResourceContainer::new();
        container.insert(TestResource { value: 42 });

        {
            let resource = container.get_mut::<TestResource>().unwrap();
            resource.value = 100;
        }

        assert_eq!(container.get::<TestResource>().unwrap().value, 100);
    }

    #[test]
    fn test_contains() {
        let mut container = ResourceContainer::new();
        assert!(!container.contains::<TestResource>());

        container.insert(TestResource { value: 42 });
        assert!(container.contains::<TestResource>());
    }

    #[test]
    fn test_remove() {
        let mut container = ResourceContainer::new();
        container.insert(TestResource { value: 42 });

        let removed = container.remove::<TestResource>();
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().value, 42);
        assert!(!container.contains::<TestResource>());
    }
}
