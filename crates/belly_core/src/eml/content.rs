use crate::{
    element::{Element, TextElementBundle},
    eml::Eml,
    relations::{
        bind::{BindableSource, BindableTarget, FromComponent, FromResourceWithTransformer},
        RelationsSystems,
    },
    to,
};
use bevy::prelude::*;
use std::any::TypeId;

pub trait IntoContent {
    fn into_content(self, parent: Entity, world: &mut World) -> Vec<Entity>;
}

impl IntoContent for String {
    fn into_content(self, parent: Entity, world: &mut World) -> Vec<Entity> {
        let mut entity = world.entity_mut(parent);
        if let Some(mut text) = entity.get_mut::<Text>() {
            text.sections[0].value = self;
        } else {
            let text = Text::from_section(self, Default::default());
            entity
                .insert(TextElementBundle {
                    text: TextBundle { text, ..default() },
                    ..default()
                });
        }
        vec![]
    }
}

impl IntoContent for &str {
    fn into_content(self, parent: Entity, world: &mut World) -> Vec<Entity> {
        self.to_string().into_content(parent, world)
    }
}

#[derive(Component)]
pub struct BindContent<S: BindableSource + IntoContent + std::fmt::Debug> {
    value: S,
}
impl<R: Component, S: BindableTarget + Clone + Default + IntoContent + std::fmt::Debug> IntoContent
    for FromComponent<R, S>
{
    fn into_content(self, _parent: Entity, world: &mut World) -> Vec<Entity> {
        let entity = world.spawn_empty().id();
        let bind = self >> to!(entity, BindContent<S>:value);
        bind.write(world);
        world
            .entity_mut(entity)
            .insert(NodeBundle::default())
            .insert(BindContent {
                value: S::default(),
            });
        let systems = world.get_resource_or_insert_with(RelationsSystems::default);
        systems
            .0
            .add_custom_system(TypeId::of::<BindContent<S>>(), bind_content_system::<S>);
        vec![entity]
    }
}

impl<
        R: Resource,
        S: BindableSource,
        T: BindableTarget + Clone + Default + IntoContent + std::fmt::Debug,
    > IntoContent for FromResourceWithTransformer<R, S, T>
{
    fn into_content(self, _parent: Entity, world: &mut World) -> Vec<Entity> {
        let entity = world.spawn_empty().id();
        let bind = self >> to!(entity, BindContent<T>:value);
        bind.write(world);
        world
            .entity_mut(entity)
            .insert(NodeBundle::default())
            .insert(BindContent {
                value: T::default(),
            });
        let systems = world.get_resource_or_insert_with(RelationsSystems::default);
        systems
            .0
            .add_custom_system(TypeId::of::<BindContent<T>>(), bind_content_system::<T>);
        vec![entity]
    }
}

fn bind_content_system<T: BindableTarget + IntoContent + Clone + std::fmt::Debug>(
    mut commands: Commands,
    binds: Query<(Entity, &BindContent<T>), Changed<BindContent<T>>>,
) {
    // info!("bindsystem for {}", type_name::<T>());
    for (entity, bind) in binds.iter() {
        let content = bind.value.clone();
        // info!("bind value changed for {:?}", entity);
        commands.add(move |world: &mut World| {
            // info!("setting value: bind value changed to {:?}", content);
            content.into_content(entity, world);
        })
    }
}

impl IntoContent for Vec<Entity> {
    fn into_content(self, _parent: Entity, _world: &mut World) -> Vec<Entity> {
        self
    }
}

impl<T: Iterator, F: Fn(T::Item) -> Eml> IntoContent for ExpandElements<T, F> {
    fn into_content(self, _parent: Entity, world: &mut World) -> Vec<Entity> {
        self.into_iter()
            .map(|builder| builder.build(world))
            .collect()
    }
}

impl IntoContent for Vec<Eml> {
    fn into_content(self, _parent: Entity, world: &mut World) -> Vec<Entity> {
        self.into_iter()
            .map(|builder| builder.build(world))
            .collect()
    }
}

impl IntoContent for Eml {
    fn into_content(self, _parent: Entity, world: &mut World) -> Vec<Entity> {
        vec![self.build(world)]
    }
}

pub struct ExpandElements<I: Iterator, F: Fn(I::Item) -> Eml> {
    mapper: F,
    previous: I,
}

impl<I, F> Iterator for ExpandElements<I, F>
where
    I: Iterator,
    F: Fn(I::Item) -> Eml,
{
    type Item = Eml;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(x) = self.previous.next() {
            return Some((self.mapper)(x));
        }
        None
    }
}

pub trait ExpandElementsExt: Iterator {
    fn elements<F: Fn(Self::Item) -> Eml>(self, mapper: F) -> ExpandElements<Self, F>
    where
        Self: Sized,
    {
        ExpandElements {
            mapper,
            previous: self,
        }
    }
}

impl<I: Iterator> ExpandElementsExt for I {}
