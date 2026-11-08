/*
 * Copyright (C) 2024 Open Source Robotics Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
*/

use crate::*;
#[cfg(feature = "bevy")]
use bevy::prelude::{Bundle, Component, Deref, DerefMut};
use serde::{Deserialize, Serialize};

pub const DEFAULT_CONVEYOR_HEIGHT: f32 = 0.5;
pub const DEFAULT_CONVEYOR_WIDTH: f32 = 0.319;
pub const DEFAULT_CONVEYOR_THICKNESS: f32 = 0.05;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "bevy", derive(Bundle))]
pub struct Conveyor<T: RefTrait> {
    pub anchors: Edge<T>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub height: ConveyorHeight,
    #[serde(default, skip_serializing_if = "is_default")]
    pub width: ConveyorWidth,
    #[serde(default, skip_serializing_if = "is_default")]
    pub thickness: ConveyorThickness,
    #[serde(skip)]
    pub marker: ConveyorMarker,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "bevy", derive(Component, Deref, DerefMut))]
pub struct ConveyorHeight(pub f32);

impl Default for ConveyorHeight {
    fn default() -> Self {
        Self(DEFAULT_CONVEYOR_HEIGHT)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "bevy", derive(Component, Deref, DerefMut))]
pub struct ConveyorWidth(pub f32);

impl Default for ConveyorWidth {
    fn default() -> Self {
        Self(DEFAULT_CONVEYOR_WIDTH)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "bevy", derive(Component, Deref, DerefMut))]
pub struct ConveyorThickness(pub f32);

impl Default for ConveyorThickness {
    fn default() -> Self {
        Self(DEFAULT_CONVEYOR_THICKNESS)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "bevy", derive(Component))]
pub struct ConveyorMarker;

impl<T: RefTrait> From<Edge<T>> for Conveyor<T> {
    fn from(anchors: Edge<T>) -> Self {
        Self {
            anchors,
            height: ConveyorHeight::default(),
            width: ConveyorWidth::default(),
            thickness: ConveyorThickness::default(),
            marker: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConveyorPortDirection {
    #[default]
    In,
    Out,
}

impl ConveyorPortDirection {
    pub fn label(&self) -> &'static str {
        match self {
            Self::In => "in",
            Self::Out => "out",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "bevy", derive(Component))]
pub struct ConveyorPort {
    pub direction: ConveyorPortDirection,
    pub location: [f32; 3],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<f32>,
}
