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
///
/// The `Default` implementation defaults to `Some(T)` if `T` is `Default`,
/// as the base assumption of StealCell that there is something in it, unless
/// it was explicitly stolen.
#[derive(PartialEq, Eq, Debug)]
pub struct StealCell<T> {
	value: Option<T>,
}

impl<T> Default for StealCell<T>
where
	T: Default,
{
	fn default() -> Self {
		Self {
			value: Some(T::default()),
		}
	}
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
		// In case we'd need to panic, the value is taken first so that
		// the stolen struct dropping doesn't cause another extra panic.
		let taken_back = stolen.value.take();

		assert!(
			self.value.is_none(),
			"trying to return a stolen value, but this cell is not empty! {}",
			type_name::<Self>()
		);

		assert!(
			taken_back.is_some(),
			"trying to return a stolen value, but it was already returned! {}",
			type_name::<Self>()
		);
		self.value = Some(taken_back.unwrap());
	}
}

impl<T> AsRef<T> for StealCell<T> {
	/// Panics if stolen!
	fn as_ref(&self) -> &T {
		self.value
			.as_ref()
			.unwrap_or_else(|| panic!("{ALREADY_STOLEN} {}", type_name::<Self>()))
	}
}

impl<T> AsMut<T> for StealCell<T> {
	/// Panics if stolen!
	fn as_mut(&mut self) -> &mut T {
		self.value
			.as_mut()
			.unwrap_or_else(|| panic!("{ALREADY_STOLEN} {}", type_name::<Self>()))
	}
}

#[cfg(not(feature = "no_std"))]
impl<T> Deref for StealCell<T> {
	type Target = T;

	/// PANIC SAFETY: The stored value can only be `None` by giving ownership
	/// away and returning it to the `StealCell`.
	fn deref(&self) -> &Self::Target {
		self.as_ref()
	}
}

#[cfg(not(feature = "no_std"))]
impl<T> DerefMut for StealCell<T> {
	/// PANIC SAFETY: The stored value can only be `None` by giving ownership
	/// away and returning it to the `StealCell`.
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.as_mut()
	}
}

/// A value stolen from a [StealCell]. If you accidentally drop it before
/// returning it where it belongs, it will panic!
pub struct Stolen<T> {
	/// Starts out as Some, becomes None once returned.
	/// If not returned, panics!
	value: Option<T>,
}

impl<T> AsRef<T> for Stolen<T> {
	/// PANIC SAFETY: The stored value can only be `None` by you explicitly
	/// returning it to the `StealCell`. As long as you haven't done that,
	/// it's not going to be a None. And after that you can't even call this,
	/// as return gives back ownership to the `StealCell`.
	fn as_ref(&self) -> &T {
		self.value.as_ref().unwrap()
	}
}

impl<T> AsMut<T> for Stolen<T> {
	/// PANIC SAFETY: The stored value can only be `None` by you explicitly
	/// returning it to the `StealCell`. As long as you haven't done that,
	/// it's not going to be a None. And after that you can't even call this,
	/// as return gives back ownership to the `StealCell`.
	fn as_mut(&mut self) -> &mut T {
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
	use crate::{StealCell, Stolen};

	/// Replaces the panic hook with a noop for the duration of the function.
	/// Useful for `#[should_panic]` tests, to ensure backtraces don't pollute
	/// stdout.
	fn mute_panic(fun: impl FnOnce()) {
		let hook = std::panic::take_hook();
		std::panic::set_hook(Box::new(|_| {}));
		fun();
		std::panic::set_hook(hook);
	}

	struct Thing {
		value: usize,
	}

	impl Default for Thing {
		fn default() -> Self {
			Self { value: 99 }
		}
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
		assert_eq!(stolen_thing.as_ref().value, 1);
		assert_eq!(stolen_thing.as_mut().value, 1);
		world.thing.return_stolen(stolen_thing);
		assert_eq!(world.thing.as_ref().value, 1);
		assert_eq!(world.thing.as_mut().value, 1);
		assert!(!world.thing.is_stolen());
	}

	#[test]
	fn defaults() {
		let stealcell = StealCell::<Thing>::default();
		assert_eq!(stealcell.as_ref().value, 99);
	}

	#[test]
	#[cfg(not(feature = "no_std"))]
	fn derefs() {
		let stealcell = StealCell::<usize>::new(12);
		assert_eq!(*stealcell, 12);
	}

	#[test]
	#[cfg(not(feature = "no_std"))]
	fn deref_muts() {
		let mut stealcell = StealCell::<usize>::new(1);
		*stealcell = 12;
		assert_eq!(*stealcell, 12);
	}

	#[test]
	#[should_panic]
	fn panics_on_unnecessary_return() {
		let mut stealcell = StealCell::<usize>::new(1);
		mute_panic(|| stealcell.return_stolen(Stolen { value: Some(12) }));
	}

	#[test]
	#[should_panic]
	fn panics_when_returning_nothing() {
		let mut stealcell = StealCell::<usize>::new(1);
		let mut actual_stolen = stealcell.steal();
		actual_stolen.value = None; // Disarming for the test

		mute_panic(|| stealcell.return_stolen(Stolen { value: None }));
	}

	mod stolen {

		use super::*;
		#[test]
		#[should_panic]
		fn panics_when_dropped() {
			let mut stealcell = StealCell::<Thing>::default();
			let stolen = stealcell.steal();
			mute_panic(|| drop(stolen));
		}

		#[test]
		#[cfg(not(feature = "no_std"))]
		fn derefs() {
			let mut stealcell = StealCell::<usize>::new(12);
			let stolen = stealcell.steal();
			assert_eq!(*stolen, 12);
			stealcell.return_stolen(stolen);
		}

		#[test]
		#[cfg(not(feature = "no_std"))]
		fn deref_muts() {
			let mut stealcell = StealCell::<usize>::default();
			let mut stolen = stealcell.steal();
			*stolen = 42;
			assert_eq!(*stolen, 42);
			stealcell.return_stolen(stolen);
		}
	}
}
