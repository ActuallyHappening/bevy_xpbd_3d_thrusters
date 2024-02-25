use bevy::prelude::*;
use bevy_editor_pls::prelude::*;
use bevy_xpbd_3d_thrusters::{plugins::ThrusterPlugins, prelude::*};

fn main() {
	let mut app = App::new();

	app.add_plugins((
		DefaultPlugins,
		ThrusterPlugins::new(Update),
		EditorPlugin::default(),
	));

	app.add_systems(Startup, setup);

	app.run();
}

fn setup(
	mut commands: Commands,
	mut mats: ResMut<Assets<StandardMaterial>>,
	mut meshs: ResMut<Assets<Mesh>>,
) {
	commands.spawn(Camera3dBundle {
		transform: Transform::from_xyz(0.0, 0.0, 10.),
		..default()
	});
	commands.insert_resource(AmbientLight {
		color: Color::WHITE,
		brightness: 60.0,
	});

	commands.spawn((
		Thruster { strength: 1.0 },
		PbrBundle {
			material: mats.add(Color::GREEN),
			mesh: meshs.add(Cuboid::from_size(Vec3::splat(1.0))),
			transform: Transform::from_xyz(0.0, 0.0, 0.0),
			..default()
		},
	));
}
