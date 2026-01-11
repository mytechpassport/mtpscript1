// Placeholder for Interpreter struct
pub struct Interpreter {
    pub heap: Vec<u8>,
    pub pci_touched: bool,
}

impl Interpreter {
    pub fn zero_sensitive(&mut self) {
        if self.pci_touched {
            for byte in &mut self.heap {
                *byte = 0;
            }
        }
    }
}

pub fn wipe_interpreter(mut interp: Interpreter, pci: bool) {
    if pci {
        interp.zero_sensitive();
    }
    drop(interp);
}
