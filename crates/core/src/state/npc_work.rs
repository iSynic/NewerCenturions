use crate::Session;

impl Session {
    pub(crate) fn retain_npc_work(&mut self) {
        if let Some(value) = self
            .combat
            .as_mut()
            .and_then(|c| c.npc_inhibit_write.take())
        {
            self.native_work.npc_work.inhibit = value;
        }
    }

    pub(crate) fn write_npc_inhibit(&mut self, value: i16) {
        self.retain_npc_work();
        self.native_work.npc_work.inhibit = value;
    }

    pub(crate) fn reset_npc_work(&mut self) {
        self.retain_npc_work();
        self.native_work.npc_work.getup();
    }
}
