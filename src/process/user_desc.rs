//! This module implements the `user_desc` structure, which is used in userspace to specify the
//! value for a descriptor, either a local or global descriptor.

use crate::gdt;
use core::ffi::c_void;
use core::fmt;

/// The size of the user_desc structure in bytes.
pub const USER_DESC_SIZE: usize = 16;

/// The `user_desc` structure.
#[repr(transparent)]
pub struct UserDesc {
	val: [i8; USER_DESC_SIZE],
}

impl UserDesc {
	/// Returns the entry number.
	#[inline(always)]
	pub fn get_entry_number(&self) -> i32 {
		unsafe {
			// Safe because the structure is large enough
			*(&self.val[0] as *const _ as *const i32)
		}
	}

	/// Sets the entry number.
	pub fn set_entry_number(&mut self, number: i32) {
		unsafe {
			// Safe because the structure is large enough
			*(&mut self.val[0] as *mut _ as *mut i32) = number;
		}
	}

	/// Returns the base address.
	#[inline(always)]
	pub fn get_base_addr(&self) -> i32 {
		unsafe {
			// Safe because the structure is large enough
			*(&self.val[4] as *const _ as *const i32)
		}
	}

	/// Returns the limit.
	#[inline(always)]
	pub fn get_limit(&self) -> i32 {
		unsafe {
			// Safe because the structure is large enough
			*(&self.val[8] as *const _ as *const i32)
		}
	}

	/// Tells whether the segment is 32 bits.
	#[inline(always)]
	pub fn is_32bits(&self) -> bool {
		(self.val[12] & 0b1) != 0
	}

	/// Tells whether the segment is writable.
	#[inline(always)]
	pub fn is_read_exec_only(&self) -> bool {
		(self.val[12] & 0b1000) == 0
	}

	/// Tells whether the segment's limit is in number of pages.
	#[inline(always)]
	pub fn is_limit_in_pages(&self) -> bool {
		(self.val[12] & 0b10000) != 0
	}

	/// Tells whether the segment is present.
	#[inline(always)]
	pub fn is_present(&self) -> bool {
		(self.val[12] & 0b100000) == 0
	}

	/// Tells whether the segment is usable.
	#[inline(always)]
	pub fn is_usable(&self) -> bool {
		(self.val[12] & 0b1000000) != 0
	}

	/// Converts the current descriptor to a GDT entry.
	pub fn to_descriptor(&self) -> gdt::Entry {
		let mut entry = gdt::Entry::default();

		entry.set_base(self.get_base_addr() as _);
		entry.set_limit(self.get_limit() as _);

		// TODO contents

		let mut access_byte = 0b01110010;
		if self.is_present() && self.is_usable() {
			access_byte |= 1 << 7;
		}
		if self.is_read_exec_only() {
			access_byte |= 1 << 3;
		}
		entry.set_access_byte(access_byte);

		let mut flags = 0b0000;
		if self.is_32bits() {
			flags |= 1 << 2;
		}
		if self.is_limit_in_pages() {
			flags |= 1 << 3;
		}
		entry.set_flags(flags);

		entry
	}
}

impl fmt::Display for UserDesc {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		writeln!(f, "entry_number: {}", self.get_entry_number())?;
		writeln!(f, "base_addr: {:p}", self.get_base_addr() as *const c_void)?;
		writeln!(f, "limit: {:x}", self.get_limit())?;
		writeln!(f, "seg_32bit: {}", self.is_32bits())?;
		// TODO writeln!(f, "contents: {}", self.)?;
		writeln!(f, "read_exec_only: {}", !self.is_read_exec_only())?;
		writeln!(f, "limit_in_pages: {}", self.is_limit_in_pages())?;
		writeln!(f, "seg_not_present: {}", !self.is_present())?;
		write!(f, "useable: {}", self.is_usable())
	}
}