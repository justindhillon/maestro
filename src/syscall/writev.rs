//! The `writev` system call allows to write sparse data on a file descriptor.

use crate::util::lock::IntMutex;
use crate::errno;
use crate::errno::Errno;
use crate::file::open_file::OpenFile;
use crate::idt;
use crate::limits;
use crate::process::iovec::IOVec;
use crate::process::mem_space::ptr::SyscallSlice;
use crate::process::mem_space::MemSpace;
use crate::process::signal::Signal;
use crate::process::Process;
use crate::util::io::IO;
use crate::util::ptr::arc::Arc;
use core::cmp::min;
use core::ffi::c_int;
use macros::syscall;

// TODO Handle blocking writes (and thus, EINTR)

/// Writes the given chunks to the file.
///
/// Arguments:
/// - `mem_space` is the memory space of the current process.
/// - `iov` is the set of chunks.
/// - `iovcnt` is the number of chunks in `iov`.
/// - `open_file` is the file to write to.
fn write(
	mem_space: Arc<IntMutex<MemSpace>>,
	iov: SyscallSlice<IOVec>,
	iovcnt: usize,
	open_file: &mut OpenFile,
) -> Result<i32, Errno> {
	let mem_space_guard = mem_space.lock();
	let iov_slice = iov.get(&mem_space_guard, iovcnt)?.ok_or(errno!(EFAULT))?;

	let mut total_len = 0;

	for i in iov_slice {
		// Ignoring zero entry
		if i.iov_len == 0 {
			continue;
		}

		// The size to write. This is limited to avoid an overflow on the total length
		let l = min(i.iov_len, i32::MAX as usize - total_len);
		let ptr = SyscallSlice::<u8>::from(i.iov_base as usize);

		if let Some(slice) = ptr.get(&mem_space_guard, l)? {
			// TODO Handle in a loop?
			total_len += open_file.write(0, slice)? as usize;
		}
	}

	Ok(total_len as _)
}

/// Peforms the writev operation.
///
/// Arguments:
/// - `fd` is the file descriptor.
/// - `iov` the IO vector.
/// - `iovcnt` the number of entries in the IO vector.
/// - `offset` is the offset in the file.
/// - `flags` is the set of flags.
pub fn do_writev(
	fd: i32,
	iov: SyscallSlice<IOVec>,
	iovcnt: i32,
	offset: Option<isize>,
	_flags: Option<i32>,
) -> Result<i32, Errno> {
	if fd < 0 {
		return Err(errno!(EBADF));
	}
	// Checking the size of the vector is in bounds
	if iovcnt < 0 || iovcnt as usize > limits::IOV_MAX {
		return Err(errno!(EINVAL));
	}

	// TODO Handle flags

	let (mem_space, open_file_mutex) = {
		let proc_mutex = Process::get_current().unwrap();
		let proc = proc_mutex.lock();

		let mem_space = proc.get_mem_space().unwrap();

		let fds_mutex = proc.get_fds().unwrap();
		let fds = fds_mutex.lock();

		let open_file_mutex = fds.get_fd(fd as _).ok_or(errno!(EBADF))?.get_open_file()?;
		(mem_space, open_file_mutex)
	};

	idt::wrap_disable_interrupts(|| {
		let mut open_file = open_file_mutex.lock();

		// The offset to restore on the fd after the write operation
		let mut prev_off = None;
		// Setting the offset temporarily
		if let Some(offset) = offset {
			if offset < -1 {
				return Err(errno!(EINVAL));
			}

			if offset != -1 {
				prev_off = Some(open_file.get_offset());
				open_file.set_offset(offset as _);
			}
		}

		let result = write(mem_space, iov, iovcnt as _, &mut *open_file);
		match &result {
			// If writing to a broken pipe, kill with SIGPIPE
			Err(e) if e.as_int() == errno::EPIPE => {
				let proc_mutex = Process::get_current().unwrap();
				let mut proc = proc_mutex.lock();

				proc.kill(&Signal::SIGPIPE, false);
			}

			_ => {}
		}

		// Restoring previous offset
		if let Some(prev_off) = prev_off {
			open_file.set_offset(prev_off);
		}

		result
	})
}

#[syscall]
pub fn writev(fd: c_int, iov: SyscallSlice<IOVec>, iovcnt: c_int) -> Result<i32, Errno> {
	do_writev(fd, iov, iovcnt, None, None)
}
