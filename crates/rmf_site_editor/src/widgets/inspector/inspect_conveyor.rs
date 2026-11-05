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
    site::{Change, ConveyorHeight, ConveyorThickness, ConveyorWidth},
    widgets::{inspector::InspectValue, prelude::*, Inspect},
};
use bevy::prelude::*;
use bevy_egui::egui::Ui;
use rmf_site_egui::WidgetSystem;
use rmf_site_format::ConveyorMarker;

#[derive(SystemParam)]
pub struct InspectConveyor<'w, 's> {
    commands: Commands<'w, 's>,
    conveyors: Query<
        'w,
        's,
        (
            &'static ConveyorHeight,
            &'static ConveyorWidth,
            &'static ConveyorThickness,
        ),
        With<ConveyorMarker>,
    >,
}

impl<'w, 's> WidgetSystem<Inspect> for InspectConveyor<'w, 's> {
    fn show(
        Inspect { selection, .. }: Inspect,
        ui: &mut Ui,
        state: &mut SystemState<Self>,
        world: &mut World,
    ) {
        let mut params = state.get_mut(world);
        let Ok((height, width, thickness)) = params.conveyors.get(selection) else {
            return;
        };

        if let Some(new_height) = InspectValue::<f32>::new("Height", height.0)
            .clamp_range(0.001..=std::f32::INFINITY)
            .speed(0.01)
            .show(ui)
        {
            params
                .commands
                .trigger(Change::new(ConveyorHeight(new_height), selection));
        }
        if let Some(new_width) = InspectValue::<f32>::new("Width", width.0)
            .clamp_range(0.001..=std::f32::INFINITY)
            .speed(0.01)
            .show(ui)
        {
            params
                .commands
                .trigger(Change::new(ConveyorWidth(new_width), selection));
        }
        if let Some(new_thickness) = InspectValue::<f32>::new("Thickness", thickness.0)
            .clamp_range(0.001..=std::f32::INFINITY)
            .speed(0.01)
            .show(ui)
        {
            params
                .commands
                .trigger(Change::new(ConveyorThickness(new_thickness), selection));
        }
        ui.add_space(10.0);
    }
}
