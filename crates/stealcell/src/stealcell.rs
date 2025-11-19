use core::any::type_name;

#[cfg(not(feature = "no_std"))]
use std::ops::{Deref, DerefMut};

const ALREADY_STOLEN: &str = "value already stolen from:";

/// An Option like type that lets you temporarily remove a value from somewhere
/// to retain mutable access on both.
///
/// It allows you to "steal" its value, taking complete ownership over it, and
/// you must pinky-promise to return it. Non-returned values will panic if
/// dropped!
pub struct StealCell<T> {
	value: Option<T>,
}

/// A value stolen from a [StealCell]. If you accidentally drop it before
/// returning it where it belongs, it will panic!
pub struct Stolen<T> {
	/// Starts out as Some, becomes None once returned.
	/// If not returned, panics!
	value: Option<T>,
}

impl<T> StealCell<T> {
	pub fn new(value: T) -> Self {
		Self { value: Some(value) }
	}

	/// Puts the cell into a "stolen" state and returns the stolen value
	/// which you must promise to return soon!
	///
	/// Panics if already stolen!
	pub fn steal(&mut self) -> Stolen<T> {
		let value = self
			.value
			.take()
			.unwrap_or_else(|| panic!("{ALREADY_STOLEN} {}", type_name::<Self>()));
		Stolen { value: Some(value) }
	}

	pub fn is_stolen(&self) -> bool {
		self.value.is_none()
	}

	/// Panics if wasn't stolen, or if the returned value was already
	/// consumed!
	pub fn return_stolen(&mut self, mut stolen: Stolen<T>) {
		assert!(
			self.value.is_none(),
			"trying to return a stolen value, but this cell is not empty! {}",
			type_name::<Self>()
		);
		assert!(
			stolen.value.is_some(),
			"trying to return a stolen value, but it was already returned! {}",
			type_name::<Self>()
		);
		self.value = Some(stolen.value.take().unwrap());
	}

	/// Panics if stolen!
	pub fn get(&self) -> &T {
		self.value
			.as_ref()
			.unwrap_or_else(|| panic!("{ALREADY_STOLEN} {}", type_name::<Self>()))
	}

	/// Panics if stolen!
	pub fn get_mut(&mut self) -> &mut T {
		self.value
			.as_mut()
			.unwrap_or_else(|| panic!("{ALREADY_STOLEN} {}", type_name::<Self>()))
	}

	/// Panics if stolen!
	#[cfg(not(feature = "no_std"))]
	pub fn as_deref(&self) -> &T::Target
	where
		T: std::ops::Deref,
	{
		self.value
			.as_ref()
			.unwrap_or_else(|| panic!("{ALREADY_STOLEN} {}", type_name::<Self>()))
			.deref()
	}

	/// Panics if stolen!
	#[cfg(not(feature = "no_std"))]
	pub fn as_deref_mut(&mut self) -> &mut T::Target
	where
		T: std::ops::DerefMut,
	{
		self.value
			.as_mut()
			.unwrap_or_else(|| panic!("{ALREADY_STOLEN} {}", type_name::<Self>()))
			.deref_mut()
	}
}

impl<T> Stolen<T> {
	/// PANIC SAFETY: The stored value can only be `None` by giving ownership
	/// away and returning it to the `StealCell`.
	pub fn get(&mut self) -> &T {
		self.value.as_ref().unwrap()
	}

	/// PANIC SAFETY: The stored value can only be `None` by giving ownership
	/// away and returning it to the `StealCell`.
	pub fn get_mut(&mut self) -> &mut T {
		self.value.as_mut().unwrap()
	}
}

#[cfg(not(feature = "no_std"))]
impl<T> Deref for Stolen<T> {
	type Target = T;

	/// PANIC SAFETY: The stored value can only be `None` by giving ownership
	/// away and returning it to the `StealCell`.
	fn deref(&self) -> &Self::Target {
		self.value.as_ref().unwrap()
	}
}

#[cfg(not(feature = "no_std"))]
impl<T> DerefMut for Stolen<T> {
	/// PANIC SAFETY: The stored value can only be `None` by giving ownership
	/// away and returning it to the `StealCell`.
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.value.as_mut().unwrap()
	}
}

impl<T> Drop for Stolen<T> {
	fn drop(&mut self) {
		if self.value.is_some() {
			panic!("You've lost a stolen value without returning it first!");
		}
	}
}

#[cfg(test)]
mod test {
	use crate::StealCell;

	struct Thing {
		value: usize,
	}

	struct World {
		thing: StealCell<Thing>,
	}

	#[test]
	fn it_does_its_job() {
		let mut world = World {
			thing: StealCell::new(Thing { value: 1 }),
		};
		assert!(!world.thing.is_stolen());
		let mut stolen_thing = world.thing.steal();
		assert!(world.thing.is_stolen());
		assert_eq!(stolen_thing.get().value, 1);
		assert_eq!(stolen_thing.get_mut().value, 1);
		world.thing.return_stolen(stolen_thing);
		assert_eq!(world.thing.get().value, 1);
		assert_eq!(world.thing.get_mut().value, 1);
		assert!(!world.thing.is_stolen());
	}
}
