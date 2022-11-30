mod parse;

use crate::fs::INodeExt;
use alloc::sync::Arc;
use crate::fs::vfs::FileSystem;
use parse::parse_zcore_elf;

pub fn init_debuginfo(rootfs: &Arc<dyn FileSystem>) {
    match rootfs.root_inode().lookup("./zcore") {
        Ok(inode) => {
            let data = inode.read_as_vec().unwrap();
            info!("zCore found, Loading debuginfo...");
            parse_zcore_elf(data);
        }
        Err(e) => error!("failed to lookup /zcore: {:?}", e)
    }
}