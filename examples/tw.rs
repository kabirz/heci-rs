use heci_rs::*;

fn main() {
	let ht = Heci::new(UVSS_GUID);
	let ret = ht.connect(HECI_TEST);
	println!("ret = {}", ret);
	let ret = ht.write("hello".as_bytes());
	println!("ret = {}", ret);
	ht.close();
}