pub trait Droppable {
    fn drop(self);
}
impl<T> Droppable for T {
    fn drop(self) {}
}
