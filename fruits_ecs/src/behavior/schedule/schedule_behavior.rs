use std::{any::{Any, TypeId}, collections::{HashMap, HashSet}, sync::{Arc, Mutex}};

use fruits_utils::thread_pool::ThreadPool;

use crate::*;

pub struct ScheduleBehavior {
    systems: Vec<Box<dyn System>>,
    system_datas: Arc<[Mutex<SystemResourcesHolder>]>,
    execution_graph: Arc<OrderGraph>,
    thread_pool: ThreadPool,
}

impl ScheduleBehavior {
    pub fn new(systems: Vec<Box<dyn System>>, execution_graph: Arc<OrderGraph>) -> Self {
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
                let systems = &self.systems;
                let system_datas = Arc::clone(&self.system_datas);

                let job = move || {
                    let system = &systems[system_index];
                    let system_data = &system_datas[system_index];

                    {
                        let input = SystemInput {
                            world_data: data,
                            system_data: &mut *system_data.try_lock().ok().unwrap(),
                        };
                    
                        
                        // Safety. Access is managed by OrderGraph and data usage.
                        unsafe {
                            system.execute(&input);
                        }
                    }
                    
                    {
                        iter.lock().unwrap().end(system_index);
                    }
                };

                let job: Box<dyn FnOnce() + Send> = Box::new(job);

                // Safety. Iteration blocks until all jobs end, so lifetimes are managed - no need for borrow-checker.
                let job = unsafe {
                    std::mem::transmute::<Box<dyn FnOnce() + Send>, Box<dyn FnOnce() + Send + 'static>>(job)
                };

                self.thread_pool.push_job(job);
            } else {
                self.thread_pool.panic_if_err();
            }
        }
    }
}

pub struct ScheduleBehaviorBuilder {
    systems: HashMap<TypeId, Box<dyn System>>,
    systems_ordering: HashSet<(OrderEntry, OrderEntry)>,
    system_groups: HashMap<&'static str, HashSet<OrderEntry>>,
}

impl ScheduleBehaviorBuilder {
    pub fn new() -> Self {
        Self {
            systems: HashMap::new(),
            systems_ordering: HashSet::new(),
            system_groups: HashMap::new(),
        }
    }

    pub fn add_system<M: 'static>(&mut self, system: impl SystemWithMarker<M> + Any) -> bool {
        self.systems.insert(system.type_id(), system.into_system_generic()).is_none()
    }

    #[must_use]
    pub fn order_system<M0: 'static>(&mut self, s: impl SystemWithMarker<M0> + Any) -> OrderHelper {
        OrderHelper::from_system(self, s)
    }

    #[must_use]
    pub fn order_group(&mut self, g: &'static str) -> OrderHelper {
        OrderHelper::from_group(self, g)
    }

    #[must_use]
    pub fn group(&mut self, group: &'static str) -> GroupHelper {
        GroupHelper::new(self, group)
    }

    pub fn build(self) -> ScheduleBehavior {
        let systems_ordering = flatten_ordering(&self.systems_ordering, &self.system_groups);

        let systems = sort_systems_by_order(self.systems, &systems_ordering);

        let execution_graph = create_ordering_graph(&systems, &systems_ordering);

        let systems = systems.into_iter().map(|s| s.system).collect::<Vec<_>>();

        ScheduleBehavior::new(systems, Arc::new(execution_graph))
    }
}

pub struct OrderHelper<'a> {
    builder: &'a mut ScheduleBehaviorBuilder,
    entry: OrderEntry,
}

impl<'a> OrderHelper<'a> {
    fn from_system<M0: 'static>(builder: &'a mut ScheduleBehaviorBuilder, previous_system: impl SystemWithMarker<M0> + Any) -> Self {
        Self {
            builder,
            entry: OrderEntry::System(previous_system.type_id()),
        }
    }
    fn from_group(builder: &'a mut ScheduleBehaviorBuilder, group: &'static str) -> Self {
        Self {
            builder,
            entry: OrderEntry::Group(group),
        }
    }

    pub fn before_system<M1: 'static>(self, s: impl SystemWithMarker<M1> + Any) {
        self.builder.systems_ordering.insert((self.entry, OrderEntry::System(s.type_id())));
    }

    pub fn before_group(self, g: &'static str) {
        self.builder.systems_ordering.insert((self.entry, OrderEntry::Group(g)));
    }
}

pub struct GroupHelper<'a> {
    builder: &'a mut ScheduleBehaviorBuilder,
    group: &'static str,
}

impl<'a> GroupHelper<'a> {
    fn new(builder: &'a mut ScheduleBehaviorBuilder, group: &'static str) -> Self {
        Self {
            builder,
            group,
        }
    }

    pub fn add_child_system<M1: 'static>(&mut self, s: impl SystemWithMarker<M1> + Any) -> &mut Self {
        self.builder.system_groups.entry(self.group).or_default().insert(OrderEntry::System(s.type_id()));
        self.builder.add_system(s);
        self
    }

    pub fn add_child_group(&mut self, g: &'static str) -> &mut Self {
        self.builder.system_groups.entry(self.group).or_default().insert(OrderEntry::Group(g));
        self
    }
}