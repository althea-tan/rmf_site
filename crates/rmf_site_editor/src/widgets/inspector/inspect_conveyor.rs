/*
 * Copyright (C) 2026 Open Source Robotics Foundation
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

use crate::{
    site::{Anchor, Change, ConveyorHeight, ConveyorThickness, ConveyorWidth, Dependents},
    widgets::{inspector::InspectValue, prelude::*, Inspect},
};
use bevy::prelude::*;
use bevy_egui::egui::Ui;
use rmf_site_egui::WidgetSystem;
use rmf_site_format::{ConveyorMarker, Edge};
use std::collections::{BTreeSet, VecDeque};

#[derive(SystemParam)]
pub struct InspectConveyor<'w, 's> {
    commands: Commands<'w, 's>,
    conveyors: Query<
        'w,
        's,
        (
            &'static Edge<Entity>,
            &'static ConveyorHeight,
            &'static ConveyorWidth,
            &'static ConveyorThickness,
        ),
        With<ConveyorMarker>,
    >,
    dependents: Query<'w, 's, &'static Dependents, With<Anchor>>,
}

impl<'w, 's> InspectConveyor<'w, 's> {
    fn inspect_dimension<T: Component + Clone + std::fmt::Debug>(
        &mut self,
        ui: &mut Ui,
        selection: Entity,
        label: &str,
        value: f32,
        wrap: fn(f32) -> T,
    ) {
        let Some(new_value) = InspectValue::<f32>::new(label, value)
            .clamp_range(0.001..=std::f32::INFINITY)
            .speed(0.01)
            .show(ui)
        else {
            return;
        };

        let mut visited = BTreeSet::from([selection]);
        let mut queue = VecDeque::from([selection]);
        while let Some(current) = queue.pop_front() {
            if let Ok((edge, ..)) = self.conveyors.get(current) {
                for anchor in edge.array() {
                    if let Ok(deps) = self.dependents.get(anchor) {
                        for &dep in deps.iter() {
                            if self.conveyors.contains(dep) && visited.insert(dep) {
                                queue.push_back(dep);
                            }
                        }
                    }
                }
            }
        }

        for conveyor in visited {
            self.commands
                .trigger(Change::new(wrap(new_value), conveyor));
        }
    }
}

impl<'w, 's> WidgetSystem<Inspect> for InspectConveyor<'w, 's> {
    fn show(
        Inspect { selection, .. }: Inspect,
        ui: &mut Ui,
        state: &mut SystemState<Self>,
        world: &mut World,
    ) {
        let mut params = state.get_mut(world);
        let Ok((_, height, width, thickness)) = params.conveyors.get(selection) else {
            return;
        };
        let (height, width, thickness) = (height.0, width.0, thickness.0);

        params.inspect_dimension(ui, selection, "Height", height, ConveyorHeight);
        params.inspect_dimension(ui, selection, "Width", width, ConveyorWidth);
        params.inspect_dimension(ui, selection, "Thickness", thickness, ConveyorThickness);
        ui.add_space(10.0);
    }
}
