use rand::{distributions, prelude::*};
use std::path::PathBuf;

pub struct Action {
    resource: Resource,
    action: ResourceAction,
    paths: Vec<PathBuf>,
}

impl Action {
    pub fn resource(&self) -> &Resource {
        &self.resource
    }

    pub fn action(&self) -> &ResourceAction {
        &self.action
    }

    pub fn add_path(&mut self, path: PathBuf) {
        self.paths.push(path);
    }
}

impl Distribution<Action> for distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Action {
        Action {
            resource: self.sample(rng),
            action: self.sample(rng),
            paths: Vec::with_capacity(2),
        }
    }
}

pub enum Resource {
    File,
    Folder,
}

pub enum ResourceAction {
    Create,
    Remove,
    Rename,
    Move,
    Copy,
}

impl Distribution<ResourceAction> for distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ResourceAction {
        match rng.gen_range(0..5) {
            0 => ResourceAction::Create,
            1 => ResourceAction::Remove,
            2 => ResourceAction::Rename,
            3 => ResourceAction::Move,
            4 => ResourceAction::Copy,
            _ => unreachable!(),
        }
    }
}

impl Distribution<Resource> for distributions::Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Resource {
        match rng.gen_range(0..2) {
            0 => Resource::File,
            1 => Resource::Folder,
            _ => unreachable!(),
        }
    }
}
