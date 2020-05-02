pub struct Core;

impl Core {
    pub fn new() -> Core {
        Core
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Core;
    #[test]
    fn create_core() {
        let core = Core::new();
    }
}