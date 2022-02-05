
pub struct ResponseWithCode<T>(pub T, pub u16);

impl<T> ResponseWithCode<T> {

    pub fn ok(response: T) -> ResponseWithCode<T> {
        ResponseWithCode(response, 200)
    }

    pub fn map<F, R>(self,
                     fun: F) -> ResponseWithCode<R>
        where F: FnOnce(T) -> R {
        ResponseWithCode(fun(self.0), self.1)
    }

}
