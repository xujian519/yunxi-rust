pub struct LifecycleManager {
    on_mount_callbacks: Vec<Box<dyn Fn() + Send + Sync>>,
    on_unmount_callbacks: Vec<Box<dyn Fn() + Send + Sync>>,
}

impl LifecycleManager {
    pub fn new() -> Self {
        Self {
            on_mount_callbacks: Vec::new(),
            on_unmount_callbacks: Vec::new(),
        }
    }

    pub fn on_mount(&self) {
        for callback in &self.on_mount_callbacks {
            callback();
        }
    }

    pub fn on_unmount(&self) {
        for callback in &self.on_unmount_callbacks {
            callback();
        }
    }

    pub fn register_on_mount<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_mount_callbacks.push(Box::new(callback));
    }

    pub fn register_on_unmount<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_unmount_callbacks.push(Box::new(callback));
    }
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}
