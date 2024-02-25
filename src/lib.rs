#![doc = include_str!("../README.md")]
#![allow(clippy::type_complexity)]

pub mod prelude {
	pub(crate) use bevy::prelude::*;
	pub use bevy_inspector_egui::prelude::*;
	pub use bevy_xpbd_3d::prelude::*;
	pub use bevy_xpbd_3d_parenting::prelude::*;
	pub(crate) use serde::{Deserialize, Serialize};

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
		#[inspector(min = 0.0, max = 1.0)]
		strength: f32,
	}

	impl Thruster {
		pub fn new(strength: f32) -> Self {
			Self { strength: strength.clamp(0.0, 1.0) }
		}
		
		pub fn strength(&self) -> f32 {
			self.strength.clamp(0.0, 1.0)
		}

		pub fn get_strength(&self) -> f32 {
			self.strength()
		}

		pub fn set_strength(&mut self, strength: f32) {
			self.strength = strength.clamp(0.0, 1.0);
		}

		/// # Safety
		/// Must keep the strength between 0.0 and 1.0.
		pub unsafe fn get_mut_strength(&mut self) -> &mut f32 {
			&mut self.strength
		}
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

	#[derive(Component, Reflect, Debug, Clone, Serialize, Deserialize, InspectorOptions)]
	#[reflect(Component, InspectorOptions)]
	pub struct ThrusterVisual {
		#[inspector(min = 0.1, max = 100.0)]
		pub size: f32,
	}

	impl Default for ThrusterVisual {
		fn default() -> Self {
			Self { size: 1.0 }
		}
	}

	/// What is spawned by [ThrusterVisual::auto_expand].
	/// Based on [ParticleEffectBundle].
	#[derive(Bundle, Debug)]
	pub struct ThrusterVisualBundle {
		particle_effect: ParticleEffect,
		compiled_effect: CompiledParticleEffect,
	}

	impl ThrusterVisual {
		pub const STRENGTH_ATTR: &'static str = "dynamic_strength_attr";

		/// Automatically expands any [ThrusterVisual] components into a full [ParticleEffectBundle] (without overriding existing components).
		pub fn auto_expand(
			thruster_visuals: Query<(Entity, &ThrusterVisual), (With<Thruster>, Without<ParticleEffect>)>,
			mut commands: Commands,
			mut effects: ResMut<Assets<EffectAsset>>,
		) {
			for (e, thruster_visual) in thruster_visuals.iter() {
				let effect = thruster_visual.compute_hanabi_effect(&mut effects);
				#[cfg(feature = "debug")]
				trace!(
					"Expanded a ThrusterVisual into a ParticleEffectBundle: {:?}",
					thruster_visual
				);
				commands.entity(e).insert(ThrusterVisualBundle {
					particle_effect: effect,
					compiled_effect: CompiledParticleEffect::default(),
				});
			}
		}

		pub fn auto_sync(
			mut thrusters: Query<
				(&Thruster, &mut EffectProperties, &mut EffectSpawner),
				With<ThrusterVisual>,
			>,
		) {
			for (thruster, properties, mut spawner) in thrusters.iter_mut() {
				EffectProperties::set_if_changed(
					properties,
					Self::STRENGTH_ATTR,
					thruster.strength().into(),
				);
				if thruster.strength() == 0. {
					spawner.set_active(false);
				} else {
					spawner.set_active(true);
				}
			}
		}

		pub fn compute_hanabi_effect(&self, effects: &mut Assets<EffectAsset>) -> ParticleEffect {
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

			let lifetime = writer.lit(3.).expr();
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
				.with_property(Self::STRENGTH_ATTR, Value::from(0.0))
				.init(init_pos1)
				// Make spawned particles move away from the emitter origin
				.init(init_vel1)
				.init(init_age1)
				.init(init_lifetime1)
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
