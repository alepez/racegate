use crate::svc::race_node::NodeAddress;

#[derive(Debug, serde::Serialize, serde::Deserialize, Default, Clone, Eq, PartialEq)]
pub struct Gate {
    pub active: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default, Clone, Eq, PartialEq)]
pub struct Gates {
    items: [Gate; 4],
}

impl Gates {
    pub fn start_gate(&self) -> &Gate {
        const INDEX: usize = NodeAddress::start().unwrap_as_gate_index();
        &self.items[INDEX]
    }

    pub fn finish_gate(&self) -> &Gate {
        const INDEX: usize = NodeAddress::finish().unwrap_as_gate_index();
        &self.items[INDEX]
    }

    pub fn get_mut_from_addr(&mut self, addr: NodeAddress) -> Option<&mut Gate> {
        let index = addr.as_gate_index()?;
        Some(&mut self.items[index])
    }
}
