use bevy::{
	core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
	prelude::*,
};
use bevy_editor_pls::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_xpbd_3d_thrusters::{plugins::ThrusterPlugins, prelude::*};

fn main() {
	let mut app = App::new();

	app.add_plugins((
		DefaultPlugins,
		ThrusterPlugins::new(Update),
		EditorPlugin::default(),
	));

	app.add_systems(Startup, setup2);
	app.add_systems(
		Update,
		(ThrusterVisual::auto_expand, ThrusterVisual::auto_sync),
	);

	app.run();
}

fn setup(
	mut commands: Commands,
	mut mats: ResMut<Assets<StandardMaterial>>,
	mut meshs: ResMut<Assets<Mesh>>,
) {
	commands.spawn(Camera3dBundle {
		transform: Transform::from_xyz(0.0, 0.0, 10.),
		camera: Camera {
			hdr: true,
			..default()
		},
		..default()
	});
	commands.insert_resource(AmbientLight {
		color: Color::WHITE,
		brightness: 60.0,
	});

	commands.spawn((
		Name::new("Thruster"),
		Thruster::new(1.0),
		PbrBundle {
			material: mats.add(Color::GREEN),
			mesh: meshs.add(Cuboid::from_size(Vec3::splat(1.0))),
			transform: Transform::from_xyz(0.0, 0.0, 0.0),
			..default()
		},
		ThrusterVisual::default(),
	));
}

fn setup2(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
	commands.spawn((
		Camera3dBundle {
			transform: Transform::from_translation(Vec3::new(0., 0., 50.)),
			camera: Camera {
				hdr: true,
				clear_color: ClearColorConfig::Custom(Color::BLACK),
				..default()
			},
			tonemapping: Tonemapping::None,
			..default()
		},
		BloomSettings::default(),
	));

	let mut color_gradient1 = Gradient::new();
	color_gradient1.add_key(0.0, Vec4::new(4.0, 4.0, 4.0, 1.0));
	color_gradient1.add_key(0.1, Vec4::new(4.0, 4.0, 0.0, 1.0));
	color_gradient1.add_key(0.9, Vec4::new(4.0, 0.0, 0.0, 1.0));
	color_gradient1.add_key(1.0, Vec4::new(4.0, 0.0, 0.0, 0.0));

	let mut size_gradient1 = Gradient::new();
	size_gradient1.add_key(0.0, Vec2::splat(0.1));
	size_gradient1.add_key(0.3, Vec2::splat(0.1));
	size_gradient1.add_key(1.0, Vec2::splat(0.0));

	let writer = ExprWriter::new();

	// Give a bit of variation by randomizing the age per particle. This will
	// control the starting color and starting size of particles.
	let age = writer.lit(0.).uniform(writer.lit(0.2)).expr();
	let init_age = SetAttributeModifier::new(Attribute::AGE, age);

	// Give a bit of variation by randomizing the lifetime per particle
	let lifetime = writer.lit(0.8).uniform(writer.lit(1.2)).expr();
	let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

	// Add constant downward acceleration to simulate gravity
	let accel = writer.lit(Vec3::Y * -8.).expr();
	let update_accel = AccelModifier::new(accel);

	// Add drag to make particles slow down a bit after the initial explosion
	let drag = writer.lit(5.).expr();
	let update_drag = LinearDragModifier::new(drag);

	let init_pos = SetPositionSphereModifier {
		center: writer.lit(Vec3::ZERO).expr(),
		radius: writer.lit(2.).expr(),
		dimension: ShapeDimension::Volume,
	};

	// Give a bit of variation by randomizing the initial speed
	let init_vel = SetVelocitySphereModifier {
		center: writer.lit(Vec3::ZERO).expr(),
		speed: (writer.rand(ScalarType::Float) * writer.lit(20.) + writer.lit(60.)).expr(),
	};

	let effect = EffectAsset::new(
		32768,
		Spawner::burst(2500.0.into(), 2.0.into()),
		writer.finish(),
	)
	.with_name("firework")
	.init(init_pos)
	.init(init_vel)
	.init(init_age)
	.init(init_lifetime)
	.update(update_drag)
	.update(update_accel)
	.render(ColorOverLifetimeModifier {
		gradient: color_gradient1,
	})
	.render(SizeOverLifetimeModifier {
		gradient: size_gradient1,
		screen_space_size: false,
	});

	let effect1 = effects.add(effect);

	commands.spawn((
		Name::new("firework"),
		ParticleEffectBundle {
			effect: ParticleEffect::new(effect1),
			transform: Transform::IDENTITY,
			..Default::default()
		},
	));
}
