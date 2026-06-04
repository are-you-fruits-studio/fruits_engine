//! # fruits_ecs
//!
//! The entity-component-system core of the Fruits engine. It stores game state as entities,
//! components, resources, and events, and runs game logic as systems that read and write that
//! state.
//!
//! # How to use
//!
//! Data types opt into the ECS with the re-exported derive macros — `Component` for data attached
//! to entities, `Resource` for world-global state, `SystemResource` for per-system state, and
//! `Event` for transient messages. Systems are plain functions whose parameters declare what they
//! touch; the world infers each system's data usage from those parameters and runs independent
//! systems in parallel.
//!
//! #### Building a world and running its systems
//!
//! Register systems on a [`WorldBuilder`], seed entities and resources, then [`build`](WorldBuilder::build)
//! the [`World`] and drive it one schedule pass at a time with
//! [`execute_iteration`](World::execute_iteration).
//!
//! ```rust,no_run
//! use fruits_ecs::*;
//!
//! #[derive(Component)]
//! struct Position { x: f32, y: f32 }
//!
//! #[derive(Component)]
//! struct Velocity { x: f32, y: f32 }
//!
//! fn integrate(mut q: WorldQuery<(&mut Position, &Velocity)>) {
//!     for (pos, vel) in q.iter_mut() {
//!         pos.x += vel.x;
//!         pos.y += vel.y;
//!     }
//! }
//!
//! let mut builder = WorldBuilder::new();
//!
//! builder
//!     .behavior_mut()
//!     .get_mut(Schedule::Update)
//!     .insert_system(integrate);
//!
//! {
//!     let mut data = builder.data_mut();
//!     let mut entities = data.entities_mut();
//!
//!     let e = entities.create_entity();
//!     entities.add_component(e, Position { x: 0.0, y: 0.0 }).ok().unwrap();
//!     entities.add_component(e, Velocity { x: 1.0, y: 0.0 }).ok().unwrap();
//! }
//!
//! let mut world = builder.build();
//! world.execute_iteration(Schedule::Update);
//! ```
//!
//! #### Querying entities and their components
//!
//! A [`WorldQuery`] parameter iterates every entity that carries the requested components.
//! Reference items (`&C`, `&mut C`) are required; an `Option<&C>` is fetched when present. A second
//! type argument filters archetypes without binding their data — [`WithFilter`], [`WithoutFilter`],
//! and [`OrFilter`].
//!
//! ```rust,no_run
//! use fruits_ecs::*;
//!
//! #[derive(Component)]
//! struct Enemy;
//! #[derive(Component)]
//! struct Health(u32);
//! #[derive(Component)]
//! struct Frozen;
//!
//! // Sum the health of every enemy that is not frozen.
//! fn total_active_enemy_health(q: WorldQuery<&Health, (WithFilter<Enemy>, WithoutFilter<Frozen>)>) {
//!     let total: u32 = q.iter().map(|h| h.0).sum();
//!     let _ = total;
//! }
//! ```
//!
//! #### Sharing state through resources
//!
//! A [`Resource`](trait@Resource) is a single world-global value. Read it with [`Res`] and mutate it
//! with [`ResMut`]; wrap either in `Option` to tolerate a missing resource. Insert the initial value
//! through the world data.
//!
//! ```rust,no_run
//! use fruits_ecs::*;
//!
//! #[derive(Resource, Default)]
//! struct Score(u32);
//!
//! fn add_point(mut score: ResMut<Score>) {
//!     score.0 += 1;
//! }
//!
//! fn announce(score: Res<Score>) {
//!     println!("score is {}", score.0);
//! }
//!
//! let mut builder = WorldBuilder::new();
//! builder.data_mut().resources_mut().insert(Score(0)).ok().unwrap();
//! builder.behavior_mut().get_mut(Schedule::Update).insert_system(add_point);
//! builder.behavior_mut().get_mut(Schedule::Update).insert_system(announce);
//! ```
//!
//! #### Reading and writing events
//!
//! An [`Event`](trait@Event) is a transient message. Append events with [`EvtMut`] and read the
//! current frame's events with [`Evt`]. The event buffers are cleared between schedule passes by the
//! engine driving the world.
//!
//! ```rust,no_run
//! use fruits_ecs::*;
//!
//! #[derive(Event)]
//! struct Damage { amount: u32 }
//!
//! fn deal_damage(mut writer: EvtMut<Damage>) {
//!     writer.push(Damage { amount: 10 });
//! }
//!
//! fn apply_damage(reader: Evt<Damage>) {
//!     for damage in reader.iter() {
//!         let _ = damage.amount;
//!     }
//! }
//! ```
//!
//! #### Keeping per-system state with `Local`
//!
//! A [`SystemResource`](trait@SystemResource) is private to one system and created from `Default` the
//! first time the system runs. Access it with [`Local`].
//!
//! ```rust,no_run
//! use fruits_ecs::*;
//!
//! #[derive(SystemResource, Default)]
//! struct Ticks(u64);
//!
//! fn count_ticks(mut ticks: Local<Ticks>) {
//!     ticks.0 += 1;
//! }
//! ```
//!
//! #### Making structural changes with `ExclusiveWorldAccess`
//!
//! Creating or destroying entities and adding or removing components is a structural change that
//! cannot run alongside other systems. A system that takes [`ExclusiveWorldAccess`] receives the
//! whole world and runs alone, with no other system in the same pass executing concurrently.
//!
//! ```rust,no_run
//! use fruits_ecs::*;
//!
//! #[derive(Component)]
//! struct Spawned;
//!
//! fn spawn_one(mut world: ExclusiveWorldAccess) {
//!     let mut entities = world.entities_mut();
//!
//!     let e = entities.create_entity();
//!     entities.add_component(e, Spawned).ok().unwrap();
//! }
//! ```
//!
//! #### Ordering systems and grouping them
//!
//! By default systems in a pass run in any order that respects their data dependencies. Pin a
//! relative order explicitly with [`order_system`](SystemsHolderBuilderMut::order_system) /
//! `before_system`, and bundle several systems under a named handle with
//! [`group`](SystemsHolderBuilderMut::group) so they can be ordered together.
//!
//! ```rust,no_run
//! use fruits_ecs::*;
//!
//! fn read_input() {}
//! fn step_physics() {}
//! fn draw() {}
//!
//! let mut builder = WorldBuilder::new();
//! let mut behavior = builder.behavior_mut();
//! let mut update = behavior.get_mut(Schedule::Update);
//!
//! update.insert_system(read_input);
//! update.insert_system(step_physics);
//! update.insert_system(draw);
//!
//! // read_input runs before step_physics, which runs before draw.
//! let _ = update
//!     .order_system(read_input)
//!     .before_system(step_physics)
//!     .before_system(draw);
//! ```
//!
//! # How to maintain
//!
//! #### Two layers: FFI core and safe wrappers
//!
//! Every container exists in two forms. A `#[repr(C)]` `*UnsafeFfi` type owns the raw storage and
//! exposes only pointer-based, `unsafe` operations; a borrow-checked wrapper (`*Mut` / `*Ref`) pairs
//! that storage with a [`TypesRegistryCache`] and presents the safe, generic API the rest of the
//! crate uses. [`WorldDataUnsafeFfi`] bundles the three stores — resources, events, and entities —
//! and [`WorldUnsafeFfi`] adds the per-schedule behavior. The stable `#[repr(C)]` layout is what lets
//! these structures cross the FFI boundary, which is why type identity is carried as a runtime `u64`
//! rather than a Rust generic.
//!
//! #### Runtime type identity
//!
//! Types are not identified by [`std::any::TypeId`] across the boundary. Instead the shared
//! [`TypesRegistryAccessFfi`] assigns each registered type a `u64` id derived from its
//! [`std::any::type_name`], stored alongside its size, alignment, and drop function in
//! [`TypeData`]. [`TypesRegistryCache`] memoizes the `TypeId -> u64` mapping per process so the
//! lookup is paid once per type. Components, resources, events, and system resources are all
//! registered lazily through `get_or_register` the first time they are used. Untyped storage
//! (`ResourcesHolderNative`, `EventsHolderNative`, archetype memory) allocates and drops values
//! purely from this `TypeData`.
//!
//! #### Archetype storage
//!
//! Entities are grouped into archetypes — one contiguous store per distinct set of component types —
//! so a query scans only the archetypes that match. Adding or removing a component moves an entity to
//! a different archetype. Within an archetype, removal is a swap-remove: the last entity is copied
//! into the freed slot, so the archetype-level `destroy_entity` / `add_component` / `remove_component`
//! operations return the entity that was moved, and [`EntitiesHolderUnsafeFfi`] uses that returned
//! entity to fix up its location metadata. Component access goes through cached per-type ids
//! ([`RegistrySpecificTypeId`]) so iteration avoids repeated registry lookups.
//!
//! #### Data usage drives parallel scheduling
//!
//! Each [`SystemParam`] reports what it touches into a [`DataUsageBuilder`]: a per-type read or write
//! ([`Res`]/[`ResMut`], [`Evt`]/[`EvtMut`], the components in a [`WorldQuery`]), a private system
//! resource ([`Local`]), or "everything, mutably" ([`ExclusiveWorldAccess`], via
//! `add_all_mutable_to_world`). The resulting [`DataUsage`] is analyzed when a schedule is built:
//! `create_ordering_graph` adds an edge between two systems whenever their accesses conflict
//! (write-after-read, read-after-write, write-after-write, or any system against a global-mutable
//! one), and any explicit ordering is layered on top. At run time `SystemsHolderNative` walks the
//! resulting [`OrderGraph`] on a [`fruits_utils::thread_pool::ThreadPool`], starting each system once
//! its predecessors finish, so non-conflicting systems execute concurrently. A conflicting or invalid
//! usage (for example two mutable claims on the same type) panics while the system is being
//! registered.
//!
//! #### System parameters and their safety contract
//!
//! A free function becomes a system through the blanket `SystemWithMarker` implementation, which is
//! generated for functions of up to fifteen [`SystemParam`] arguments. Both [`SystemParam`] and
//! [`SystemWithMarker`] are `unsafe` traits: their implementations are produced automatically and are
//! sound only because the scheduler honors the declared [`DataUsage`]. `SystemParam::new` hands out
//! references into the live world from raw pointers and performs no access synchronization itself, so
//! constructing a parameter outside the scheduler — where the order graph has not guaranteed
//! exclusive or shared access — is undefined behavior. If a required resource or event is absent the
//! system panics rather than silently skipping work.
//!
//! #### Schedules
//!
//! [`Schedule`] is a fixed set of passes ([`Start`](Schedule::Start), [`Update`](Schedule::Update)).
//! The world holds one independent system set per schedule, and the driving application chooses when
//! each pass runs — typically `Start` once and `Update` every frame.

mod behavior;
mod data;
mod data_usage;
mod init_ctx;
mod types_registry;
mod world;

pub use behavior::*;
pub use data::*;
pub use data_usage::*;
pub use init_ctx::*;
pub use types_registry::*;
pub use world::*;

pub use fruits_ecs_macros::*;

// todo: fix mods use in the whole crate
