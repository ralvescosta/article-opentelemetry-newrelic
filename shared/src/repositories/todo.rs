use opentelemetry::Context;

pub trait TodoRepository: Send + Sync + 'static {
    fn print(&self, ctx: &Context);
}
