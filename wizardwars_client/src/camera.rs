use bevy::prelude::*;
pub struct CameraTarget;
pub struct FollowCamera {
    pub target: Vec3,
    pub vertical_offset: f32,
    pub distance: f32,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(
            CoreStage::PostUpdate,
            update_camera_target
                .system()
                .chain(camera_follow_system.system()),
        );
    }
}

pub fn camera_follow_system(mut query: Query<(&mut Transform, &FollowCamera)>) {
    let dir = Vec3::new(0.0, 1.5, 1.0).normalize();
    for (mut transform, camera) in query.iter_mut() {
        let offset = camera.distance * dir;
        let position = offset + camera.target;
        let offset = Vec3::Y * camera.vertical_offset;
        transform.translation = position;
        transform.look_at(camera.target + offset, Vec3::Y);
    }
}

pub fn update_camera_target(
    mut cameras: Query<&mut FollowCamera>,
    targets: Query<&Transform, With<CameraTarget>>,
) {
    if let Some((mut camera, target)) = cameras.iter_mut().zip(targets.iter()).next() {
        camera.target = target.translation;
    }
}
