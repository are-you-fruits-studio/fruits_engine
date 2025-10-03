use std::{any::Any, collections::{HashMap, HashSet}, sync::{Arc, Mutex}};

use fruits_ffi::FfiString;
use fruits_utils::thread_pool::ThreadPool;

use crate::*;

pub struct SystemsHolder {
    systems: Vec<SystemFfi>,
    system_datas: Arc<[Mutex<SystemResourcesUnsafeHolder>]>,
    execution_graph: Arc<OrderGraph>,
    thread_pool: ThreadPool,
    types: TypesRegistryCache,
}

impl SystemsHolder {
    pub fn new(systems: Vec<SystemFfi>, execution_graph: Arc<OrderGraph>, types: TypesRegistryCache) -> Self {
        Self {
            system_datas: systems.iter().map(|_| Mutex::new(SystemResourcesUnsafeHolder::new(types.clone()))).collect::<Arc<_>>(),
            systems,
            execution_graph,
            thread_pool: ThreadPool::new(Self::non_main_threads_count()),
            types,
        }
    }

    fn non_main_threads_count() -> usize {
        match std::thread::available_parallelism() {
            Ok(count) => (count.get() - 1).max(1),
            Err(_) => 3,
        }
    }

    pub fn execute_iteration(&self, data: WorldDataMut) {
        // Safety. No reference outlives this function.
        let data = unsafe { WorldDataUnsafeRef::from_safe(data) };

        let iter = Arc::new(Mutex::new(self.execution_graph.iter()));

        self.thread_pool.scope(|scope| {
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

                    let data = data.clone();
                
                    let job = move || {
                        let system = &systems[system_index];
                        let system_data = &system_datas[system_index];

                        {
                            unsafe {
                                let system_ctx = SystemCtxFfi {
                                    world_mut: data.ffi(),
                                    system_data: system_data.try_lock().ok().unwrap().ffi(),
                                };
                            
                                // Safety. Access is managed by OrderGraph and data usage.
                                system.execute(system_ctx);
                            }
                        }
                        
                        {
                            iter.lock().unwrap().end(system_index);
                        }
                    };
                
                    let job: Box<dyn FnOnce() + Send> = Box::new(job);
                
                    scope.push_job_unhandled(job);
                } else {
                    scope.panic_if_err();
                }
            }
        });
    }
}

pub struct ScheduleBehaviorBuilder {
    systems: HashMap<FfiString, SystemFfi>,
    systems_ordering: HashSet<(OrderEntry, OrderEntry)>,
    system_groups: HashMap<FfiString, HashSet<OrderEntry>>,
    types: TypesRegistryCache,
}

impl ScheduleBehaviorBuilder {
    pub fn new(types: TypesRegistryCache) -> Self {
        Self {
            systems: HashMap::new(),
            systems_ordering: HashSet::new(),
            system_groups: HashMap::new(),
            types,
        }
    }

    pub fn insert_system<M: 'static>(&mut self, system: impl SystemWithMarker<M> + Any) -> bool {
        self.systems.insert(
            FfiString::from_string(std::any::type_name_of_val(&system).to_string()),
            SystemFfi::new(system, self.types.clone()),
        ).is_none()
    }

    #[must_use]
    pub fn order_system<'a, M0: 'static>(&'a mut self, s: impl SystemWithMarker<M0> + Any) -> OrderHelper<'a> {
        OrderHelper::from_system(self, s)
    }

    #[must_use]
    pub fn order_group<'a>(&'a mut self, g: &'static str) -> OrderHelper<'a> {
        OrderHelper::from_group(self, g)
    }

    #[must_use]
    pub fn group<'a>(&'a mut self, group: &'static str) -> GroupHelper<'a> {
        GroupHelper::new(self, group)
    }

    pub fn build(self) -> SystemsHolder {
        let systems_ordering = flatten_ordering(&self.systems_ordering, &self.system_groups);

        let systems = sort_systems_by_order(self.systems, &systems_ordering);

        let execution_graph = create_ordering_graph(&systems, &systems_ordering);

        SystemsHolder::new(systems, Arc::new(execution_graph), self.types)
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
            entry: OrderEntry::System(FfiString::from_string(std::any::type_name_of_val(&previous_system).to_string())),
        }
    }
    fn from_group(builder: &'a mut ScheduleBehaviorBuilder, group: &'static str) -> Self {
        Self {
            builder,
            entry: OrderEntry::Group(FfiString::from_string(group.to_string())),
        }
    }

    pub fn before_system<M1: 'static>(self, s: impl SystemWithMarker<M1> + Any) {
        self.builder.systems_ordering.insert((self.entry, OrderEntry::System(FfiString::from_string(std::any::type_name_of_val(&s).to_string()))));
    }

    pub fn before_group(self, g: &'static str) {
        self.builder.systems_ordering.insert((self.entry, OrderEntry::Group(FfiString::from_string(g.to_string()))));
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

    pub fn insert_child_system<M1: 'static>(&mut self, s: impl SystemWithMarker<M1> + Any) -> &mut Self {
        self.builder.system_groups.entry(FfiString::from_string(self.group.to_string())).or_default().insert(OrderEntry::System(FfiString::from_string(std::any::type_name_of_val(&s).to_string())));
        self.builder.insert_system(s);
        self
    }

    pub fn insert_child_group(&mut self, g: &'static str) -> &mut Self {
        self.builder.system_groups.entry(FfiString::from_string(self.group.to_string())).or_default().insert(OrderEntry::Group(FfiString::from_string(g.to_string())));
        self
    }
}