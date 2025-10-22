use std::hash::{Hash, Hasher};

use string_cache::DefaultAtom as FName;

#[derive(Debug, Eq, Clone, Ord, PartialOrd)]
pub struct GameplayTag {
    //标签完整名字
    tag_name: FName,
}

impl PartialEq for GameplayTag {
    fn eq(&self, other: &Self) -> bool {
        self.tag_name == other.tag_name
    }
}

impl Hash for GameplayTag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tag_name.hash(state);
    }
}

impl GameplayTag {
    pub fn new(full_name: FName) -> GameplayTag {
        GameplayTag {
            tag_name: full_name,
        }
    }
}
