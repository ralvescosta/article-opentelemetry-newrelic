pub trait TodoRepository: Send + Sync + 'static {
    fn print(&self);
}
