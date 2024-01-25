use super::*;

#[test]
fn derive() {
    #[derive(HasId)]
    struct Test {
        #[id]
        pub rid: usize,

        #[allow(dead_code)]
        pub data: usize,
    }
}

#[test]
fn derive_serde() {
    #[derive(HasIdSerde)]
    struct TestSerde {
        #[id]
        pub rid: usize,

        #[allow(dead_code)]
        pub data: usize,
    }
}
