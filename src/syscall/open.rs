//! The open system call allows a process to open a file and get a file descriptor.

use crate::errno::Errno;
use crate::errno;
use crate::file::File;
use crate::file::FileContent;
use crate::file::FileType;
use crate::file::Gid;
use crate::file::Mode;
use crate::file::Uid;
use crate::file::fcache;
use crate::file::file_descriptor::FDTarget;
use crate::file::file_descriptor;
use crate::file::path::Path;
use crate::file;
use crate::process::Process;
use crate::process::regs::Regs;
use crate::util::FailableClone;
use crate::util::ptr::SharedPtr;

// TODO Implement all flags

/// Returns the file at the given path `path`.
/// If the file doesn't exist and the O_CREAT flag is set, the file is created, then the function
/// returns it. If the flag is not set, the function returns an error with the appropriate errno.
/// If the file is to be created, the function uses `mode` to set its permissions and `uid and
/// `gid` to set the user ID and group ID.
fn get_file(path: Path, flags: i32, mode: Mode, uid: Uid, gid: Gid)
	-> Result<SharedPtr<File>, Errno> {
	let mutex = fcache::get();
	let mut guard = mutex.lock();
	let files_cache = guard.get_mut().as_mut().unwrap();

	// Getting the path of the parent directory
	let mut parent_path = path.failable_clone()?;
	parent_path.pop();

	// The parent directory
	let parent_mutex = files_cache.get_file_from_path(&parent_path, uid, gid, true)?;
	let mut parent_guard = parent_mutex.lock();
	let parent = parent_guard.get_mut();

	// The file's basename
	let name = path[path.get_elements_count() - 1].failable_clone()?;

	// Tells whether to follow symbolic links on the last component of the path.
	let follow_links = flags & file_descriptor::O_NOFOLLOW == 0;

	let file_result = files_cache.get_file_from_parent(parent, name.failable_clone()?, uid, gid,
		follow_links);
	if let Ok(file) = file_result {
		// If the file is found, return it
		Ok(file)
	} else if flags & file_descriptor::O_CREAT != 0 {
		// Creating the file
		files_cache.create_file(parent, name, uid, gid, mode, FileContent::Regular)
	} else {
		Err(errno!(ENOENT))
	}
}

/// Performs the open system call.
pub fn open_(pathname: *const u8, flags: i32, mode: file::Mode) -> Result<i32, Errno> {
	let mutex = Process::get_current().unwrap();
	let mut guard = mutex.lock();
	let proc = guard.get_mut();

	// Getting the path string
	let path_str = super::util::get_str(proc, pathname)?;

	let mode = mode & !proc.get_umask();
	let uid = proc.get_euid();
	let gid = proc.get_egid();

	// Getting the file
	let abs_path = super::util::get_absolute_path(&proc, Path::from_str(path_str, true)?)?;
	let file = get_file(abs_path, flags, mode, uid, gid)?;

	// If O_DIRECTORY is set and the file is not a directory, return an error
	if flags & file_descriptor::O_DIRECTORY != 0
		&& file.lock().get().get_file_type() != FileType::Directory {
		return Err(errno!(ENOTDIR));
	}

	// Create and return the file descriptor
	let fd = proc.create_fd(flags, FDTarget::File(file))?;
	Ok(fd.get_id() as _)
}

/// The implementation of the `open` syscall.
pub fn open(regs: &Regs) -> Result<i32, Errno> {
	let pathname = regs.ebx as *const u8;
	let flags = regs.ecx as i32;
	let mode = regs.edx as file::Mode;

	open_(pathname, flags, mode)
}
