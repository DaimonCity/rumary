use std::path::Path;

struct ValidationService<P: AsRef<Path>> {
    check_list: Vec<P>,
    white_list: Vec<P>,
    black_list: Vec<P>,
}

impl<P: AsRef<Path>> ValidationService<P> {
    //fn new(check_list: Vec<P>, white_list: Vec<P>) -> Self {}
}
