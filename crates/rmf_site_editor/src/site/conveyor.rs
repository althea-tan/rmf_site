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

use crate::site::{Anchor, AnchorParams, Category, Dependents, EdgeLabels, SiteAssets};
use bevy::{ecs::hierarchy::ChildOf, math::Affine3A, prelude::*};
use rmf_site_format::{
    ConveyorHeight, ConveyorMarker, ConveyorThickness, ConveyorWidth, Edge,
    DEFAULT_CONVEYOR_HEIGHT, DEFAULT_CONVEYOR_THICKNESS, DEFAULT_CONVEYOR_WIDTH,
};
use rmf_site_mesh::{make_cuboid, WithOutlineMeshExt};
use rmf_site_picking::Selectable;

pub(crate) const CONVEYOR_BELT_THICKNESS: f32 = 0.001;
const CONVEYOR_BAR_SIZE: f32 = 0.03;
const CONVEYOR_BAR_ELEVATION: f32 = 0.05;
const CONVEYOR_MARGIN: f32 = 0.0145;
const DEFAULT_CONVEYOR_STAND_HEIGHT: f32 =
    DEFAULT_CONVEYOR_HEIGHT - DEFAULT_CONVEYOR_THICKNESS / 2.0;

pub(crate) fn make_conveyor_stand_mesh() -> Mesh {
    let y_offset = (DEFAULT_CONVEYOR_WIDTH - CONVEYOR_BAR_SIZE) / 2.0;
    let leg_z = -DEFAULT_CONVEYOR_STAND_HEIGHT / 2.0;
    let bar_z = CONVEYOR_BAR_ELEVATION + CONVEYOR_BAR_SIZE / 2.0 - DEFAULT_CONVEYOR_STAND_HEIGHT;

    let make_leg = |y: f32| {
        make_cuboid(
            CONVEYOR_BAR_SIZE,
            CONVEYOR_BAR_SIZE,
            DEFAULT_CONVEYOR_STAND_HEIGHT,
        )
        .transform_by(Affine3A::from_translation(Vec3::new(0.0, y, leg_z)))
    };
    let bar = make_cuboid(CONVEYOR_BAR_SIZE, DEFAULT_CONVEYOR_WIDTH, CONVEYOR_BAR_SIZE)
        .transform_by(Affine3A::from_translation(Vec3::new(0.0, 0.0, bar_z)));

    Mesh::from(
        make_leg(-y_offset)
            .merge_with(make_leg(y_offset))
            .merge_with(bar),
    )
    .with_generated_outline_normals()
    .unwrap()
}

#[derive(Component, Debug, Clone, Copy)]
pub struct ConveyorSegments {
    start: Entity,
    start_belt: Entity,
    mid: Entity,
    mid_belt: Entity,
    end: Entity,
    end_belt: Entity,
    start_stand: Entity,
    end_stand: Entity,
}

struct ConveyorVisuals {
    parent_global_tf: GlobalTransform,
    start: Transform,
    start_belt: Transform,
    start_stand: Transform,
    mid: Transform,
    mid_belt: Transform,
    mid_mesh: Mesh,
    mid_belt_mesh: Mesh,
    end: Transform,
    end_belt: Transform,
    end_stand: Transform,
}

fn get_conveyor_anchor_position(
    anchor: Entity,
    label: &str,
    conveyor: Entity,
    anchors: &AnchorParams,
) -> Option<Vec3> {
    match anchors.point_in_parent_frame_of(anchor, Category::Conveyor, conveyor) {
        Ok(p) => Some(p.with_z(0.0)),
        Err(_) => {
            error!("Failed to find {label} anchor {anchor:?} for conveyor {conveyor:?}");
            None
        }
    }
}

