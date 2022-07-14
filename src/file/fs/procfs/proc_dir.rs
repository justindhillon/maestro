//! This module implements the directory of a process in the procfs.

use crate::errno::Errno;
use crate::file::FileContent;
use crate::file::Gid;
use crate::file::Mode;
use crate::file::Uid;
use crate::file::fs::kernfs::KernFS;
use crate::file::fs::kernfs::node::KernFSNode;
use crate::file;
use crate::process::pid::Pid;
use crate::time::unit::Timestamp;
use crate::util::IO;
use crate::util::container::hashmap::HashMap;
use crate::util::ptr::cow::Cow;

/// Structure representing the directory of a process.
pub struct ProcDir {
	/// The PID of the process.
	pid: Pid,

	/// The content of the directory. This will always be a Directory variant.
	content: FileContent,
}

impl ProcDir {
	/// Creates a new instance for the process with the given PID `pid`.
	/// The function adds every nodes to the given kernfs `fs`.
	pub fn new(pid: Pid, _fs: &mut KernFS) -> Result<Self, Errno> {
		let entries = HashMap::new();
		// TODO Add every nodes to the fs

		Ok(Self {
			pid,

			content: FileContent::Directory(entries),
		})
	}
}

impl KernFSNode for ProcDir {
	fn get_hard_links_count(&self) -> u16 {
		1
	}

	fn set_hard_links_count(&mut self, _: u16) {}

	fn get_mode(&self) -> Mode {
		0o777
	}

	fn set_mode(&mut self, _: Mode) {}

	fn get_uid(&self) -> Uid {
		file::ROOT_UID
	}

	fn set_uid(&mut self, _: Uid) {}

	fn get_gid(&self) -> Gid {
		file::ROOT_GID
	}

	fn set_gid(&mut self, _: Gid) {}

	fn get_atime(&self) -> Timestamp {
		0
	}

	fn set_atime(&mut self, _: Timestamp) {}

	fn get_ctime(&self) -> Timestamp {
		0
	}

	fn set_ctime(&mut self, _: Timestamp) {}

	fn get_mtime(&self) -> Timestamp {
		0
	}

	fn set_mtime(&mut self, _: Timestamp) {}

	fn get_content<'a>(&'a self) -> Cow<'a, FileContent> {
		Cow::from(&self.content)
	}

	fn set_content(&mut self, _: FileContent) {}
}

impl IO for ProcDir {
	fn get_size(&self) -> u64 {
		0
	}

	fn read(&mut self, _offset: u64, _buff: &mut [u8]) -> Result<u64, Errno> {
		Err(errno!(EINVAL))
	}

	fn write(&mut self, _offset: u64, _buff: &[u8]) -> Result<u64, Errno> {
		Err(errno!(EINVAL))
	}
}
