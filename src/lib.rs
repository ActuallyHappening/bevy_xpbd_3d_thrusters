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
	use crate::prelude::*;
}