fn make_conveyor_visuals(
    entity: Entity,
    edge: &Edge<Entity>,
    height: f32,
    width: f32,
    thickness: f32,
    anchors: &AnchorParams,
    child_of: &Query<&ChildOf>,
    global_transforms: &Query<&GlobalTransform>,
) -> Option<ConveyorVisuals> {
    let start_anchor = get_conveyor_anchor_position(edge.start(), "start", entity, anchors)?;
    let end_anchor = get_conveyor_anchor_position(edge.end(), "end", entity, anchors)?;
    let center = (start_anchor + end_anchor) / 2.0;

    // Sync GlobalTransform to align end caps and resize mid mesh within the same frame.
    let parent_global_tf = child_of
        .get(entity)
        .ok()
        .and_then(|c| global_transforms.get(c.parent()).ok())
        .copied()
        .unwrap_or(GlobalTransform::IDENTITY);

    let width = width.max(0.001);
    let thickness = thickness.max(0.001);
    let belt_width = (width - CONVEYOR_MARGIN).max(0.001);
    let belt_z = (thickness + CONVEYOR_BELT_THICKNESS) / 2.0;
    let flat_rotation = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);

    let start_to_end = end_anchor - start_anchor;
    let length = start_to_end.length();
    let direction = if length > 0.001 {
        start_to_end / length
    } else {
        Vec3::X
    };
    let rot = Quat::from_rotation_z(start_to_end.y.atan2(start_to_end.x));

    let stand_height = (height - thickness / 2.0).max(0.001);
    let stand_scale = Vec3::new(
        1.0,
        width / DEFAULT_CONVEYOR_WIDTH,
        stand_height / DEFAULT_CONVEYOR_STAND_HEIGHT,
    );
    let offset = (width / 2.0 + CONVEYOR_BAR_SIZE / 2.0).min(length / 2.0);

    let end_transforms = |anchor_pos: Vec3, stand_center: Vec3| {
        let end_cap = Transform::from_xyz(anchor_pos.x, anchor_pos.y, height)
            .with_rotation(flat_rotation)
            .with_scale(Vec3::new(width, thickness, width));
        let belt = Transform::from_xyz(anchor_pos.x, anchor_pos.y, height + belt_z)
            .with_rotation(flat_rotation)
            .with_scale(Vec3::new(belt_width, 1.0, belt_width));
        let stand = Transform::from_xyz(stand_center.x, stand_center.y, stand_height)
            .with_rotation(rot)
            .with_scale(stand_scale);
        (end_cap, belt, stand)
    };

    let (start, start_belt, start_stand) =
        end_transforms(start_anchor, start_anchor + direction * offset);
    let (end, end_belt, end_stand) = end_transforms(end_anchor, end_anchor - direction * offset);

    let mid = Transform::from_xyz(center.x, center.y, height).with_rotation(rot);
    let mid_belt = Transform::from_xyz(center.x, center.y, height + belt_z).with_rotation(rot);

    // Bake dimensions into the cuboid mesh to ensure uniform outline normals are generated.
    let mesh_length = length.max(0.001);
    let make_mid_mesh = |width: f32, thickness: f32| {
        Mesh::from(Cuboid::new(mesh_length, width, thickness))
            .with_generated_outline_normals()
            .inspect_err(|err| error!("Failed to generate conveyor outline normals: {err}"))
            .ok()
    };
    let mid_mesh = make_mid_mesh(width, thickness)?;
    let mid_belt_mesh = make_mid_mesh(belt_width, CONVEYOR_BELT_THICKNESS)?;

    Some(ConveyorVisuals {
        parent_global_tf,
        start,
        start_belt,
        start_stand,
        mid,
        mid_belt,
        mid_mesh,
        mid_belt_mesh,
        end,
        end_belt,
        end_stand,
    })
}

pub fn add_conveyor_visuals(
    mut commands: Commands,
    new_conveyors: Query<
        (
            Entity,
            &Edge<Entity>,
            &ConveyorHeight,
            &ConveyorWidth,
            &ConveyorThickness,
        ),
        Added<ConveyorMarker>,
    >,
    mut dependents: Query<&mut Dependents, With<Anchor>>,
    child_of: Query<&ChildOf>,
    global_transforms: Query<&GlobalTransform>,
    anchors: AnchorParams,
    assets: Res<SiteAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if new_conveyors.is_empty() {
        return;
    }

    for (e, edge, height, width, thickness) in &new_conveyors {
        for anchor in &edge.array() {
            if let Ok(mut deps) = dependents.get_mut(*anchor) {
                deps.insert(e);
            }
        }

        let Some(visuals) = make_conveyor_visuals(
            e,
            edge,
            height.0,
            width.0,
            thickness.0,
            &anchors,
            &child_of,
            &global_transforms,
        ) else {
            continue;
        };

        let mut spawn_segment = |tf, mesh, mat| {
            commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(mat),
                    tf,
                    visuals.parent_global_tf.mul_transform(tf),
                    Visibility::Inherited,
                    ChildOf(e),
                    Selectable::new(e),
                ))
                .id()
        };

        let mid = spawn_segment(
            visuals.mid,
            meshes.add(visuals.mid_mesh),
            assets.conveyor_metal_material.clone(),
        );
        let mid_belt = spawn_segment(
            visuals.mid_belt,
            meshes.add(visuals.mid_belt_mesh),
            assets.conveyor_belt_material.clone(),
        );

        let mut spawn_end = |end_cap_tf, belt_tf, stand_tf| {
            (
                spawn_segment(
                    end_cap_tf,
                    assets.conveyor_end_mesh.clone(),
                    assets.conveyor_metal_material.clone(),
                ),
                spawn_segment(
                    belt_tf,
                    assets.conveyor_belt_end_mesh.clone(),
                    assets.conveyor_belt_material.clone(),
                ),
                spawn_segment(
                    stand_tf,
                    assets.conveyor_stand_mesh.clone(),
                    assets.conveyor_metal_material.clone(),
                ),
            )
        };
        let (start, start_belt, start_stand) =
            spawn_end(visuals.start, visuals.start_belt, visuals.start_stand);
        let (end, end_belt, end_stand) =
            spawn_end(visuals.end, visuals.end_belt, visuals.end_stand);

        commands.entity(e).insert((
            ConveyorSegments {
                start,
                start_belt,
                mid,
                mid_belt,
                end,
                end_belt,
                start_stand,
                end_stand,
            },
            Transform::IDENTITY,
            visuals.parent_global_tf,
            Visibility::Inherited,
            Category::Conveyor,
            EdgeLabels::StartEnd,
        ));
    }
}

