extern crate remote_trait_object_macro;

#[remote_trait_object_macro::service]
pub trait MyHandle: Service {
    fn fn_1(&self, a: i32, b: String) -> String;
    type What;
}

fn main() {

}