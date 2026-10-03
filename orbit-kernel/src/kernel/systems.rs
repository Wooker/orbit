use crate::containers::systems::SystemId;

pub const PORT_SYSTEMS: usize = 2;

#[derive(Default)]
pub(super) struct Systems {
    external_ic: Option<usize>,
    port: [Option<usize>; PORT_SYSTEMS],
}

impl Systems {
    #[inline(never)]
    pub fn get_free_mut(&mut self, id: SystemId) -> &mut Option<usize> {
        &mut self.external_ic
    }

    #[inline(never)]
    pub fn find_first_in(&self, id: SystemId) -> Option<usize> {
        match id {
            SystemId::ExternalInterruptController => self.external_ic,
            SystemId::Port => {
                if let Some(entry) = self.port.iter().find(|i| i.is_some()) {
                    *entry
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    // Return the index in systems array
    #[inline(never)]
    pub fn find_by_container_index(&self, id: SystemId, container_index: usize) -> Option<usize> {
        match id {
            SystemId::ExternalInterruptController => {
                if let Some(eic) = self.external_ic
                    && eic == container_index
                {
                    Some(0)
                } else {
                    None
                }
            }
            SystemId::Port => {
                if let Some((idx, _)) = self
                    .port
                    .iter()
                    .enumerate()
                    .find(|(idx, i)| i.is_some() && i.unwrap() == container_index)
                {
                    Some(idx)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    #[inline(never)]
    pub fn insert_in(&mut self, id: SystemId, container_index: usize) {
        match id {
            SystemId::ExternalInterruptController => {
                if let None = self.external_ic {
                    self.external_ic = Some(container_index);
                }
            }
            SystemId::Port => {}
            _ => {}
        }
    }
}