fn update_conveyor_visuals(
    entity: Entity,
    edge: &Edge<Entity>,
    segments: &ConveyorSegments,
    height: f32,
    width: f32,
    thickness: f32,
    child_of: &Query<&ChildOf>,
    tf_params: &mut ParamSet<(
        (AnchorParams, Query<&GlobalTransform>),
        Query<(&mut Transform, &mut GlobalTransform), Without<Anchor>>,
    )>,
    mesh_handles: &Query<&Mesh3d>,
    meshes: &mut Assets<Mesh>,
) {
    let Some(visuals) = ({
        let (anchors, global_transforms) = tf_params.p0();
        make_conveyor_visuals(
            entity,
            edge,
            height,
            width,
            thickness,
            &anchors,
            child_of,
            &global_transforms,
        )
    }) else {
        return;
    };

    for (e, new_mesh) in [
        (segments.mid, visuals.mid_mesh),
        (segments.mid_belt, visuals.mid_belt_mesh),
    ] {
        if let Ok(mesh_handle) = mesh_handles.get(e) {
            if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
                *mesh = new_mesh;
            }
        }
    }

    let mut transforms = tf_params.p1();
    if let Ok((mut tf, mut global_tf)) = transforms.get_mut(entity) {
        *tf = Transform::IDENTITY;
        *global_tf = visuals.parent_global_tf;
    }
    for (e, seg_tf) in [
        (segments.start, visuals.start),
        (segments.start_belt, visuals.start_belt),
        (segments.start_stand, visuals.start_stand),
        (segments.mid, visuals.mid),
        (segments.mid_belt, visuals.mid_belt),
        (segments.end, visuals.end),
        (segments.end_belt, visuals.end_belt),
        (segments.end_stand, visuals.end_stand),
    ] {
        if let Ok((mut tf, mut global_tf)) = transforms.get_mut(e) {
            *tf = seg_tf;
            *global_tf = visuals.parent_global_tf.mul_transform(seg_tf);
        }
    }
}

pub fn update_conveyor_for_moved_anchor(
    conveyors: Query<
        (
            Entity,
            &Edge<Entity>,
            &ConveyorSegments,
            &ConveyorHeight,
            &ConveyorWidth,
            &ConveyorThickness,
        ),
        With<ConveyorMarker>,
    >,
    changed_anchors: Query<
        &Dependents,
        (
            With<Anchor>,
            Or<(Changed<Anchor>, Changed<GlobalTransform>)>,
        ),
    >,
    child_of: Query<&ChildOf>,
    mut tf_params: ParamSet<(
        (AnchorParams, Query<&GlobalTransform>),
        Query<(&mut Transform, &mut GlobalTransform), Without<Anchor>>,
    )>,
    mesh_handles: Query<&Mesh3d>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for dependents in &changed_anchors {
        for dependent in dependents.iter() {
            if let Ok((e, edge, segments, height, width, thickness)) = conveyors.get(*dependent) {
                update_conveyor_visuals(
                    e,
                    edge,
                    segments,
                    height.0,
                    width.0,
                    thickness.0,
                    &child_of,
                    &mut tf_params,
                    &mesh_handles,
                    &mut meshes,
                );
            }
        }
    }
}

pub fn update_changed_conveyor(
    conveyors: Query<
        (
            Entity,
            &Edge<Entity>,
            &ConveyorSegments,
            &ConveyorHeight,
            &ConveyorWidth,
            &ConveyorThickness,
        ),
        (
            With<ConveyorMarker>,
            Or<(
                Changed<Edge<Entity>>,
                Changed<ConveyorHeight>,
                Changed<ConveyorWidth>,
                Changed<ConveyorThickness>,
            )>,
        ),
    >,
    child_of: Query<&ChildOf>,
    mut tf_params: ParamSet<(
        (AnchorParams, Query<&GlobalTransform>),
        Query<(&mut Transform, &mut GlobalTransform), Without<Anchor>>,
    )>,
    mesh_handles: Query<&Mesh3d>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (e, edge, segments, height, width, thickness) in &conveyors {
        update_conveyor_visuals(
            e,
            edge,
            segments,
            height.0,
            width.0,
            thickness.0,
            &child_of,
            &mut tf_params,
            &mesh_handles,
            &mut meshes,
        );
    }
}
