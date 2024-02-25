#![doc = include_str!("../README.md")]
#![allow(clippy::type_complexity)]

pub mod prelude {
	pub(crate) use bevy::prelude::*;
	pub use bevy_inspector_egui::prelude::*;
	pub use bevy_xpbd_3d::prelude::*;
	pub use bevy_xpbd_3d_parenting::prelude::*;

	pub use crate::components::*;
	pub use crate::traits::*;
	pub use crate::visuals::*;
}

pub mod plugins {
	use bevy::{
		app::PluginGroupBuilder,
		ecs::schedule::{InternedScheduleLabel, ScheduleLabel},
	};
	use bevy_hanabi::HanabiPlugin;

	use crate::prelude::*;

	/// All necessary plugins to enable all features of `bevy_xpbd_3d_thrusters`.
	/// These plugins are *optional*, and can certainly be added manually.
	#[derive(Debug, Clone, Copy)]
	pub struct ThrusterPlugins {
		schedule: InternedScheduleLabel,
	}

	impl ThrusterPlugins {
		/// Create a new ThrusterPlugins instance,
		/// with the given schedule label.
		///
		/// This schedule label *must* be the same as the schedule that
		/// `bevy_xpbd_3d` runs on.
		///
		/// See [ParentingPlugin::new]
		pub fn new(schedule: impl ScheduleLabel) -> Self {
			Self {
				schedule: schedule.intern(),
			}
		}
	}

	impl PluginGroup for ThrusterPlugins {
		fn build(self) -> bevy::app::PluginGroupBuilder {
			PluginGroupBuilder::start::<Self>()
				.add(ThrusterPlugin)
				.add(ParentingPlugin::new(self.schedule))
				.add(HanabiPlugin)
		}
	}

	/// Registers a bunch of types.
	/// Most systems are manually registered.
	#[derive(Debug, Clone, Copy)]
	pub struct ThrusterPlugin;

	impl Plugin for ThrusterPlugin {
		fn build(&self, app: &mut App) {
			app.register_type::<Thruster>();
		}
	}
}

pub mod components {
	use serde::{Deserialize, Serialize};

	use crate::prelude::*;

	#[derive(Component, Reflect, Debug, Clone, Serialize, Deserialize, InspectorOptions)]
	#[reflect(Component, InspectorOptions)]
	pub struct Thruster {
		#[inspector(min = 0.0)]
		pub strength: f32,
	}
}

pub mod traits {
	use crate::components::Thruster;

	pub trait UpdateFromThruster {
		fn update_with_parent_thruster(&mut self, parent_thruster: &Thruster);
	}
}

pub mod visuals {
	use bevy_hanabi::prelude::*;

	use crate::prelude::*;

	pub struct ThrusterVisual {
		pub size: f32,
	}

	impl Default for ThrusterVisual {
		fn default() -> Self {
			Self {
				size: 1.0,
			}
		}
	}

	impl ThrusterVisual {
		pub const STRENGTH_ATTR: &'static str = "dynamic_strength_attr";

		pub fn compute_hanabi_effect(self, mut effects: ResMut<Assets<EffectAsset>>) -> ParticleEffect {
			let size = self.size;

			let mut color_gradient = Gradient::new();
			color_gradient.add_key(0.0, Vec4::splat(1.0));
			color_gradient.add_key(0.4, Vec4::new(1.0, 1.0, 0.0, 1.0));
			color_gradient.add_key(0.7, Vec4::new(1.0, 0.0, 0.0, 1.0));
			color_gradient.add_key(1.0, Vec4::new(0.2, 0., 0., 1.));

			let mut size_gradient = Gradient::new();
			size_gradient.add_key(0.0, Vec2::splat(0.1));
			size_gradient.add_key(0.5, Vec2::splat(0.5));
			size_gradient.add_key(1.0, Vec2::splat(0.08));

			let writer = ExprWriter::new();

			let age = (writer.lit(1.) - writer.prop(Self::STRENGTH_ATTR)).expr();
			let init_age1 = SetAttributeModifier::new(Attribute::AGE, age);

			let lifetime = writer.lit(1.).expr();
			let init_lifetime1 = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

			let init_pos1 = SetPositionCone3dModifier {
				base_radius: writer.lit(size * 0.1).expr(),
				top_radius: writer.lit(size * 0.7).expr(),
				height: writer.lit(size * 2.).expr(),
				dimension: ShapeDimension::Volume,
			};

			let init_vel1 = SetVelocitySphereModifier {
				center: writer.lit(Vec3::ZERO).expr(),
				speed: writer.prop(Self::STRENGTH_ATTR).expr(),
			};

			let effect = effects.add(
				EffectAsset::new(
					32768,
					Spawner::rate(500.0.into()).with_starts_active(false),
					writer.finish(),
				)
				.with_name("emit:rate")
				.with_property(Self::STRENGTH_ATTR, Value::from(0.5))
				// .with_property("my_accel", Vec3::new(0., -3., 0.).into())
				.init(init_pos1)
				// Make spawned particles move away from the emitter origin
				.init(init_vel1)
				.init(init_age1)
				.init(init_lifetime1)
				// .update(update_accel1)
				.render(ColorOverLifetimeModifier {
					gradient: color_gradient,
				})
				.render(SizeOverLifetimeModifier {
					gradient: size_gradient,
					screen_space_size: false,
				})
				.render(OrientModifier {
					mode: OrientMode::ParallelCameraDepthPlane,
					..default()
				}),
			);

			ParticleEffect::new(effect)
		}
	}
}
