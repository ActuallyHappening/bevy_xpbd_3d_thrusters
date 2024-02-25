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