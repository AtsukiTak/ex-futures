pub struct Should<T>(Option<T>);

impl<T> Should<T> {
    pub fn new(item: T) -> Should<T> {
        Should(Some(item))
    }

    pub fn as_ref(&self) -> &T {
        self.0.as_ref().expect(
            "You never use item which is already taken",
        )
    }

    pub fn as_mut(&mut self) -> &mut T {
        self.0.as_mut().expect(
            "You never use item which is already taken",
        )
    }

    pub fn take(&mut self) -> T {
        self.0.take().expect(
            "You never use item which is already taken",
        )
    }
}
