mod blog {
    include!(concat!(env!("OUT_DIR"), "/blog.rs"));
}

pub use blog::*;
