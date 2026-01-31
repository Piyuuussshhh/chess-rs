use bevy::prelude::*;

#[derive(Event)]
pub struct MoveMadeEvent {
    pub piece: Entity,
    pub start: (u8, u8),
    pub end: (u8, u8),
}