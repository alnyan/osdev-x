use abi::{error::Error, io::OpenFlags, path};

use crate::{file::FileRef, node::VnodeRef};

pub struct IoContext {
    root: VnodeRef,
    cwd: VnodeRef,
}

impl IoContext {
    pub fn new(root: VnodeRef) -> Self {
        Self {
            cwd: root.clone(),
            root,
        }
    }

    fn _find(&self, mut at: VnodeRef, path: &str, follow: bool) -> Result<VnodeRef, Error> {
        let mut element;
        let mut rest = path;

        loop {
            (element, rest) = path::split_left(rest);

            if !at.is_directory() {
                todo!();
            }

            match element {
                path::PARENT_NAME => {
                    at = at.parent();
                }
                path::SELF_NAME => {}
                _ => break,
            }
        }

        // TODO resolve link target

        if element.is_empty() && rest.is_empty() {
            return Ok(at);
        }

        let node = at.lookup_or_load(element)?;

        if rest.is_empty() {
            Ok(node)
        } else {
            self._find(node, rest, follow)
        }
    }

    pub fn find(
        &self,
        at: Option<VnodeRef>,
        mut path: &str,
        follow: bool,
    ) -> Result<VnodeRef, Error> {
        let at = if path.starts_with('/') {
            path = path.trim_start_matches('/');
            self.root.clone()
        } else if let Some(at) = at {
            at
        } else {
            self.cwd.clone()
        };

        self._find(at, path, follow)
    }

    pub fn open(
        &self,
        at: Option<VnodeRef>,
        path: &str,
        opts: OpenFlags,
    ) -> Result<FileRef, Error> {
        let node = match self.find(at.clone(), path, true) {
            Err(Error::DoesNotExist) => todo!(),
            o => o,
        }?;

        node.open(opts)
    }
}

#[cfg(test)]
mod tests {
    use abi::error::Error;

    use crate::{node::VnodeRef, IoContext};
    use std::fmt;

    macro_rules! node {
        ($name:literal) => {{
            $crate::node::Vnode::new($name, $crate::node::VnodeKind::Regular)
        }};

        ($name:literal [ $($child:expr),* ]) => {{
            let _node = $crate::node::Vnode::new($name, $crate::node::VnodeKind::Directory);

            $(
                _node.add_child($child);
            )*

            _node
        }};
    }

    struct DumpNode<'a> {
        node: &'a VnodeRef,
    }

    impl fmt::Debug for DumpNode<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.node.dump(f, 0)
        }
    }

    #[test]
    fn test_vnode_find() {
        let t = node! {
            "" [
                node!("file1.txt"),
                node!("file2.txt"),
                node! {
                    "dir1" [
                        node!("file3.txt")
                    ]
                }
            ]
        };

        let ctx = IoContext::new(t);

        // Absolute lookups
        assert_eq!(
            ctx.find(None, "/file1.txt", false).unwrap().name(),
            "file1.txt"
        );
        assert_eq!(
            ctx.find(None, "/file3.txt", false).unwrap_err(),
            Error::DoesNotExist
        );
        assert_eq!(
            ctx.find(None, "/dir1/file3.txt", false).unwrap().name(),
            "file3.txt"
        );

        // Non-absolute lookups from root
        assert_eq!(
            ctx.find(None, "file1.txt", false).unwrap().name(),
            "file1.txt"
        );
        assert_eq!(
            ctx.find(None, "dir1/file3.txt", false).unwrap().name(),
            "file3.txt"
        );

        // Absolute lookups from non-root
        let cwd = ctx.find(None, "/dir1", false).unwrap();

        assert_eq!(
            ctx.find(Some(cwd.clone()), "/file1.txt", false)
                .unwrap()
                .name(),
            "file1.txt"
        );
        assert_eq!(
            ctx.find(Some(cwd.clone()), "/dir1/file3.txt", false)
                .unwrap()
                .name(),
            "file3.txt"
        );
        assert_eq!(
            ctx.find(Some(cwd.clone()), "/file3.txt", false)
                .unwrap_err(),
            Error::DoesNotExist
        );
        assert_eq!(
            ctx.find(Some(cwd.clone()), "/dir2", false).unwrap_err(),
            Error::DoesNotExist
        );

        // Non-absolute lookups in non-root
        assert_eq!(
            ctx.find(Some(cwd.clone()), "file3.txt", false)
                .unwrap()
                .name(),
            "file3.txt"
        );
        assert_eq!(
            ctx.find(Some(cwd.clone()), "././file3.txt", false)
                .unwrap()
                .name(),
            "file3.txt"
        );
        assert_eq!(
            ctx.find(Some(cwd.clone()), "../dir1/file3.txt", false)
                .unwrap()
                .name(),
            "file3.txt"
        );
        assert_eq!(
            ctx.find(Some(cwd.clone()), ".", false).unwrap().name(),
            "dir1"
        );
        assert_eq!(ctx.find(Some(cwd.clone()), "..", false).unwrap().name(), "");
        assert_eq!(
            ctx.find(Some(cwd.clone()), "../..", false).unwrap().name(),
            ""
        );
    }
}
