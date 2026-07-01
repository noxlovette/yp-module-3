mod blog {
    include!(concat!(env!("OUT_DIR"), "/blog.v1.rs"));
}

pub use blog::*;
