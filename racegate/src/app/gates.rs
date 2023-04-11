#[derive(Debug, serde::Serialize, serde::Deserialize, Default, Clone, Eq, PartialEq)]
pub struct Gate {
    active: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default, Clone, Eq, PartialEq)]
pub struct Gates {
    items: [Gate; 16],
}

impl Gates {
    pub fn start_gate(&self) -> &Gate {
        &self.items[0]
    }
    pub fn finish_gate(&self) -> &Gate {
        &self.items[17]
    }
}
