use std::{any::{Any, TypeId}, collections::{HashMap, HashSet}, sync::{Arc, Mutex}};

use fruits_utils::thread_pool::ThreadPool;

use crate::*;

pub struct ScheduleBehavior {
    systems: Arc<[Arc<dyn System>]>,
    system_datas: Arc<[Mutex<SystemResourcesHolder>]>,
    execution_graph: Arc<OrderGraph>,
    thread_pool: ThreadPool,
}

impl ScheduleBehavior {
    pub fn new(systems: Arc<[Arc<dyn System>]>, execution_graph: Arc<OrderGraph>) -> Self {
        Self {
            system_datas: systems.iter().map(|_| Mutex::new(SystemResourcesHolder::new())).collect::<Arc<_>>(),
            systems,
            execution_graph,
            thread_pool: ThreadPool::new(Self::non_main_threads_count())
        }
    }

    fn non_main_threads_count() -> usize {
        match std::thread::available_parallelism() {
            Ok(count) => (count.get() - 1).max(1),
            Err(_) => 3,
        }
    }

    pub fn execute_iteration(&self, data: &mut WorldData) {
        // Safety. No reference outlives this function.
        let data = unsafe {  &*&WorldDataUnsafe::from_safe_mut(data) };

        let iter = Arc::new(Mutex::new(self.execution_graph.iter()));

        loop {
            let system_index = {
                let mut iter = iter.lock().unwrap();
                
                if iter.all_ended() {
                    break;
                }

                iter.start_next()
            };

            if let Some(system_index) = system_index {
                let iter = Arc::clone(&iter);
                let systems = Arc::clone(&self.systems);
                let system_datas = Arc::clone(&self.system_datas);

                let job = move || {
                    let system = &systems[system_index];
                    let system_data = &system_datas[system_index];

                    let input = SystemInput {
                        world_data: data,
                        system_data: &mut *system_data.try_lock().ok().unwrap(),
                    };
    
                    
                    // Safety. Access is managed by OrderGraph and data usage.
                    unsafe {
                        system.execute(&input);
                    }
                    
                    {
                        iter.lock().unwrap().end(system_index);
                    }
                };

                let job: Box<dyn FnOnce() + Send> = Box::new(job);

                // Safety. Iteration blocks until all jobs end, so lifetimes are managed - no need for borrow-checker.
                let job = unsafe {
                    std::mem::transmute::<_, Box<dyn FnOnce() + Send + 'static>>(job)
                };

                self.thread_pool.push_job(job);
            } else {
                self.thread_pool.panic_if_err();
            }
        }
    }
}

pub struct ScheduleBehaviorBuilder {
    systems: HashMap<TypeId, Arc<dyn System>>,
    systems_ordering: HashSet<(TypeId, TypeId)>
}

impl ScheduleBehaviorBuilder {
    pub fn new() -> Self {
        Self {
            systems: HashMap::new(),
            systems_ordering: HashSet::new(),
        }
    }

    pub fn add_system<M: 'static>(&mut self, system: impl SystemWithMarker<M> + Any) -> bool {
        self.systems.insert(system.type_id(), Arc::from(system.into_system_generic())).is_none()
    }

    pub fn order_systems<M0: 'static, M1: 'static>(
        &mut self,
        previous_system: impl SystemWithMarker<M0> + Any,
        next_system: impl SystemWithMarker<M1> + Any,
    ) {
        self.systems_ordering.insert((previous_system.type_id(), next_system.type_id()));
    }

    pub fn build(self) -> ScheduleBehavior {
        let systems = sort_systems_by_order(&self.systems, &self.systems_ordering);

        let execution_graph = create_ordering_graph(&systems, &self.systems_ordering);

        let systems = systems.iter().map(|s| Arc::clone(&s.system)).collect::<Arc<_>>();

        ScheduleBehavior::new(systems, Arc::new(execution_graph))
    }
}
