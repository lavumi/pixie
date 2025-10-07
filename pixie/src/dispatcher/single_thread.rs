use super::UnifiedDispatcher;
use specs::prelude::*;

#[macro_export]
macro_rules! construct_dispatcher {
    (
        $(
            (
                $type:ident,
                $name:expr,
                $deps:expr
            )
        ),*
    ) => {
        pub fn new_dispatch() -> Box<dyn $crate::dispatcher::UnifiedDispatcher + 'static> {
            let mut dispatch = $crate::dispatcher::SingleThreadedDispatcher{
                systems : Vec::new()
            };

            $(
                dispatch.systems.push( Box::new( $type {} ));
            )*

            return Box::new(dispatch);
        }
    };
}

pub struct SingleThreadedDispatcher<'a> {
    pub systems : Vec<Box<dyn RunNow<'a>>>
}

impl<'a> UnifiedDispatcher for SingleThreadedDispatcher<'a> {
    fn run_now(&mut self, ecs : *mut World) {
        unsafe {
            for sys in self.systems.iter_mut() {
                sys.run_now(&*ecs);
            }
            // crate::effects::run_effects_queue(&mut *ecs);
        }
    }
}
