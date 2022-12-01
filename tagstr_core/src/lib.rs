use hashbrown::HashSet;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;
use std::sync::RwLock;

use lazy_static::lazy_static;

lazy_static! {
    static ref TAGS: RwLock<HashSet<&'static str>> = RwLock::new(Default::default());
    static ref UNDEFINED: Tag = "undefined".as_tag();
}

fn value(name: impl AsRef<str>) -> &'static str {
    let name = name.as_ref();
    // try read
    {
        let map = TAGS.read().unwrap();
        if let Some(value) = map.get(name) {
            return value;
        }
    }

    // try read one more time then write
    {
        let mut map = TAGS.write().unwrap();
        if let Some(value) = map.get(name) {
            return value;
        }
        let value = Box::leak(name.to_string().into_boxed_str());
        map.insert(value);
        value
    }
}

pub fn tag(name: impl AsRef<str>) -> Tag {
    Tag(value(name))
}

pub const fn undefined_tag() -> Tag {
    Tag("undefined")
}

#[derive(Clone, Copy)]
pub struct Tag(&'static str);

impl Tag {
    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.0 as *const _ == other.0 as *const _
    }
}

impl Eq for Tag {}

impl Hash for Tag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.0 as *const _, state)
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Debug for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for Tag {
    fn default() -> Self {
        *UNDEFINED
    }
}

impl Deref for Tag {
    type Target = &'static str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for Tag {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl From<Tag> for &str {
    fn from(t: Tag) -> Self {
        t.0
    }
}

impl From<&str> for Tag {
    fn from(t: &str) -> Self {
        tag(t)
    }
}

impl From<Tag> for String {
    fn from(t: Tag) -> Self {
        t.0.to_string()
    }
}

impl From<String> for Tag {
    fn from(t: String) -> Self {
        tag(t)
    }
}

pub trait AsTag {
    fn as_tag(&self) -> Tag;
}

impl AsTag for String {
    fn as_tag(&self) -> Tag {
        tag(self)
    }
}

impl AsTag for &str {
    fn as_tag(&self) -> Tag {
        tag(self)
    }
}

#[cfg(test)]
mod test {
    use super::tag;
    use super::Tag;

    #[test]
    fn tag_equals() {
        let str_tag: Tag = "hello".into();
        let string_tag: Tag = "hello".to_string().into();
        let tag_tag = tag(str_tag);
        assert_eq!(str_tag, string_tag);
        assert_eq!(tag_tag, string_tag);

        let bye_str_tag = tag("goodbye");
        let bye_string_tag: Tag = "goodbye".to_string().into();
        assert_ne!(str_tag, bye_str_tag);
        assert_ne!(string_tag, bye_string_tag);
        assert_ne!(string_tag, bye_str_tag);
    }
}
