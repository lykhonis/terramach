/*
 * Terra Mach
 * Copyright [2020] Terra Mach Authors
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>
 */

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use crate::IndexPool;

pub type Id = usize;

#[derive(Debug, Clone)]
struct Node<T> {
    id: Id,
    inner: Option<T>,
}

impl<T> Node<T> {
    pub fn new(id: Id, data: impl Into<Option<T>>) -> Self {
        Self {
            id,
            inner: data.into(),
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn data(self) -> Option<T> {
        self.inner
    }
}

impl<T> Deref for Node<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Node<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Debug, Clone)]
pub struct Tree<T> {
    ids: IndexPool,
    root: Id,
    nodes: HashMap<Id, Node<T>>,
    child_parent: HashMap<Id, Id>,
    parent_children: HashMap<Id, Vec<Id>>,
}

impl<T> Tree<T> {
    pub fn new() -> Self {
        let mut indices = IndexPool::new();
        let root = indices.take();
        let mut nodes = HashMap::new();
        nodes.insert(root, Node::new(root, None));
        Self {
            ids: indices,
            root,
            nodes,
            child_parent: HashMap::new(),
            parent_children: HashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.len() == 1 /*root*/
    }

    pub fn len(&self) -> usize {
        self.nodes.len() - 1
    }

    pub fn insert(&mut self, node: T, parent: impl Into<Option<Id>>) -> Id {
        let id = self.ids.take();
        let parent = parent.into().unwrap_or(self.root);
        let node = Node::new(id, node);
        self.nodes.insert(id, node);
        self.child_parent.insert(id, parent);
        if let Some(children) = self.parent_children.get_mut(&parent) {
            children.push(id);
        } else {
            self.parent_children.insert(parent, vec![id]);
        }
        id
    }

    pub fn replace(&mut self, id: Id, node: T) -> Option<T> {
        if id == self.root {
            return None;
        }
        let node = Node::new(id, node);
        self.nodes.insert(id, node)?.inner
    }

    pub fn remove(&mut self, id: Id) -> Option<T> {
        if id == self.root {
            return None;
        }
        let mut node = self.nodes.remove(&id)?;
        self.ids.give(id);
        let parent = self.child_parent.remove(&id).unwrap();
        if let Some(children) = self.parent_children.remove(&id) {
            for child in children {
                self.child_parent.insert(child, parent);
                if let Some(children) = self.parent_children.get_mut(&parent) {
                    children.push(child);
                } else {
                    self.parent_children.insert(parent, vec![child]);
                }
            }
        }
        node.inner.take()
    }

    pub fn remove_all(&mut self, id: Id) -> Option<Vec<Id>> {
        if id == self.root {
            return None;
        }
        self.nodes.remove(&id)?;
        let parent = self.child_parent.remove(&id).unwrap();
        if let Some(children) = self.parent_children.get_mut(&parent) {
            children.remove_item(&id);
        }
        let mut removed = vec![id];
        self.ids.give(id);
        if let Some(children) = self.parent_children.remove(&id) {
            for child in children {
                if let Some(removed_children) = self.remove_all(child) {
                    removed.extend(removed_children);
                }
            }
        }
        Some(removed)
    }

    pub fn node(&self, id: Id) -> Option<&T> {
        self.nodes.get(&id)?.deref().as_ref()
    }

    pub fn node_mut(&mut self, id: Id) -> Option<&mut T> {
        self.nodes.get_mut(&id)?.deref_mut().as_mut()
    }

    pub fn parent(&self, child: Id) -> Option<Id> {
        let parent = self.child_parent.get(&child).map(|id| *id)?;
        if parent == self.root {
            None
        } else {
            Some(parent)
        }
    }

    pub fn child_count(&self, parent: impl Into<Option<Id>>) -> usize {
        if let Some(parent) = parent.into() {
            if let Some(children) = self.parent_children.get(&parent) {
                return children.len();
            }
        }
        0
    }

    pub fn children(&self, parent: impl Into<Option<Id>>) -> Option<&Vec<Id>> {
        if let Some(parent) = parent.into() {
            self.parent_children.get(&parent)
        } else {
            self.parent_children.get(&self.root)
        }
    }
}
